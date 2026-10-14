//! What an instruction reports to the instructions after it ("if you do", "that many",
//! CR 608.2c): scrying 0 isn't scrying (CR 701.22b); turning a permanent face down is
//! doing it; putting one object's counters on another counts the counters put (CR 122.8).

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `first`, then "if you did, you gain 3 life".
fn then_report(first: Effect) -> Effect {
    Effect::Seq(vec![
        first,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(3),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ])
}

#[test]
fn scrying_zero_doesnt_count_as_scrying() {
    cr!("701.22b", "608.2c");
    for (n, life) in [(0, 20), (1, 23)] {
        let mut t = TestGame::new(2);
        run(
            &mut t,
            P0,
            None,
            then_report(Effect::Scry {
                who: PlayerRef::You,
                n: Value::c(n),
            }),
        );
        assert_eq!(t.life(P0), life, "scry {n}");
    }
}

#[test]
fn turning_a_creature_face_down_is_doing_it() {
    cr!("608.2c", "708.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run(
        &mut t,
        P0,
        Some(bears),
        then_report(Effect::TurnFaceDown { what: Sel::This }),
    );
    assert!(t.obj_now(bears).face_down);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn that_many_after_putting_its_counters_on_another_creature() {
    cr!("122.8", "608.2c");
    let mut t = TestGame::new(2);
    let from = t.battlefield(P0, "Grizzly Bears");
    let to = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(from), "+1/+1", 2, None);
    run_targeted(
        &mut t,
        P0,
        Some(from),
        Effect::Seq(vec![
            Effect::PutCountersOf {
                from: Sel::This,
                to: Sel::Target(0),
                kind: None,
            },
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::Prev,
            },
        ]),
        vec![vec![Entity::Object(to)]],
    );
    assert_eq!(t.counters(to, "+1/+1"), 2);
    assert_eq!(t.life(P0), 22);
}
