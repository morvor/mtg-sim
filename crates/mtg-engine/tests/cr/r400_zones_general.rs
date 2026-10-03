//! CR 400: zones in general — which zones exist and who they belong to, public and hidden
//! zones, owners' zones, cards that can't enter or leave zones, order, moving objects,
//! outside the game, and actions performed on whole zones.

use crate::r600_common::*;
use crate::r703_common::{run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The given objects, which are in a zone of this kind.
fn in_zone(zone: ZoneKind, ids: Vec<ObjectId>) -> Sel {
    Sel::All(Filter::and(vec![Filter::InZone(zone), Filter::Objects(ids)]))
}

fn sideboard(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.custom(p, (*card(name)).clone(), Zone::Outside(p))
}

fn move_to(t: &mut TestGame, id: ObjectId, to: Zone) -> Option<ObjectId> {
    t.g.move_object(id, to, MoveCause::Effect, Some(P0))
}

// ---------------------------------------------------------------------------
// 400.1–400.2
// ---------------------------------------------------------------------------

#[test]
fn each_player_has_their_own_library_hand_and_graveyard_and_shares_the_other_zones() {
    cr!("400.1", "400.2");
    let mut t = TestGame::new(2);
    // Libraries, hands, and graveyards are per player.
    let g0 = t.graveyard(P0, "Grizzly Bears");
    let g1 = t.graveyard(P1, "Hill Giant");
    assert_eq!(t.zone(g0), Zone::Graveyard(P0));
    assert_eq!(t.zone(g1), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 1);
    assert_ne!(t.g.player(P0).library, t.g.player(P1).library);
    // Exile is shared.
    let e0 = move_to(&mut t, g0, Zone::Exile).unwrap();
    let e1 = move_to(&mut t, g1, Zone::Exile).unwrap();
    assert_eq!(t.g.exile, vec![e0, e1]);
    // The battlefield is shared.
    let b0 = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Hill Giant");
    assert!(t.g.battlefield.contains(&b0) && t.g.battlefield.contains(&b1));
    // So is the stack: both players' spells are on one stack.
    let s0 = t.custom(P0, free_instant("Mine"), Zone::Hand(P0));
    let s1 = t.custom(P1, free_instant("Theirs"), Zone::Hand(P1));
    let s0 = t.cast(P0, s0).go();
    let s1 = t.cast(P1, s1).go();
    assert_eq!(t.g.stack, vec![s0, s1]);
    t.resolve_all();
    // And the command zone: both players' emblems are in it.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CreateEmblem {
            who: PlayerRef::EachPlayer,
            abilities: vec![],
        },
        &[],
    );
    let emblems: Vec<ObjectId> = t
        .g
        .command
        .iter()
        .copied()
        .filter(|id| t.obj(*id).kind == ObjKind::Emblem)
        .collect();
    assert_eq!(emblems.len(), 2);
    // Objects in the public zones can be seen by every player; the ante zone is one of
    // them.
    let ante = t.custom(P1, free_instant("Stake"), Zone::Ante);
    for id in [e0, e1, b0, b1, emblems[0], emblems[1], ante] {
        assert!(can_look_at(&t.g, P0, id) && can_look_at(&t.g, P1, id));
    }
}

fn free_instant(name: &str) -> CardDef {
    CB::new(name)
        .instant()
        .cost("{0}")
        .spell(Body::effect(gain(1)))
        .build()
}

#[test]
fn a_hand_stays_hidden_even_when_all_its_cards_are_revealed() {
    cr!("400.2");
    assert!(!Zone::Hand(P0).is_public());
    assert!(!Zone::Library(P0).is_public());
    for z in [
        Zone::Graveyard(P0),
        Zone::Battlefield,
        Zone::Stack,
        Zone::Exile,
        Zone::Ante,
        Zone::Command,
    ] {
        assert!(z.is_public(), "{z:?}");
    }
    // A card put into a hidden zone from a public one is a new object that no longer
    // can be seen by other players.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(can_look_at(&t.g, P1, bears));
    let h = move_to(&mut t, bears, Zone::Hand(P0)).unwrap();
    assert!(!can_look_at(&t.g, P1, h));
}

// ---------------------------------------------------------------------------
// 400.3: owners' zones
// ---------------------------------------------------------------------------

