//! CR 406: the exile zone — exiling from any zone, face-down exiled cards and who may look
//! at them, casting them, choosing among them, and linked "exiled with" abilities.

use crate::r600_common::*;
use crate::r703_common::{run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::eval::Ctx;
use mtg_engine::events::MoveCause;
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::zones;
use mtg_engine::*;

/// The given objects, which are in a zone of this kind.
fn in_zone(zone: ZoneKind, ids: Vec<ObjectId>) -> Sel {
    Sel::All(Filter::and(vec![Filter::InZone(zone), Filter::Objects(ids)]))
}

/// Exiles `ids` face down at the same time, as an effect of `source` would.
fn exile_face_down(t: &mut TestGame, ids: &[ObjectId], source: Option<ObjectId>) -> Vec<ObjectId> {
    let moves = ids
        .iter()
        .map(|id| MoveEv {
            obj: *id,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Exile,
            by: Some(P0),
            etb: EtbInfo {
                face_down: Some(keywords::KeywordKind::Morph),
                ..Default::default()
            },
            source,
        })
        .collect();
    t.g.move_objects(moves).into_iter().flatten().collect()
}

// ---------------------------------------------------------------------------
// 406.2
// ---------------------------------------------------------------------------

#[test]
fn to_exile_is_to_put_into_exile_from_any_zone() {
    cr!("406.2");
    let mut t = TestGame::new(2);
    let from_bf = t.battlefield(P0, "Grizzly Bears");
    let from_hand = t.hand(P0, "Hill Giant");
    let from_gy = t.graveyard(P1, "Lightning Bolt");
    let from_lib = t.library_top(P1, "Forest");
    let q = t.custom(P0, free_instant("Quick"), Zone::Hand(P0));
    let from_stack = t.cast(P0, q).go();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Exile {
            what: Sel::Union(vec![
                in_zone(ZoneKind::Battlefield, vec![from_bf]),
                in_zone(ZoneKind::Hand, vec![from_hand]),
                in_zone(ZoneKind::Graveyard, vec![from_gy]),
                in_zone(ZoneKind::Library, vec![from_lib]),
                in_zone(ZoneKind::Stack, vec![from_stack]),
            ]),
            face_down: false,
            link: false,
        },
        &[],
    );
    for id in [from_bf, from_hand, from_gy, from_lib, from_stack] {
        assert_eq!(t.zone(id), Zone::Exile);
    }
    assert_eq!(t.stack_len(), 0);
    // "An exiled card": every one of them is a card in the exile zone.
    let exiled_cards = t.g.objects_matching(
        &Filter::and(vec![Filter::Card, Filter::InZone(ZoneKind::Exile)]),
        &Ctx::new(None, P0),
    );
    assert_eq!(exiled_cards.len(), 5);
}

fn free_instant(name: &str) -> CardDef {
    CB::new(name)
        .instant()
        .cost("{0}")
        .spell(Body::effect(gain(1)))
        .build()
}

// ---------------------------------------------------------------------------
// 406.3: face-down exiled cards
// ---------------------------------------------------------------------------

#[test]
fn exiled_cards_are_face_up_unless_exiled_face_down_and_then_no_one_may_look() {
    cr!("406.3");
    supported("Bomat Courier");
    let mut t = TestGame::new(2);
    // Face up by default: anyone may examine it.
    let bolt = t.exile(P1, "Lightning Bolt");
    assert!(!t.obj(bolt).face_down);
    assert!(can_look_at(&t.g, P0, bolt) && can_look_at(&t.g, P1, bolt));
    // Bomat Courier: "Whenever Bomat Courier attacks, exile the top card of your library
    // face down." Nobody may look at it — not even its owner.
    let courier = t.battlefield(P0, "Bomat Courier");
    let top = t.library_top(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(courier, Entity::Player(P1))], &[]);
    let exiled = t.g.current(top);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.obj(exiled).face_down);
    assert!(!can_look_at(&t.g, P0, exiled));
    assert!(!can_look_at(&t.g, P1, exiled));
}

