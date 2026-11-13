//! Rulings batch P102 — engine fix: a target in an instruction another player performs
//! ("that player puts a flood counter on target non-Island land they control", "you may
//! have that player return target permanent card from their graveyard") is chosen as the
//! ability goes on the stack, but "they"/"their" still names that player, not the
//! ability's controller (CR 115.1, 601.2c).

use crate::r_p102_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::next_upkeep;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn quicksilver_fountain_targets_a_land_of_the_upkeep_player() {
    cr!("115.1", "603.3d");
    // P0's Fountain; in P1's upkeep, P1 chooses one of P1's own lands.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quicksilver Fountain");
    let mine = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Plains");
    t.answer_targets(P1, &[o(theirs)]);
    let from = t.asked().len();
    next_upkeep(&mut t, P1);
    t.resolve_all();
    let offered = target_candidates(&t, P1, from);
    assert_eq!(offered, vec![vec![o(theirs)]]);
    assert_eq!(t.counters(theirs, "flood"), 1);
    assert!(t.obj_now(theirs).chars.has_subtype("Island"));
    assert_eq!(t.counters(mine, "flood"), 0);
}

#[test]
fn neyam_shai_murad_targets_a_card_in_the_damaged_players_graveyard() {
    cr!("115.1", "510.3a");
    supported("Neyam Shai Murad");
    let mut t = TestGame::new(2);
    let neyam = t.battlefield(P0, "Neyam Shai Murad");
    let mine = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[o(theirs)]);
    t.answer_yes(P0, true);
    choose(&mut t, P1, &[mine]);
    let from = t.asked().len();
    t.attack(&[(neyam, Entity::Player(P1))], &[]);
    t.resolve_all();
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![o(theirs)]]);
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(t.obj_now(mine).controller, P0);
    assert!(t.on_battlefield(mine));
}
