//! Shared helpers for the tests of rulings batch P057 (`r_p057_*.rs`): "if you control a
//! creature with power 4 or greater" abilities, pump abilities, and choices made as a
//! permanent enters. (The helpers of batches S01–S30 are used too.)

#![allow(dead_code)]

use crate::r_s05_common::run_from;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives `id` +`p`/+`tough` until end of turn (as a resolving effect controlled by
/// `id`'s controller would).
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    let id = t.g.current(id);
    let who = t.obj_now(id).controller;
    run_from(
        t,
        who,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![id])),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

/// Advances from `p`'s precombat main phase into `p`'s beginning of combat step (its
/// "at the beginning of combat" triggers are put on the stack).
pub fn into_beginning_of_combat(t: &mut TestGame, p: PlayerId) {
    t.g.combat = None;
    t.set_step(p, Step::PrecombatMain);
    t.advance_to(p, Step::BeginningOfCombat);
    t.settle();
}

/// Advances to `p`'s next upkeep from `p`'s draw-less untap: the "at the beginning of your
/// upkeep" triggers are on the stack and `p` has priority.
pub fn into_upkeep(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::Untap);
    t.advance_to(p, Step::Upkeep);
    t.settle();
}