#[test]
fn an_object_goes_to_its_owners_library_hand_or_graveyard() {
    cr!("400.3");
    supported("Control Magic");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let magic = t.hand(P0, "Control Magic");
    t.cast(P0, magic).target(bears).go();
    t.resolve();
    assert_eq!(t.obj(bears).controller, P0);
    // Destroyed while P0 controls it: it goes to P1's graveyard.
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P0), 1, "only Control Magic");
    // An effect that tries to put another player's card into your hand, library, or
    // graveyard puts it into its owner's instead.
    let giant = t.battlefield(P1, "Hill Giant");
    for (to, expected) in [
        (Zone::Hand(P0), Zone::Hand(P1)),
        (Zone::Library(P0), Zone::Library(P1)),
        (Zone::Graveyard(P0), Zone::Graveyard(P1)),
    ] {
        let now = t.g.current(giant);
        move_to(&mut t, now, to);
        assert_eq!(t.zone(giant), expected);
    }
    // Other zones aren't owned: exile is exile.
    let now = t.g.current(giant);
    move_to(&mut t, now, Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
}

// ---------------------------------------------------------------------------
// 400.4: cards that can't enter or leave zones
// ---------------------------------------------------------------------------

#[test]
fn instant_and_sorcery_cards_cant_enter_the_battlefield() {
    cr!("400.4", "400.4a");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let wrath = t.hand(P0, "Wrath of God");
    let bears = t.graveyard(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::Union(vec![
                in_zone(ZoneKind::Graveyard, vec![bolt, bears]),
                in_zone(ZoneKind::Hand, vec![wrath]),
            ]),
            to: Destination::battlefield(),
        },
        &[],
    );
    // They remain in their previous zones (as the same objects); the creature card
    // enters.
    assert!(t.is_live(bolt) && t.zone(bolt) == Zone::Graveyard(P0));
    assert!(t.is_live(wrath) && t.zone(wrath) == Zone::Hand(P0));
    assert!(t.on_battlefield(bears));
}

#[test]
fn nontraditional_command_zone_cards_cant_leave_the_command_zone() {
    cr!("400.4", "400.4b");
    let mut t = TestGame::new(2);
    let plane = t.command(P0, "Goldmeadow");
    assert!(t.obj(plane).is(CardType::Plane));
    let scheme = t.custom(
        P0,
        crate::r703_common::oracle_card("Idle Scheme", "Scheme", "", None, ""),
        Zone::Command,
    );
    let vanguard = t.custom(
        P0,
        crate::r703_common::oracle_card("Idle Avatar", "Vanguard", "", None, ""),
        Zone::Command,
    );
    let conspiracy = t.custom(
        P0,
        crate::r703_common::oracle_card("Idle Conspiracy", "Conspiracy", "", None, ""),
        Zone::Command,
    );
    let phenomenon = t.custom(
        P0,
        crate::r703_common::oracle_card("Idle Phenomenon", "Phenomenon", "", None, ""),
        Zone::Command,
    );
    for id in [plane, scheme, vanguard, conspiracy, phenomenon] {
        for to in [
            Zone::Hand(P0),
            Zone::Graveyard(P0),
            Zone::Exile,
            Zone::Library(P0),
            Zone::Battlefield,
        ] {
            assert_eq!(move_to(&mut t, id, to), None, "{to:?}");
            assert!(t.is_live(id));
            assert_eq!(t.zone(id), Zone::Command);
        }
    }
    // A commander in the command zone is an ordinary card: it can leave.
    let cmdr = t.command(P0, "Grizzly Bears");
    assert!(move_to(&mut t, cmdr, Zone::Hand(P0)).is_some());
    assert_eq!(t.zone(cmdr), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------
// 400.5: order
// ---------------------------------------------------------------------------

#[test]
fn the_order_of_a_graveyard_and_the_stack_doesnt_change() {
    cr!("400.5");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let c = t.graveyard(P0, "Lightning Bolt");
    // A card leaving the middle of the pile doesn't reorder the others; a new card goes
    // on top.
    move_to(&mut t, b, Zone::Exile);
    let d = t.hand(P0, "Shock");
    let d = move_to(&mut t, d, Zone::Graveyard(P0)).unwrap();
    assert_eq!(t.g.player(P0).graveyard, vec![a, c, d]);
    // The stack: last in, first out.
    let first = t.custom(P0, free_instant("First"), Zone::Hand(P0));
    let second = t.custom(P0, free_instant("Second"), Zone::Hand(P0));
    let first = t.cast(P0, first).go();
    let second = t.cast(P0, second).go();
    assert_eq!(t.g.stack, vec![first, second]);
    t.resolve();
    assert_eq!(t.g.stack, vec![first]);
}

// ---------------------------------------------------------------------------
// 400.6: moving objects
// ---------------------------------------------------------------------------

#[test]
fn a_moving_objects_own_replacement_abilities_apply_from_any_zone() {
    cr!("400.6", "616.1");
    for name in ["Progenitus", "Blightsteel Colossus", "Rest in Peace"] {
        supported(name);
    }
    // Discarded from a hidden zone to a public one: its owner looks at it and applies its
    // ability.
    let mut t = TestGame::new(2);
    let lib_before = t.library_size(P0);
    let prog = t.hand(P0, "Progenitus");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Discard {
            who: PlayerRef::You,
            n: Value::c(1),
            random: false,
            filter: Filter::Any,
        },
        &[],
    );
    assert_eq!(t.zone(prog), Zone::Library(P0));
    assert_eq!(t.library_size(P0), lib_before + 1);
    assert_eq!(t.graveyard_size(P0), 0);
    // Milled from the library.
    let colossus = t.library_top(P0, "Blightsteel Colossus");
    t.g.mill(P0, 1);
    assert_eq!(t.zone(colossus), Zone::Library(P0));
    assert_eq!(t.graveyard_size(P0), 0);
    // Replacement effects from elsewhere apply too: with Rest in Peace, a sacrificed
    // creature is exiled.
    t.battlefield(P1, "Rest in Peace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bears, P0);
    t.settle();
    assert_eq!(t.zone(bears), Zone::Exile);
    // The Colossus's own ability and Rest in Peace both apply to its move. Neither is a
    // self-replacement effect (CR 614.15), so its controller chooses which to apply
    // (CR 616.1): either outcome is possible.
    for choice in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Rest in Peace");
        let colossus = t.battlefield(P0, "Blightsteel Colossus");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(choice));
        let asked = t.asked().len();
        t.g.sacrifice(colossus, P0);
        t.settle();
        let options = t.asked()[asked..]
            .iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } => Some((*p, options.clone())),
                _ => None,
            })
            .expect("the controller chooses a replacement effect");
        assert_eq!(options.0, P0);
        assert_eq!(options.1.len(), 2, "{:?}", options.1);
        let expected = if options.1[choice].contains("Rest in Peace") {
            Zone::Exile
        } else {
            Zone::Library(P0)
        };
        assert_eq!(t.zone(colossus), expected, "chose {:?}", options.1[choice]);
    }
}

