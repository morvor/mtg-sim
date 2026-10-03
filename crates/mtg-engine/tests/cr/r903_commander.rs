//! CR 903.1–903.3, 903.6–903.11: the Commander casual variant in play — setting up the
//! game, commanders (including melded and merged ones and "your commander"), returning
//! to the command zone, commander damage, and cards from outside the game.

use crate::r100_common::{fillers, pregame};
use crate::r703_common::{oracle_card, run_effect, supported};
use crate::r900_common::name_of;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::merge;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// A Commander game (already under way) of `n` players.
fn commander_game(n: usize) -> TestGame {
    TestGame::with_config(n, GameConfig::commander_game())
}

/// Makes `id` a commander (the designation of its card, CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.obj(id).owner;
    let name = t.obj(id).card.as_ref().unwrap().name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
}

/// A 100-card Commander deck: `commander` and 99 basic lands.
fn deck_of(commander: &str, land: &str) -> Vec<Arc<CardDef>> {
    std::iter::once(card(commander))
        .chain((0..99).map(|_| card(land)))
        .collect()
}

#[test]
fn a_commander_game_is_led_by_commanders_with_the_normal_rules() {
    cr!("903.1", "903.6", "903.7");
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            starting_player: Some(P0),
            ..GameConfig::commander_game()
        },
        vec![
            deck_of("Isamaru, Hound of Konda", "Plains"),
            deck_of("Niv-Mizzet, Parun", "Island"),
        ],
    );
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    assert!(t.g.designate_commander(P1, "Niv-Mizzet, Parun"));
    t.g.start();
    // Each commander starts face up in the command zone; the other 99 cards are the
    // library (shuffled) from which each player drew seven cards; life totals are 40.
    for (p, name) in [(P0, "Isamaru, Hound of Konda"), (P1, "Niv-Mizzet, Parun")] {
        let ids = t.g.find_in_zone(Zone::Command, name);
        assert_eq!(ids.len(), 1);
        assert!(!t.obj(ids[0]).face_down);
        assert_eq!(t.obj(ids[0]).owner, p);
        assert_eq!(t.hand_size(p), 7);
        assert_eq!(t.library_size(p), 92);
        assert_eq!(t.life(p), 40);
    }
    // The normal rules still apply: the game goes on as usual (the second player draws).
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), 8);
}

#[test]
fn commander_is_two_player_or_free_for_all() {
    cr!("903.2");
    let c = GameConfig::commander_game();
    assert_eq!(c.validate(2), Ok(()));
    assert_eq!(c.validate(4), Ok(()));
    assert!(c.attack_multiple_players);
    assert_eq!(c.range_of_influence, None);
    let t = commander_game(4);
    for q in [P1, P2, P3] {
        assert!(t.g.are_opponents(P0, q));
    }
    assert_eq!(t.g.range_of_influence(P0), None);
}

#[test]
fn a_melded_commander_is_the_players_commander() {
    cr!("903.3b");
    supported("Gisela, the Broken Blade");
    let mut t = commander_game(2);
    let bruna = t.battlefield(P0, "Bruna, the Fading Light");
    make_commander(&mut t, bruna);
    let gisela = t.battlefield(P0, "Gisela, the Broken Blade");
    let a = t.g.exile_object(bruna, None).unwrap();
    let b = t.g.exile_object(gisela, None).unwrap();
    let brisela = merge::meld(&mut t.g, a, b, card("Brisela, Voice of Nightmares"), P0)
        .expect("melded");
    t.g.recompute();
    assert_eq!(t.obj(brisela).chars.name, "Brisela, Voice of Nightmares");
    assert!(t.obj(brisela).is_commander);
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(brisela, &Filter::Commander, &ctx));
    // Its combat damage is commander damage from Bruna.
    t.g.objects[brisela.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(brisela, Entity::Player(P1))], &[]);
    assert_eq!(
        t.player(P1).commander_damage.get("Bruna, the Fading Light"),
        Some(&9)
    );
    // When it dies, only Bruna is a commander in the graveyard.
    t.answer_yes(P0, false);
    t.g.destroy(brisela, None);
    t.settle();
    let gy: Vec<ObjectId> = t.g.player(P0).graveyard.clone();
    assert_eq!(gy.len(), 2);
    for id in gy {
        assert_eq!(
            t.obj(id).is_commander,
            name_of(&t, id) == "Bruna, the Fading Light"
        );
    }
}

