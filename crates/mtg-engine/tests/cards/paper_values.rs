//! Values and conditions of older Oracle wordings (cards4-paper-effects, family
//! `values`): "no more scream counters", two rounded halves, a leading condition on the
//! spell's target, "You may pay {2} for each wage counter on it".

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn all_hallows_eve_returns_creatures_after_the_last_scream_counter() {
    cr!("122.1", "603.4");
    ruling!(
        "All Hallow's Eve",
        "Each creature returned to the battlefield by All Hallow's Eve enters under its owner's control."
    );
    supported("All Hallow's Eve");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let eve = t.hand(P0, "All Hallow's Eve");
    t.lands(P0, "Swamp", 4);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Island");
    t.cast(P0, eve).go();
    t.resolve();
    assert!(t.in_exile("All Hallow's Eve"));
    // P0's next upkeep: one counter removed, one left.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("All Hallow's Eve"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The following upkeep: no more counters.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "All Hallow's Eve"));
    let bears = t.named_on_battlefield("Grizzly Bears");
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!((bears.len(), giant.len()), (1, 1));
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert_eq!(t.obj_now(giant[0]).controller, P1);
    // Only creature cards.
    assert!(t.in_graveyard(P1, "Island"));
}

#[test]
fn banshee_splits_x_rounding_down_to_the_target_and_up_to_you() {
    cr!("107.1a", "120.3");
    supported("Banshee");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.battlefield(P0, "Banshee");
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, b, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn banshee_deals_no_damage_to_you_if_the_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Banshee",
        "If the targeted permanent or player is an illegal target by the time Banshee’s ability resolves, the ability doesn’t resolve. You won’t be dealt damage."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.battlefield(P0, "Banshee");
    t.lands(P0, "Swamp", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.activate(P0, b, 0, &[Entity::Object(bears)]).unwrap();
    t.g.exile_object(bears, None);
    t.resolve();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn blood_lust_condition_on_its_target() {
    cr!("601.2c", "608.2h");
    supported("Blood Lust");
    // Toughness 5 or greater: +4/-4.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Blood Lust");
    t.lands(P0, "Mountain", 2);
    let big = t.battlefield(P1, "Craw Wurm"); // 6/4
    let wall = t.battlefield(P1, "Wall of Stone"); // 0/8
    t.cast(P0, spell).target(Entity::Object(wall)).go();
    t.resolve();
    assert_eq!(t.pt(wall), (4, 4));
    // Otherwise: +4/-X where X is its toughness minus 1, leaving it at 1.
    let spell = t.hand(P0, "Blood Lust");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, spell).target(Entity::Object(big)).go();
    t.resolve();
    assert_eq!(t.pt(big), (10, 1));
}

#[test]
fn rogue_skycaptain_pays_per_wage_counter_or_changes_control() {
    cr!("118.12", "122.1");
    ruling!(
        "Rogue Skycaptain",
        "If you can’t pay the entire payment, none of it is paid."
    );
    supported("Rogue Skycaptain");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Rogue Skycaptain");
    t.lands(P0, "Mountain", 3);
    // First upkeep: one wage counter, pay {2}.
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "wage"), 1);
    assert_eq!(t.obj_now(s).controller, P0);
    // Next upkeep: two counters, {4}: only three lands, so it can't be paid.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "wage"), 0);
    assert_eq!(t.obj_now(s).controller, P1);
}
