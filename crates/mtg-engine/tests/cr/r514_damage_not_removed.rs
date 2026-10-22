//! "Damage isn't removed from [permanents] during cleanup steps" (an exception to
//! CR 514.2; `rule_statics::cleanup_damage`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn bolt(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(target).go();
    t.resolve();
}

#[test]
fn only_creatures_your_opponents_control_keep_their_damage() {
    cr!("514.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Uthgardt Fury");
    let mine = t.battlefield(P0, "Craw Wurm");
    let theirs = t.battlefield(P1, "Craw Wurm");
    bolt(&mut t, P0, mine);
    bolt(&mut t, P0, theirs);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(mine).damage, 0);
    assert_eq!(t.obj_now(theirs).damage, 3);
    // Still there after the opponent's own cleanup step: another 3 damage kills it.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(theirs).damage, 3);
    bolt(&mut t, P0, theirs);
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn all_creatures_keep_their_damage_while_the_ability_functions() {
    cr!("514.2");
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Market Melee");
    let mine = t.battlefield(P0, "Craw Wurm");
    let theirs = t.battlefield(P1, "Craw Wurm");
    bolt(&mut t, P0, mine);
    bolt(&mut t, P0, theirs);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(mine).damage, 3);
    assert_eq!(t.obj_now(theirs).damage, 3);
    // Once the Case is gone, the next cleanup step removes the damage.
    t.g.move_object(case, Zone::Graveyard(P0), events::MoveCause::Effect, None);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(mine).damage, 0);
    assert_eq!(t.obj_now(theirs).damage, 0);
}

#[test]
fn regeneration_still_removes_the_damage() {
    cr!("514.2", "701.19a");
    ruling!("Ancient Adamantoise", "Effects that remove all damage from a permanent (such as regeneration) will still remove damage");
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P0, "Ancient Adamantoise");
    bolt(&mut t, P0, turtle);
    assert_eq!(t.obj_now(turtle).damage, 3);
    t.lands(P0, "Plains", 1);
    let ward = t.hand(P0, "Death Ward");
    t.cast(P0, ward).target(turtle).go();
    t.resolve();
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(turtle).go();
    t.resolve();
    assert!(t.on_battlefield(turtle));
    assert_eq!(t.obj_now(turtle).damage, 0);
}

#[test]
fn case_is_solved_with_three_damaged_creatures() {
    cr!("719.3a", "120.3e");
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Market Melee");
    let wurms: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
    bolt(&mut t, P0, wurms[0]);
    bolt(&mut t, P0, wurms[1]);
    t.advance_to(P1, Step::Upkeep);
    // Two damaged creatures at the end step: not solved.
    assert!(!cases::is_solved(&t.g, case));
    bolt(&mut t, P0, wurms[2]);
    t.advance_to(P0, Step::Upkeep);
    assert!(!cases::is_solved(&t.g, case));
    t.advance_to(P1, Step::Upkeep);
    assert!(cases::is_solved(&t.g, case));
}
