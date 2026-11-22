//! Peace Talks (hand-written, `src/cards/peace_talks.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn peace(t: &mut TestGame) {
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Peace Talks");
    t.cast(P0, s).go();
    t.resolve();
}

#[test]
fn no_targeting_players_or_permanents_this_turn() {
    cr!("115.4", "601.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    peace(&mut t);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(bears).try_go().is_err());
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn no_attacks_this_turn_and_next_turn_then_normal() {
    cr!("508.1c");
    ruling!("Peace Talks", "It affects the current turn and the next turn.");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    peace(&mut t);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    // The turn after next is unaffected.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn activated_abilities_cant_target() {
    cr!("115.4", "602.2b");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    peace(&mut t);
    // "{T}: deals 1 damage to any target": no player or permanent can be chosen.
    assert!(t.activate(P0, sorcerer, 0, &[P1.into()]).is_err());
    assert!(t.activate(P0, sorcerer, 0, &[bears.into()]).is_err());
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn triggered_abilities_can_still_target() {
    cr!("115.4", "603.3d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    peace(&mut t);
    t.answer_targets(P0, &[bears.into()]);
    t.enter(P0, "Flametongue Kavu");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn the_turn_after_next_allows_targeting_again() {
    cr!("115.4");
    let mut t = TestGame::new(2);
    peace(&mut t);
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let b1 = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, b1).target(P0).try_go().is_err());
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let b2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b2).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}
