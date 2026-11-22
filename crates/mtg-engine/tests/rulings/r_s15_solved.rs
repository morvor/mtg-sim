//! Rulings batch S15 — solved (CR 702.169, 719.3): Case of the Gateway Express.

use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::cases;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_case_stays_solved_until_it_leaves_the_battlefield() {
    cr!("719.3a", "719.3b", "702.169b");
    ruling!(
        "Case of the Gateway Express",
        "Once a Case becomes solved, it stays solved until it leaves the battlefield."
    );
    supported("Case of the Gateway Express");
    // "To solve — Three or more creatures attacked this turn." "Solved — Creatures you
    // control get +1/+0."
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Gateway Express");
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let attackers: Vec<(ObjectId, Entity)> =
        bears.iter().map(|b| (*b, Entity::Player(P1))).collect();
    attack_with(&mut t, &attackers);
    block_and_finish(&mut t, P1, &[]);
    // It becomes solved at the beginning of P0's end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(cases::is_solved(&t.g, case));
    assert_eq!(t.pt(bears[0]), (3, 2));
    // On later turns, with no creature attacking, it's still solved.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(cases::is_solved(&t.g, case));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(cases::is_solved(&t.g, case));
    assert_eq!(t.pt(bears[0]), (3, 2));
    // Once it leaves the battlefield, it's a new object that isn't solved.
    let exiled = move_to(&mut t, case, Zone::Exile).expect("exiled");
    let back = move_to(&mut t, exiled, Zone::Battlefield).expect("returned");
    assert!(!cases::is_solved(&t.g, back));
    assert_eq!(t.pt(bears[0]), (2, 2));
}