#[test]
fn a_player_who_looked_at_a_card_and_exiled_it_face_down_may_keep_looking_at_it() {
    cr!("406.3");
    ruling!(
        "Kayla's Music Box",
        "No other player may look at the face-down cards you own exiled with Kayla’s Music Box, even if another player takes control of it."
    );
    supported("Kayla's Music Box");
    let mut t = TestGame::new(2);
    let music_box = t.battlefield(P0, "Kayla's Music Box");
    let top = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    // "{W}, {T}: Look at the top card of your library, then exile it face down."
    t.activate(P0, music_box, 0, &[]).unwrap();
    t.resolve();
    let exiled = t.g.current(top);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.obj(exiled).face_down);
    assert!(can_look_at(&t.g, P0, exiled));
    assert!(!can_look_at(&t.g, P1, exiled));
    // Even once the instruction no longer applies (the Music Box is gone, or another
    // player controls it), P0 may still look; P1 still may not.
    t.g.objects[music_box.0 as usize].base_controller = P1;
    t.g.recompute();
    assert!(can_look_at(&t.g, P0, exiled));
    assert!(!can_look_at(&t.g, P1, exiled));
    t.g.move_object(music_box, Zone::Graveyard(P0), MoveCause::Effect, None);
    assert!(can_look_at(&t.g, P0, exiled));
    // Until it leaves exile: then it's a new object in its new zone, and the permission
    // doesn't carry over if it's exiled face down again.
    let hand = t
        .g
        .move_object(exiled, Zone::Hand(P0), MoveCause::Effect, None)
        .unwrap();
    let again = exile_face_down(&mut t, &[hand], None)[0];
    assert!(!zones::may_look(&t.g, P0, again));
    assert!(!can_look_at(&t.g, P0, again));
}

#[test]
fn a_player_allowed_to_look_at_a_foretold_card_may_look_at_it_while_it_remains_exiled() {
    cr!("406.3", "702.143a");
    supported("Demon Bolt");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Demon Bolt");
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.take_action(
        P0,
        Action::Special(mtg_engine::decision::SpecialAction::Foretell { card: bolt }),
    );
    let foretold = t.g.current(bolt);
    assert_eq!(t.zone(foretold), Zone::Exile);
    assert!(t.obj(foretold).face_down);
    // "That player may look at that card as long as it remains in exile"; nobody else may.
    assert!(can_look_at(&t.g, P0, foretold));
    assert!(!can_look_at(&t.g, P1, foretold));
    // A later turn: still.
    t.advance_to(P1, Step::Upkeep);
    assert!(can_look_at(&t.g, P0, foretold));
    assert!(!can_look_at(&t.g, P1, foretold));
}

#[test]
fn a_face_down_exiled_card_has_no_characteristics_until_turned_face_up_to_be_played() {
    cr!("406.3a");
    let mut t = TestGame::new(2);
    let music_box = t.battlefield(P0, "Kayla's Music Box");
    let top = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.activate(P0, music_box, 0, &[]).unwrap();
    t.resolve();
    let exiled = t.g.current(top);
    let c = &t.obj(exiled).chars;
    assert_eq!(c.name, "");
    assert!(c.card_types.is_empty());
    assert!(c.mana_cost.is_none());
    assert_eq!(c.colors, ColorSet::NONE);
    let ctx = Ctx::new(None, P0);
    assert!(!t.g.matches(exiled, &Filter::creature(), &ctx));
    // "{T}: Until end of turn, you may play cards you own exiled with Kayla's Music Box."
    t.g.players[P0.idx()].lands_played_this_turn = 0;
    t.g.objects[music_box.0 as usize].tapped = false;
    t.activate(P0, music_box, 1, &[]).unwrap();
    t.resolve();
    t.lands(P0, "Forest", 2);
    // It's turned face up as it's cast: the spell is a Grizzly Bears.
    let spell = t.cast(P0, exiled).go();
    assert!(!t.obj(spell).face_down);
    assert_eq!(t.obj(spell).chars.name, "Grizzly Bears");
    assert!(t.obj(spell).chars.is(CardType::Creature));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

/// "You may cast creature spells from among cards in exile."
fn creature_caster() -> CardDef {
    CB::new("Exile Scholar")
        .creature(1, 1)
        .ability(stat(StaticEffect::PlayPermission(PlayPermission {
            who: PlayerRel::You,
            zone: ZoneKind::Exile,
            top_only: false,
            what: Filter::creature(),
            lands: false,
            spells: true,
            cost: None,
        })))
        .build()
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Cast { card: c, method: CastMethod::Normal } if *c == card)
    })
}

