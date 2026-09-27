//! Rulings batch S11 — modular (CR 702.43): "This permanent enters with N +1/+1 counters
//! on it" and "When this permanent is put into a graveyard from the battlefield, you may
//! put a +1/+1 counter on target artifact creature for each +1/+1 counter on this
//! permanent" (its last known information, CR 603.10a).

use crate::r_s01_common::*;
use crate::r_s05_common::enter;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

#[test]
fn modular_counts_the_plus_one_counters_it_had_as_minus_one_counters_killed_it() {
    cr!("702.43a", "704.5f", "704.5q", "704.3", "603.10a");
    ruling!(
        "Arcbound Whelp",
        "If this creature gets enough -1/-1 counters put on it to cause it to go to the graveyard, modular will put a number of +1/+1 counters on the target artifact creature equal to the number of +1/+1 counters on this creature before it left the battlefield."
    );
    supported("Arcbound Whelp");
    supported("Arcbound Javelineer");
    // Arcbound Whelp: 0/0, modular 2. Two -1/-1 counters make it 0/0: it dies as the
    // counters would annihilate (simultaneous state-based actions), with its two +1/+1
    // counters.
    let mut t = TestGame::new(2);
    let whelp = enter(&mut t, P0, "Arcbound Whelp");
    let worker = enter(&mut t, P0, "Arcbound Worker");
    assert_eq!(plus1(&t, whelp), 2);
    t.answer_targets(P0, &[Entity::Object(worker)]);
    t.answer_yes(P0, true);
    t.g.add_counters(Entity::Object(whelp), counters::MINUS1, 2, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Arcbound Whelp"));
    assert_eq!(plus1(&t, worker), 1 + 2);
    // Arcbound Javelineer (0/1, modular 1) given three -1/-1 counters by Skinrender.
    let mut t = TestGame::new(2);
    let javelineer = enter(&mut t, P0, "Arcbound Javelineer");
    let worker = enter(&mut t, P0, "Arcbound Worker");
    t.answer_targets(P1, &[Entity::Object(javelineer)]);
    t.answer_targets(P0, &[Entity::Object(worker)]);
    t.answer_yes(P0, true);
    enter(&mut t, P1, "Skinrender");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Arcbound Javelineer"));
    assert_eq!(plus1(&t, worker), 1 + 1);
}