/// A legendary creature card with mutate {G}.
fn apex() -> CardDef {
    oracle_card(
        "Test Apex",
        "Legendary Creature — Beast",
        "{2}{G}",
        Some((4, 4)),
        "Mutate {G}",
    )
}

/// Casts P0's commander Test Apex from the command zone for its mutate cost onto `target`,
/// on top.
fn mutate_commander_onto(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let apex = t.custom(P0, apex(), Zone::Command);
    make_commander(t, apex);
    t.lands(P0, "Forest", 1);
    t.cast(P0, apex)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(target)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
    apex
}

#[test]
fn a_merged_permanent_with_a_commander_component_is_the_commander() {
    cr!("903.3c");
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj(bears).is_commander);
    mutate_commander_onto(&mut t, bears);
    // The Bears merged with the commander: the merged permanent is the commander.
    assert_eq!(merge::physical_components(&t.g, bears).len(), 2);
    assert_eq!(t.obj(bears).chars.name, "Test Apex");
    assert!(t.obj(bears).is_commander);
    // Merged under a noncommander on top, it's still the commander: its combat damage
    // counts for the commander card.
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apex_card = t.custom(P0, apex(), Zone::Command);
    make_commander(&mut t, apex_card);
    t.lands(P0, "Forest", 1);
    t.cast(P0, apex_card)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    assert_eq!(t.obj(bears).chars.name, "Grizzly Bears");
    assert!(t.obj(bears).is_commander);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.player(P1).commander_damage.get("Test Apex"), Some(&2));
}

#[test]
fn your_commanders_characteristics_are_seen_in_any_zone() {
    cr!("903.3e", "904.13a");
    supported("You Will Know True Suffering");
    // Archenemy Commander: P0 is the archenemy; their commander Uril, the Miststalker
    // (mana value 5) is in their library.
    let mut t = TestGame::with_config(3, GameConfig::archenemy_commander(vec![0, 1, 1]));
    let uril = t.library_top(P0, "Uril, the Miststalker");
    make_commander(&mut t, uril);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dreadmaw = t.battlefield(P2, "Colossal Dreadmaw");
    let their_cmdr = t.battlefield(P1, "Isamaru, Hound of Konda");
    make_commander(&mut t, their_cmdr);
    let scheme = t.custom(P0, (*card("You Will Know True Suffering")).clone(), Zone::Command);
    t.g.objects[scheme.0 as usize].face_down = true;
    t.g.recompute();
    // "When you set this scheme in motion, it deals damage equal to your commander's mana
    // value to each noncommander creature your opponents control."
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(dreadmaw));
    assert_eq!(t.obj(dreadmaw).damage, 5);
    assert!(t.on_battlefield(their_cmdr));
    assert_eq!(t.obj(their_cmdr).damage, 0);
}

#[test]
fn a_commander_may_return_to_the_command_zone() {
    cr!("903.9");
    let mut t = commander_game(2);
    let cmdr = t.battlefield(P0, "Isamaru, Hound of Konda");
    make_commander(&mut t, cmdr);
    // From the graveyard (a state-based action) ...
    t.answer_yes(P0, true);
    t.g.destroy(cmdr, None);
    t.settle();
    let now = t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda");
    assert_eq!(now.len(), 1);
    // ... and instead of going to a hand (a replacement effect).
    let again = t.g.move_object(now[0], Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0)).unwrap();
    t.answer_yes(P0, true);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(again)],
    );
    assert_eq!(t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda").len(), 1);
    assert!(!t.in_hand(P0, "Isamaru, Hound of Konda"));
}

