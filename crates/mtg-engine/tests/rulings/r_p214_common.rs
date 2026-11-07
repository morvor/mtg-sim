//! Shared helpers for the tests of rulings batch P214 (`r_p214_*.rs`): evolve, exert,
//! exhaust, exploit, explore, fabricate, fast healing, fateful hour, ferocious, fight.

#![allow(dead_code)]

use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The target creature gets +p/+t until end of turn (as a resolving effect of P1's).
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    run_from(
        t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

/// Whether the object (followed across zone changes) is exerted.
pub fn exerted(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).exerted
}

/// Whether the object (followed across zone changes) is tapped.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Whether the decision is an "Exert ...?" question.
pub fn is_exert_question(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Exert"))
}

/// Whether the decision is a yes/no question.
pub fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}