/// A second replacement effect for cards put into graveyards, contradicting Rest in
/// Peace's.
fn undertow() -> CardDef {
    compile_def(
        "Undertow Shrine",
        "Enchantment",
        "{0}",
        "If a card would be put into a graveyard from anywhere, shuffle it into its owner's library instead.",
    )
}

#[test]
fn the_controller_or_else_the_owner_chooses_between_contradictory_effects() {
    cr!("400.6");
    let replacement_choosers = |t: &TestGame, from: usize| -> Vec<PlayerId> {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .map(|(p, _)| *p)
            .collect()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    t.custom(P1, undertow(), Zone::Battlefield);
    // A creature P1 owns but P0 controls: its controller chooses.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].base_controller = P0;
    t.g.recompute();
    let asked = t.asked().len();
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(replacement_choosers(&t, asked), vec![P0]);
    // A card in a hand has no controller: its owner chooses.
    let card_in_hand = t.hand(P1, "Hill Giant");
    let asked = t.asked().len();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: in_zone(ZoneKind::Hand, vec![card_in_hand]),
            to: Destination::zone(ZoneKind::Graveyard),
        },
        &[],
    );
    assert_eq!(replacement_choosers(&t, asked), vec![P1]);
    assert!(matches!(t.zone(card_in_hand), Zone::Exile | Zone::Library(_)));
    assert_eq!(t.graveyard_size(P1), 0);
}

// ---------------------------------------------------------------------------
// 400.11: outside the game
// ---------------------------------------------------------------------------

#[test]
fn cards_in_a_sideboard_are_outside_the_game_and_can_be_brought_in() {
    cr!("400.11", "400.11a");
    ruling!(
        "Burning Wish",
        "In a sanctioned event, a card that’s “outside the game” is one that’s in your sideboard."
    );
    supported("Burning Wish");
    let mut t = TestGame::new(2);
    let axe = sideboard(&mut t, P0, "Lava Axe");
    assert_eq!(t.zone(axe), Zone::Outside(P0));
    t.lands(P0, "Mountain", 2);
    let wish = t.hand(P0, "Burning Wish");
    t.cast(P0, wish).go();
    t.resolve();
    assert_eq!(t.zone(axe), Zone::Hand(P0));
    assert!(t.g.player(P0).sideboard.is_empty());
}