#[test]
fn a_commander_newly_in_a_graveyard_or_exile_may_go_to_the_command_zone() {
    cr!("903.9a");
    let mut t = commander_game(2);
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    make_commander(&mut t, a);
    t.answer_yes(P0, true);
    t.g.exile_object(a, None);
    // It's a state-based action: nothing happens until they're checked.
    assert!(t.in_exile("Isamaru, Hound of Konda"));
    t.settle();
    assert!(!t.in_exile("Isamaru, Hound of Konda"));
    assert_eq!(t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda").len(), 1);
    // A commander that was already in the graveyard when state-based actions were last
    // checked stays there.
    let b = t.graveyard(P1, "Niv-Mizzet, Parun");
    make_commander(&mut t, b);
    t.settle();
    assert!(t.in_graveyard(P1, "Niv-Mizzet, Parun"));
    // Not a commander: a card of the same kind stays in the graveyard.
    let c = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(c, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_commander_going_to_a_hand_or_library_may_go_to_the_command_zone_instead() {
    cr!("903.9b");
    // To its owner's hand.
    let mut t = commander_game(2);
    let cmdr = t.battlefield(P0, "Isamaru, Hound of Konda");
    make_commander(&mut t, cmdr);
    t.answer_yes(P0, true);
    let unsummon = t.hand(P1, "Unsummon");
    t.lands(P1, "Island", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, unsummon).target(cmdr).go();
    t.resolve();
    assert_eq!(t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda").len(), 1);
    assert!(!t.in_hand(P0, "Isamaru, Hound of Konda"));
    // Into its owner's library (Chaos Warp shuffles it in): the owner may choose not to.
    let mut t = commander_game(2);
    let cmdr = t.battlefield(P0, "Isamaru, Hound of Konda");
    make_commander(&mut t, cmdr);
    let lib = t.library_size(P0);
    t.answer_yes(P0, false);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Library),
        },
        &[Entity::Object(cmdr)],
    );
    assert_eq!(t.library_size(P0), lib + 1);
    // Not a commander: no such choice.
    let mut t = commander_game(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(bears)],
    );
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn a_merged_commander_splits_up_as_it_returns_to_the_command_zone() {
    cr!("903.9c");
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_commander_onto(&mut t, bears);
    assert!(t.obj(bears).is_commander);
    // The merged commander would return to its owner's hand; P0 puts it into the command
    // zone instead: the commander card goes there, the Bears card to the hand.
    t.answer_yes(P0, true);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(bears)],
    );
    let in_command = t.g.find_in_zone(Zone::Command, "Test Apex");
    assert_eq!(in_command.len(), 1);
    assert!(t.obj(in_command[0]).is_commander);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Test Apex"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn the_commander_variant_adds_a_way_to_lose_to_the_normal_ones() {
    cr!("903.10");
    let mut t = commander_game(3);
    // The normal rules for ending the game apply: 0 life loses.
    t.g.players[1].life = 1;
    t.g.lose_life(P1, 1);
    t.settle();
    assert!(t.has_lost(P1));
    // And so does 21 combat damage from one commander.
    let cmdr = t.battlefield(P0, "Hill Giant");
    make_commander(&mut t, cmdr);
    t.g.players[2].commander_damage.insert("Hill Giant".into(), 21);
    t.settle();
    assert!(t.has_lost(P2));
}

#[test]
fn twenty_one_combat_damage_from_the_same_commander() {
    cr!("903.10a");
    let mut t = commander_game(3);
    let a = t.battlefield(P0, "Colossal Dreadmaw");
    make_commander(&mut t, a);
    let b = t.battlefield(P2, "Colossal Dreadmaw");
    let _ = b;
    let giant = t.battlefield(P2, "Hill Giant");
    make_commander(&mut t, giant);
    // 18 from P0's commander, 12 from P2's: 30 in all, but not 21 from the same one.
    t.g.players[1].commander_damage.insert("Colossal Dreadmaw".into(), 18);
    t.g.players[1].commander_damage.insert("Hill Giant".into(), 12);
    t.settle();
    assert!(!t.has_lost(P1));
    // Noncombat damage from the commander doesn't count.
    t.g.deal_damage(a, Entity::Player(P1), 6, false);
    t.settle();
    assert!(!t.has_lost(P1));
    assert_eq!(t.player(P1).commander_damage.get("Colossal Dreadmaw"), Some(&18));
    // Three more combat damage from it: 21 or more.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert!(t.has_lost(P1));
    assert!(t.life(P1) > 0);
}

