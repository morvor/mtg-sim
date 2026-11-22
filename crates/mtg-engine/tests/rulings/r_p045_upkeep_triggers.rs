//! Rulings batch P045 — "at the beginning of each opponent's upkeep, if that player has N
//! or fewer cards in hand, ..." (intervening "if" clauses, CR 603.4), and abilities that
//! trigger during the untap step (CR 502.4, 503.1a).

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use crate::r_s17_common::enter_transformed;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls `permanent`; P1 holds `hand` Grizzly Bears plus, if `inspiration`, an
/// Inspiration ("Target player draws two cards.") and lands to cast it. Advances to P1's
/// upkeep (triggers put on the stack).
fn to_p1_upkeep(t: &mut TestGame, hand: usize, inspiration: bool) {
    bears_in_hand(t, P1, hand);
    if inspiration {
        t.hand(P1, "Inspiration");
        give_mana_for(t, P1, "Inspiration");
    }
    t.advance_to(P1, Step::Upkeep);
    t.settle();
}

/// In response to the upkeep trigger, P1 casts Inspiration targeting themself.
fn p1_draws_two_in_response(t: &mut TestGame) {
    let insp = t.g.find_in_zone(mtg_engine::object::Zone::Hand(P1), "Inspiration")[0];
    t.cast(P1, insp).target(Entity::Player(P1)).go();
    t.resolve();
}

#[test]
fn davriel_checks_the_hand_as_the_upkeep_begins_and_again_on_resolution() {
    cr!("603.4", "503.1a", "504.1");
    ruling!(
        "Davriel, Rogue Shadowmage",
        "If an opponent has two or more cards in their hand as their upkeep begins, Davriel's first ability won't trigger."
    );
    ruling!(
        "Davriel, Rogue Shadowmage",
        "If an opponent has one or fewer cards in hand as Davriel's first ability triggers but has two or more cards in hand as that ability resolves, Davriel doesn't deal damage to them."
    );
    supported("Davriel, Rogue Shadowmage");
    // Two cards as the upkeep begins: no trigger (the draw step comes later).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Davriel, Rogue Shadowmage");
    to_p1_upkeep(&mut t, 2, false);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P1), 2);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.life(P1), 20);
    // One card (Inspiration): it triggers; P1 draws two in response: no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Davriel, Rogue Shadowmage");
    to_p1_upkeep(&mut t, 0, true);
    assert_eq!(t.stack_len(), 1);
    p1_draws_two_in_response(&mut t);
    assert_eq!(t.hand_size(P1), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Without the response it deals 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Davriel, Rogue Shadowmage");
    to_p1_upkeep(&mut t, 1, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn davriel_in_two_headed_giant_triggers_for_each_opponent() {
    cr!("603.4", "810.10a", "810.2");
    ruling!(
        "Davriel, Rogue Shadowmage",
        "In a Two-Headed Giant game, Davriel's first ability triggers for each opponent separately as appropriate during that player's team's upkeep."
    );
    // P0 (team P0+P1) controls Davriel. P2 holds no card, P3 holds three: only P2's
    // trigger. Then both with at most one card: two triggers.
    for (p3_hand, triggers) in [(3usize, 1usize), (1, 2)] {
        let mut t = crate::r_p050_common::two_headed_giant();
        t.battlefield(P0, "Davriel, Rogue Shadowmage");
        bears_in_hand(&mut t, P3, p3_hand);
        let before = crate::r_p050_common::team_lives(&t);
        t.advance_to(P2, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), triggers);
        t.resolve_all();
        let now = crate::r_p050_common::team_lives(&t);
        assert_eq!(before.1 - now.1, 2 * triggers as i32);
    }
}

#[test]
fn prickle_faeries_checks_the_hand_on_trigger_and_resolution() {
    cr!("603.4", "503.1a");
    ruling!(
        "Invasion of Eldraine // Prickle Faeries",
        "Prickle Faeries’s ability will check as each opponent’s upkeep begins whether that player has two or fewer cards in hand."
    );
    supported("Invasion of Eldraine // Prickle Faeries");
    // Three cards: no trigger.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Invasion of Eldraine // Prickle Faeries");
    to_p1_upkeep(&mut t, 3, false);
    assert_eq!(t.stack_len(), 0);
    // Two cards (one is Inspiration): triggers; P1 draws two in response (three cards):
    // no damage.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Invasion of Eldraine // Prickle Faeries");
    to_p1_upkeep(&mut t, 1, true);
    assert_eq!(t.stack_len(), 1);
    p1_draws_two_in_response(&mut t);
    assert_eq!(t.hand_size(P1), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Two cards and no response: 2 damage.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Invasion of Eldraine // Prickle Faeries");
    to_p1_upkeep(&mut t, 2, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn hollowsage_untap_step_trigger_waits_for_the_upkeep() {
    cr!("502.4", "503.1a", "603.3b");
    ruling!(
        "Hollowsage",
        "If Hollowsage becomes untapped during your untap step, the ability will trigger. However, since no player gets priority during the untap step, it waits to be put on the stack until your upkeep starts."
    );
    supported("Hollowsage");
    // P1 controls a tapped Hollowsage and Rotting Regisaur ("At the beginning of your
    // upkeep, discard a card."): both go on the stack in P1's upkeep, in the order P1
    // chooses.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P1, "Hollowsage");
    t.g.tap(h);
    t.battlefield(P1, "Rotting Regisaur");
    bears_in_hand(&mut t, P0, 1);
    bears_in_hand(&mut t, P1, 1);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_yes(P1, true);
    let from = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(!t.obj_now(h).tapped);
    assert_eq!(t.stack_len(), 2);
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::Order { .. })));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
