//! Power/toughness changes with multipliers and "additional" changes (CR 613.4c).

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn deserts_due_gets_an_additional_minus_one_for_each_desert() {
    cr!("613.4c");
    assert_supported("Desert's Due");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Colossal Dreadmaw");
    t.battlefield(P0, "Desert");
    t.battlefield(P0, "Desert");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Desert's Due");
    t.cast(P0, s).target(target).go();
    t.resolve();
    // Colossal Dreadmaw 6/6: -2/-2 and -1/-1 twice.
    assert_eq!(t.pt(target), (2, 2));
}

#[test]
fn wild_might_any_player_may_pay_to_stop_the_additional_bonus() {
    cr!("118.12a");
    assert_supported("Wild Might");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 2);
        t.lands(P1, "Island", 2);
        let s = t.hand(P0, "Wild Might");
        t.cast(P0, s).target(bears).go();
        t.answer_yes(P0, false);
        t.answer_yes(P1, pays);
        t.resolve();
        assert_eq!(t.pt(bears), if pays { (3, 3) } else { (7, 7) }, "{pays}");
    }
}

#[test]
fn nuclear_fallout_gives_twice_minus_x() {
    cr!("613.4c", "107.3");
    assert_supported("Nuclear Fallout");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Nuclear Fallout");
    t.cast(P0, s).x(1).go();
    t.resolve();
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn exponential_growth_doubles_power_x_times() {
    cr!("701.10d", "107.3");
    assert_supported("Exponential Growth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 8);
    let s = t.hand(P0, "Exponential Growth");
    t.cast(P0, s).x(3).target(bears).go();
    t.resolve();
    // 2 → 4 → 8 → 16.
    assert_eq!(t.pt(bears), (16, 2));
}

#[test]
fn yare_lets_the_creature_block_two_additional_creatures() {
    cr!("509.1b");
    assert_supported("Yare");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.lands(P1, "Plains", 3);
    let y = t.hand(P1, "Yare");
    t.cast(P1, y).target(wall).go();
    t.resolve();
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(wall, a), (wall, b), (wall, c)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}
