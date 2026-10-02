//! The activation restriction grammar (CR 602.2, 602.5b, 602.5d, 602.5e): "Activate no
//! more than twice each turn", "Activate only once", "Only your opponents may activate
//! this ability and only as a sorcery", "Any player may activate this ability but only
//! during their turn", "Activate only during any upkeep step", "You can't activate this
//! ability during combat", and lists of clauses ("only during the declare blockers step,
//! only if ..., and only once each turn").

use mtg_engine::decision::{Action, Answer};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Whether `p` may activate the `index`th activated ability of `src` now.
fn can(t: &mut TestGame, p: PlayerId, src: ObjectId, index: usize) -> bool {
    t.g.recompute();
    let uid = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .nth(index)
        .map(|a| a.uid)
        .expect("no such ability");
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok = t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Activate { source, ability, .. } if *source == src && *ability == uid)
    });
    t.g.turn.priority = saved;
    ok
}

#[test]
fn no_more_than_twice_each_turn() {
    cr!("602.5b");
    compiles("Phyrexian Battleflies");
    compiles("Sewer Rats");
    let mut t = TestGame::new(2);
    let flies = t.battlefield(P0, "Phyrexian Battleflies");
    t.lands(P0, "Swamp", 3);
    for _ in 0..2 {
        assert!(can(&mut t, P0, flies, 0));
        t.activate(P0, flies, 0, &[]).unwrap();
        t.resolve();
    }
    assert_eq!(t.pt(flies), (2, 1));
    // A third activation this turn isn't allowed, though there's mana for it.
    assert!(!can(&mut t, P0, flies, 0));
    assert!(t.activate(P0, flies, 0, &[]).is_err());
    // Next turn, it can be activated again.
    t.advance_to(P1, Step::Upkeep);
    assert!(can(&mut t, P0, flies, 0));
}