#[test]
fn cards_in_exile_or_the_ante_arent_outside_the_game() {
    cr!("400.11");
    ruling!(
        "Burning Wish",
        "You can’t acquire exiled cards because those cards are still in one of the game’s zones."
    );
    ruling!("Burning Wish", "Can’t acquire the Ante cards.");
    let mut t = TestGame::new(2);
    let exiled = t.exile(P0, "Lava Axe");
    let anted = t.custom(P0, (*card("Stone Rain")).clone(), Zone::Ante);
    t.lands(P0, "Mountain", 2);
    let wish = t.hand(P0, "Burning Wish");
    t.cast(P0, wish).go();
    t.resolve();
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.zone(anted), Zone::Ante);
    assert!(!t.in_hand(P0, "Lava Axe") && !t.in_hand(P0, "Stone Rain"));
}

#[test]
fn a_card_brought_into_the_game_stays_in_it() {
    cr!("400.11b");
    let mut t = TestGame::new(2);
    let axe = sideboard(&mut t, P0, "Lava Axe");
    t.lands(P0, "Mountain", 7);
    let wish = t.hand(P0, "Burning Wish");
    t.cast(P0, wish).go();
    t.resolve();
    // Cast, it goes to the graveyard like any other card — it doesn't go back to the
    // sideboard.
    let axe = t.g.current(axe);
    t.cast(P0, axe).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_graveyard(P0, "Lava Axe"));
    assert!(t.g.player(P0).sideboard.is_empty());
    // It remains in the game until its owner leaves it (CR 800.4a).
    let mut t = TestGame::new(3);
    let axe = sideboard(&mut t, P1, "Lava Axe");
    move_to(&mut t, axe, Zone::Hand(P1));
    assert_eq!(t.zone(axe), Zone::Hand(P1));
    t.g.player_loses(P1);
    assert!(!matches!(t.zone(axe), Zone::Hand(_)));
}

#[test]
fn cards_outside_the_game_are_affected_only_by_their_own_characteristic_defining_abilities() {
    cr!("400.11c");
    let mut t = TestGame::new(2);
    let goyf = sideboard(&mut t, P0, "Tarmogoyf");
    let bears = sideboard(&mut t, P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    // "Cards in hands are red" and "Cards outside the game are red" — from a permanent:
    // only the first has any effect.
    let red = |zone: ZoneKind| {
        stat(StaticEffect::Continuous {
            affected: Filter::and(vec![Filter::Card, Filter::InZone(zone)]),
            mods: vec![Modification::AddColors(ColorSet::single(Color::Red))],
        })
    };
    t.custom(
        P0,
        CB::new("Crimson Haze")
            .enchantment()
            .ability(red(ZoneKind::Hand))
            .ability(red(ZoneKind::Outside))
            .build(),
        Zone::Battlefield,
    );
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Hill Giant");
    t.g.recompute();
    // The card in hand is affected; the sideboard cards aren't.
    assert!(t.obj(in_hand).chars.colors.contains(Color::Red));
    assert!(!t.obj(bears).chars.colors.contains(Color::Red));
    assert!(!t.obj(goyf).chars.colors.contains(Color::Red));
    // Tarmogoyf's own characteristic-defining ability applies: instant and creature.
    assert_eq!(t.obj(goyf).chars.power, Some(2));
    assert_eq!(t.obj(goyf).chars.toughness, Some(3));
}

// ---------------------------------------------------------------------------
// 400.12: doing something to a zone
// ---------------------------------------------------------------------------

#[test]
fn an_action_on_a_zone_is_performed_on_every_card_in_it() {
    cr!("400.12");
    supported("Timetwister");
    let mut t = TestGame::new(2);
    for n in ["Grizzly Bears", "Hill Giant", "Shock"] {
        t.hand(P0, n);
    }
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Opt");
    t.hand(P1, "Forest");
    for _ in 0..4 {
        t.graveyard(P1, "Island");
    }
    t.lands(P0, "Island", 3);
    let twister = t.hand(P0, "Timetwister");
    let (lib0, lib1) = (t.library_size(P0), t.library_size(P1));
    t.cast(P0, twister).go();
    t.resolve();
    // Every card in each hand and graveyard was shuffled in (Timetwister itself goes to
    // the graveyard afterward); then each player drew seven.
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.library_size(P0), lib0 + 5 - 7);
    assert_eq!(t.library_size(P1), lib1 + 5 - 7);
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
    assert!(t.in_graveyard(P0, "Timetwister"));
    assert_eq!(t.graveyard_size(P1), 0);
}
