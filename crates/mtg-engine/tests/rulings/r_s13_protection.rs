//! Rulings batch S13 — "protection" (shared by Dunerider Outlaw and Whirling Dervish,
//! whose other ability reads "At the beginning of each end step, if this creature dealt
//! damage to an opponent this turn, put a +1/+1 counter on it.").

use crate::r_s01_common::*;
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn damage_dealt_to_an_opponent_who_left_the_game_still_counts() {
    cr!("603.4", "104.3a", "800.4a", "120.3a");
    ruling!(
        "Dunerider Outlaw",
        "If Dunerider Outlaw dealt damage to an opponent who later left the game before that turn's end step, its ability still gives it a +1/+1 counter."
    );
    supported("Dunerider Outlaw");
    supported("Whirling Dervish");
    // Three players: the Outlaw deals combat damage to P1, who then concedes.
    let mut t = TestGame::new(3);
    let outlaw = t.battlefield(P0, "Dunerider Outlaw");
    attack_with(&mut t, &[(outlaw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 19);
    t.g.perform_action(P1, Action::Concede).expect("concede");
    assert!(t.has_lost(P1));
    assert!(!t.g.player(P1).in_game());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(outlaw, counters::PLUS1), 1);
    assert_eq!(t.pt(outlaw), (2, 2));
    // It didn't deal damage to an opponent the next turn: no counter at that end step.
    t.advance_to(P2, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(outlaw, counters::PLUS1), 1);
    // Whirling Dervish, which dealt no damage this turn: no counter.
    let mut t = TestGame::new(2);
    let dervish = t.battlefield(P0, "Whirling Dervish");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(dervish, counters::PLUS1), 0);
    // Dealing damage to an opponent (not in combat) counts too.
    let mut t = TestGame::new(2);
    let dervish = t.battlefield(P0, "Whirling Dervish");
    t.g.deal_damage(dervish, Entity::Player(P1), 1, false);
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(dervish, counters::PLUS1), 1);
}