#[test]
fn activate_only_once_counts_over_the_objects_existence() {
    cr!("602.5b", "400.7");
    ruling!("Haunted Screen", "it's a new object with no memory");
    compiles("Haunted Screen");
    let mut t = TestGame::new(2);
    let screen = t.battlefield(P0, "Haunted Screen");
    t.lands(P0, "Plains", 14);
    // The third activated ability: "{7}: Put seven +1/+1 counters ... Activate only once."
    assert!(can(&mut t, P0, screen, 2));
    t.activate(P0, screen, 2, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(screen, "+1/+1"), 7);
    assert!(!can(&mut t, P0, screen, 2));
    // Not on a later turn either.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!can(&mut t, P0, screen, 2));
    // A new object may activate it again (CR 400.7).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 14);
    let screen = t.battlefield(P0, "Haunted Screen");
    t.activate(P0, screen, 2, &[]).unwrap();
    t.resolve();
    let card = t
        .g
        .move_object(screen, mtg_engine::object::Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    let screen = t
        .g
        .move_object(card, mtg_engine::object::Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    assert!(can(&mut t, P0, screen, 2));
}

#[test]
fn only_your_opponents_may_activate_and_only_as_a_sorcery() {
    cr!("602.2", "602.5d");
    ruling!("Oft-Nabbed Goat", "Only the opponents of");
    compiles("Detention Vortex");
    let mut t = TestGame::new(2);
    let goat = t.battlefield(P0, "Oft-Nabbed Goat");
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Swamp", 1);
    // Its controller can't, even at sorcery timing.
    assert!(!can(&mut t, P0, goat, 0));
    assert!(t.activate(P0, goat, 0, &[]).is_err());
    // The opponent can't during P0's turn (not sorcery timing for them).
    assert!(!can(&mut t, P1, goat, 0));
    // During the opponent's main phase with an empty stack, they can.
    t.set_step(P1, Step::PrecombatMain);
    assert!(can(&mut t, P1, goat, 0));
    let hand = t.hand_size(P1);
    t.activate(P1, goat, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.g.obj(goat).controller, P1);
    assert_eq!(t.counters(goat, "-1/-1"), 1);
    // Now P1 controls it, so only P0 (P1's opponent) may activate it.
    assert!(!can(&mut t, P1, goat, 0));
}

#[test]
fn any_player_may_activate_but_only_during_their_turn() {
    cr!("602.2");
    compiles("Volrath's Dungeon");
    compiles("Scandalmonger");
    let mut t = TestGame::new(2);
    let dungeon = t.battlefield(P0, "Volrath's Dungeon");
    // "Pay 5 life: Destroy ~": P1 can't during P0's turn, P0 can.
    assert!(can(&mut t, P0, dungeon, 0));
    assert!(!can(&mut t, P1, dungeon, 0));
    t.set_step(P1, Step::Upkeep);
    assert!(can(&mut t, P1, dungeon, 0));
    assert!(!can(&mut t, P0, dungeon, 0));
    t.activate(P1, dungeon, 0, &[]).unwrap();
    assert_eq!(t.life(P1), 15);
    t.resolve();
    assert!(!t.on_battlefield(dungeon));
    // "Any player may activate this ability but only as a sorcery."
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Scandalmonger");
    t.lands(P1, "Swamp", 2);
    t.hand(P0, "Grizzly Bears");
    assert!(!can(&mut t, P1, monger, 0));
    t.set_step(P1, Step::PrecombatMain);
    assert!(can(&mut t, P1, monger, 0));
    t.activate(P1, monger, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn any_player_during_any_upkeep_and_during_their_draw_step() {
    cr!("602.2", "503.1", "504.1");
    ruling!("Well of Knowledge", "as many times as they choose");
    compiles("Infinite Hourglass");
    compiles("Well of Knowledge");
    let mut t = TestGame::new(2);
    let glass = t.battlefield(P0, "Infinite Hourglass");
    t.g.add_counters(Entity::Object(glass), "time", 2, None);
    t.lands(P1, "Island", 3);
    assert!(!can(&mut t, P1, glass, 0));
    t.set_step(P0, Step::Upkeep);
    assert!(can(&mut t, P1, glass, 0));
    t.activate(P1, glass, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(glass, "time"), 1);
    let mut t = TestGame::new(2);
    let well = t.battlefield(P0, "Well of Knowledge");
    t.lands(P1, "Island", 4);
    t.set_step(P0, Step::Draw);
    // P0's draw step: only P0 may (it's their draw step).
    assert!(!can(&mut t, P1, well, 0));
    t.set_step(P1, Step::Draw);
    assert!(can(&mut t, P1, well, 0));
    let hand = t.hand_size(P1);
    for _ in 0..2 {
        t.activate(P1, well, 0, &[]).unwrap();
        t.resolve();
    }
    assert_eq!(t.hand_size(P1), hand + 2);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can(&mut t, P1, well, 0));
}

#[test]
fn activate_only_during_an_opponents_upkeep() {
    cr!("602.5b");
    compiles("Trade Caravan");
    compiles("Dwarven Armory");
    let mut t = TestGame::new(2);
    let caravan = t.battlefield(P0, "Trade Caravan");
    t.g.add_counters(Entity::Object(caravan), "currency", 2, None);
    let forest = t.lands(P0, "Forest", 1)[0];
    t.g.tap(forest);
    t.set_step(P0, Step::Upkeep);
    assert!(!can(&mut t, P0, caravan, 0));
    t.set_step(P1, Step::Upkeep);
    assert!(can(&mut t, P0, caravan, 0));
    t.activate(P0, caravan, 0, &[Entity::Object(forest)]).unwrap();
    t.resolve();
    assert!(!t.obj_now(forest).tapped);
    t.set_step(P1, Step::Draw);
    assert!(!can(&mut t, P0, caravan, 0));
}

#[test]
fn you_cant_activate_this_ability_during_combat() {
    cr!("602.5");
    compiles("Djinn of Infinite Deceits");
    let mut t = TestGame::new(2);
    let djinn = t.battlefield(P0, "Djinn of Infinite Deceits");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    assert!(can(&mut t, P0, djinn, 0));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can(&mut t, P0, djinn, 0));
    t.set_step(P0, Step::PostcombatMain);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, djinn, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.g.obj(b).controller, P0);
    assert_eq!(t.g.obj(a).controller, P1);
}

#[test]
fn a_list_of_clauses_all_apply() {
    cr!("602.5b", "509.1");
    ruling!("Grizzled Wolverine", "It can only get +2/+0");
    compiles("Grizzled Wolverine");
    let mut t = TestGame::new(2);
    let wolverine = t.battlefield(P0, "Grizzled Wolverine");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    // Not in the main phase.
    assert!(!can(&mut t, P0, wolverine, 0));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(wolverine, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bear, wolverine)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(can(&mut t, P0, wolverine, 0));
    t.activate(P0, wolverine, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(wolverine), (4, 2));
    // Only once each turn.
    assert!(!can(&mut t, P0, wolverine, 0));

    // Unblocked: no creature is blocking it.
    let mut t = TestGame::new(2);
    let wolverine = t.battlefield(P0, "Grizzled Wolverine");
    t.lands(P0, "Mountain", 1);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(wolverine, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(!can(&mut t, P0, wolverine, 0));
}

#[test]
fn activate_only_if_and_only_once() {
    cr!("602.5b", "702.2");
    compiles("Thought Shucker");
    compiles("In the Trenches");
    let mut t = TestGame::new(2);
    let shucker = t.battlefield(P0, "Thought Shucker");
    t.lands(P0, "Island", 4);
    assert!(!can(&mut t, P0, shucker, 0));
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    assert!(can(&mut t, P0, shucker, 0));
    t.activate(P0, shucker, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(shucker, "+1/+1"), 1);
    assert!(!can(&mut t, P0, shucker, 0));
}

#[test]
fn equip_activate_only_once_each_turn() {
    cr!("602.5b", "702.6a");
    compiles("Leather Armor");
    compiles("Shredder's Armor");
    let mut t = TestGame::new(2);
    let armor = t.battlefield(P0, "Leather Armor");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.activate(P0, armor, 0, &[Entity::Object(a)]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(armor).attached_to, Some(Entity::Object(a)));
    assert!(!can(&mut t, P0, armor, 0));
    assert!(t.activate(P0, armor, 0, &[Entity::Object(b)]).is_err());
}