#[test]
fn cards_from_outside_the_game_cant_be_brought_into_a_commander_game() {
    cr!("903.11");
    supported("Burning Wish");
    // In a normal game, Burning Wish finds a sorcery P0 owns outside the game.
    let mut t = TestGame::new(2);
    let side = t.g.add_to_sideboard(P0, vec![card("Lava Axe")]);
    let wish = t.hand(P0, "Burning Wish");
    t.lands(P0, "Mountain", 2);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.cast(P0, wish).go();
    t.resolve();
    assert!(t.in_hand(P0, "Lava Axe"));
    // In a Commander game it can't.
    let mut t = commander_game(2);
    let side = t.g.add_to_sideboard(P0, vec![card("Lava Axe")]);
    let wish = t.hand(P0, "Burning Wish");
    t.lands(P0, "Mountain", 2);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.cast(P0, wish).go();
    t.resolve();
    assert!(!t.in_hand(P0, "Lava Axe"));
    assert_eq!(t.zone(side[0]), Zone::Outside(P0));
}

fn companion_action(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).iter().any(
        |a| matches!(a, Action::Special(SpecialAction::CompanionToHand { card: c }) if *c == card),
    )
}

#[test]
fn a_card_brought_in_from_outside_must_fit_the_deck() {
    cr!("903.11a");
    // A special action may bring a companion in from outside the game (CR 702.139a) —
    // but only one within the commander's color identity, whose name isn't among the
    // starting deck's cards or the cards the player owns in the game.
    let setup = |commander: &str, companion: &str, deck_card: Option<&str>| {
        let mut t = commander_game(2);
        let c = t.command(P0, commander);
        make_commander(&mut t, c);
        let side = t.g.add_to_sideboard(P0, vec![card(companion)]);
        if let Some(n) = deck_card {
            let id = t.library_top(P0, n);
            t.g.start.starting_decks.insert(P0, vec![id]);
        }
        assert!(mtg_engine::kw::companion::choose_companion(&mut t.g, P0, side[0]));
        t.lands(P0, "Wastes", 3);
        t.set_step(P0, Step::PrecombatMain);
        (t, side[0])
    };
    // Kaheera (green-white) with a green-white commander (Wilt-Leaf Liege's color
    // identity): yes.
    let (mut t, k) = setup("Wilt-Leaf Liege", "Kaheera, the Orphanguard", None);
    assert!(companion_action(&mut t, P0, k));
    t.g.perform_action(P0, Action::Special(SpecialAction::CompanionToHand { card: k }))
        .unwrap();
    assert!(t.in_hand(P0, "Kaheera, the Orphanguard"));
    // With a mono-white commander: green isn't in its color identity.
    let (mut t, k) = setup("Isamaru, Hound of Konda", "Kaheera, the Orphanguard", None);
    assert!(!companion_action(&mut t, P0, k));
    // The same name as a card in the starting deck: no.
    let (mut t3, k3) = setup(
        "Wilt-Leaf Liege",
        "Kaheera, the Orphanguard",
        Some("Kaheera, the Orphanguard"),
    );
    assert!(!companion_action(&mut t3, P0, k3));
    // The same name as a card the player owns in the game: no.
    let (mut t4, k4) = setup("Wilt-Leaf Liege", "Kaheera, the Orphanguard", None);
    t4.battlefield(P0, "Kaheera, the Orphanguard");
    assert!(!companion_action(&mut t4, P0, k4));
}
