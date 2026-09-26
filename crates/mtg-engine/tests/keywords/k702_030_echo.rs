//! CR 702.30 Echo.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use crate::common_k702_027_037::*;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const ECHO: &str = "Echo";

#[test]
fn echo_asks_for_its_cost_at_the_first_upkeep_after_it_enters() {
    cr!("702.30", "702.30a");
    ruling!(
        "Karmic Guide",
        "Paying for echo is always optional. When the echo triggered ability resolves, if you can't pay the echo cost or choose not to, you sacrifice that permanent."
    );
    assert_supported("Goblin Patrol");
    // Paid: it stays.
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Goblin Patrol");
    t.lands(P0, "Mountain", 1);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, ECHO), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(patrol));
    assert_eq!(untapped_lands(&t, P0), 0);
    // Not paid: it's sacrificed.
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Goblin Patrol");
    t.lands(P0, "Mountain", 1);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(!t.on_battlefield(patrol));
    assert!(t.in_graveyard(P0, "Goblin Patrol"));
    assert_eq!(untapped_lands(&t, P0), 1);
    // Can't be paid: it's sacrificed.
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Goblin Patrol");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(patrol));
}

#[test]
fn echo_triggers_only_once_for_each_time_it_comes_under_your_control() {
    cr!("702.30a");
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Goblin Patrol");
    t.lands(P0, "Mountain", 1);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(patrol));
    // The next upkeep: it's been under P0's control since before the last upkeep.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, ECHO), 0);
    assert!(t.on_battlefield(patrol));
    assert_eq!(pay_questions(&t, P0), 1);
}

#[test]
fn echo_triggers_only_in_its_controllers_upkeep() {
    cr!("702.30a");
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Goblin Patrol");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, ECHO), 0);
    assert!(t.on_battlefield(patrol));
}

#[test]
fn echo_triggers_after_gaining_control_of_the_permanent() {
    cr!("702.30a");
    ruling!(
        "Karmic Guide",
        "Your permanent's echo ability will trigger at the beginning of your upkeep if it entered the battlefield since the beginning of your last upkeep, or if you gained control of it since the beginning of your last upkeep."
    );
    assert_supported("Control Magic");
    let mut t = TestGame::new(2);
    // P1 has controlled it for a while (it paid its echo long ago).
    let patrol = t.battlefield(P1, "Goblin Patrol");
    t.lands(P1, "Mountain", 1);
    next_upkeep(&mut t, P1);
    t.answer_yes(P1, true);
    t.resolve();
    next_upkeep(&mut t, P1);
    assert_eq!(triggers_on_stack(&t, ECHO), 0);
    // P0 takes it.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Mountain", 1);
    let magic = t.hand(P0, "Control Magic");
    t.cast(P0, magic).target(patrol).go();
    t.resolve();
    assert_eq!(t.obj_now(patrol).controller, P0);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, ECHO), 1);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(!t.on_battlefield(patrol));
    assert!(t.in_graveyard(P1, "Goblin Patrol"));
}

#[test]
fn a_permanent_that_arrives_during_your_upkeep_echoes_at_your_next_upkeep() {
    cr!("702.30a");
    assert_supported("Simian Grunts");
    let mut t = TestGame::new(2);
    next_upkeep(&mut t, P0);
    // Flashed in during the upkeep, after it began: no echo trigger this upkeep...
    t.lands(P0, "Forest", 3);
    let grunts = t.hand(P0, "Simian Grunts");
    t.cast(P0, grunts).go();
    t.resolve();
    assert!(t.on_battlefield(grunts));
    assert_eq!(triggers_on_stack(&t, ECHO), 0);
    // ...but it came under P0's control since the beginning of that upkeep.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, ECHO), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(grunts));
}

#[test]
fn an_echo_cost_can_be_a_non_mana_cost() {
    cr!("702.30a");
    assert_supported("Rakdos Headliner");
    let mut t = TestGame::new(2);
    let headliner = t.battlefield(P0, "Rakdos Headliner");
    let bears = t.hand(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert!(t.on_battlefield(headliner));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand - 1);
    // With no card to discard, it can't be paid.
    let mut t = TestGame::new(2);
    let headliner = t.battlefield(P0, "Rakdos Headliner");
    next_upkeep(&mut t, P0);
    for c in t.g.player(P0).hand.clone() {
        t.g.discard(P0, c, None);
    }
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(headliner));
}

#[test]
fn nothing_happens_if_the_permanent_left_the_battlefield() {
    cr!("702.30a");
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Goblin Patrol");
    t.lands(P0, "Mountain", 1);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, ECHO), 1);
    t.g.destroy(patrol, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(pay_questions(&t, P0), 0);
    assert_eq!(untapped_lands(&t, P0), 1);
}

#[test]
fn urza_block_echo_costs_equal_their_mana_costs() {
    cr!("702.30b");
    for name in ["Goblin Patrol", "Pouncing Jaguar", "Viashino Outrider"] {
        let c = card(name);
        let chars = &c.front().chars;
        let echo = chars
            .keywords()
            .find(|k| k.kind == KeywordKind::Echo)
            .and_then(|k| k.cost.clone())
            .expect("echo cost");
        assert_eq!(
            format!("{}", echo.mana.unwrap()),
            format!("{}", chars.mana_cost.clone().unwrap()),
            "{name}"
        );
    }
    // The trigger asks for exactly that cost.
    let mut t = TestGame::new(2);
    let outrider = t.battlefield(P0, "Viashino Outrider");
    t.lands(P0, "Mountain", 2);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    // {2}{R} can't be paid with two lands.
    assert!(!t.on_battlefield(outrider));
    let mut t = TestGame::new(2);
    let outrider = t.battlefield(P0, "Viashino Outrider");
    t.lands(P0, "Mountain", 3);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(outrider));
    assert_eq!(untapped_lands(&t, P0), 0);
}