#[test]
fn casting_a_spell_with_qualities_from_face_down_cards_requires_looking_at_them() {
    cr!("406.3b");
    let mut t = TestGame::new(2);
    t.custom(P0, creature_caster(), Zone::Battlefield);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    let exiled = exile_face_down(&mut t, &[bears, bolt], None);
    let (bears, bolt) = (exiled[0], exiled[1]);
    // P0 isn't allowed to look at them: they can't be cast.
    assert!(!can_cast(&mut t, P0, bears));
    // Once P0 may look at them, the one whose spell would have the stated quality can be.
    zones::allow_look(&mut t.g, P0, bears);
    zones::allow_look(&mut t.g, P0, bolt);
    assert!(can_cast(&mut t, P0, bears));
    assert!(!can_cast(&mut t, P0, bolt));
    let spell = t.cast(P0, bears).go();
    assert_eq!(t.obj(spell).chars.name, "Grizzly Bears");
}

#[test]
fn a_player_who_cant_look_at_face_down_exiled_cards_chooses_a_pile_and_gets_a_random_card() {
    cr!("406.4");
    // "Put a card exiled with this into its owner's hand", chosen by a player.
    let choose_one = |chooser: PlayerRef| Effect::Move {
        what: Sel::Choose {
            chooser,
            filter: Filter::InZone(ZoneKind::Exile),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    };
    let mut returned = std::collections::BTreeSet::new();
    for seed in 0..12u64 {
        let mut t = TestGame::with_config(
            2,
            game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let cards: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant", "Craw Wurm"]
            .iter()
            .map(|n| t.hand(P1, n))
            .collect();
        // Two piles: two cards exiled together, then one more.
        let first = exile_face_down(&mut t, &cards[..2], None);
        let lone = exile_face_down(&mut t, &cards[2..], None);
        // P0 can't look at them: the choices offered are one per pile.
        t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Object(first[0])]));
        let asked = t.asked().len();
        run_effect(&mut t, P0, None, choose_one(PlayerRef::You), &[]);
        let offered = t.asked()[asked..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
                _ => None,
            })
            .unwrap();
        assert_eq!(offered, 2, "a pile of two and a pile of one");
        // A card from the chosen pile was chosen at random.
        let in_hand: Vec<String> = t
            .g
            .player(P1)
            .hand
            .iter()
            .map(|id| t.obj(*id).chars.name.to_string())
            .collect();
        assert_eq!(in_hand.len(), 1);
        assert_ne!(in_hand[0], "Craw Wurm");
        returned.insert(in_hand[0].clone());
        assert_eq!(t.zone(lone[0]), Zone::Exile);
    }
    assert_eq!(returned.len(), 2, "both cards of the pile come up");
    // A player who may look at the cards chooses a specific one.
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    let exiled = exile_face_down(&mut t, &[a, b], None);
    for e in &exiled {
        zones::allow_look(&mut t.g, P0, *e);
    }
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Object(exiled[1])]));
    let asked = t.asked().len();
    run_effect(&mut t, P0, None, choose_one(PlayerRef::You), &[]);
    let offered = t.asked()[asked..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, 2);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// 406.5–406.6: keeping track of exiled cards; linked abilities
// ---------------------------------------------------------------------------

#[test]
fn cards_exiled_by_different_objects_are_kept_track_of_separately() {
    cr!("406.5");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 6);
    let p1 = t.hand(P0, "Banisher Priest");
    t.cast(P0, p1).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    let p2 = t.hand(P0, "Banisher Priest");
    t.cast(P0, p2).go();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    // The first Priest leaves: only the card it exiled returns.
    let first = t.g.current(p1);
    t.g.destroy(first, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn an_ability_referring_to_cards_exiled_with_an_object_is_linked_to_its_exiling_ability() {
    cr!("406.6");
    ruling!(
        "Bomat Courier",
        "Bomat Courier's last ability only puts those cards into your hand, not those of any other Bomat Courier."
    );
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Bomat Courier");
    let other = t.battlefield(P0, "Bomat Courier");
    t.library_top(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(courier, Entity::Player(P1))], &[]);
    t.library_top(P0, "Hill Giant");
    t.g.turn.number += 1;
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(other, Entity::Player(P1))], &[]);
    // A card exiled some other way.
    t.exile(P0, "Lightning Bolt");
    // "{R}, Discard your hand, Sacrifice Bomat Courier: Put all cards exiled with Bomat
    // Courier into their owners' hands."
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, courier, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert!(t.in_exile("Lightning Bolt"));
}
