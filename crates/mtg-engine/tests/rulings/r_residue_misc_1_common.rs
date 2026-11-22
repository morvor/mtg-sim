//! Shared helpers for the tests of item `rulings-residue-misc-1` (`r_residue_misc_1_*.rs`):
//! the small open residues of batches P002–P058 (pass A of the misc-1 item). (The helpers
//! of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p046_common::{cost_to_cast, fill_pool, others_asked_since, pool, tap_lands};
pub use crate::r_residue_keywords_common::*;
pub use crate::r_s05_common::enter;
pub use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};

/// Advances from wherever the game is to the end step of `p`'s turn (its "at the
/// beginning of the end step" triggers on the stack), resolves everything, and settles.
pub fn through_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.resolve_all();
}

/// Makes the permanent (followed across zone changes) a creature with base power and
/// toughness `p`/`q` until end of turn, as an effect of its controller's.
pub fn animate(t: &mut TestGame, id: ObjectId, p: i32, q: i32) {
    let id = t.g.current(id);
    let c = t.obj_now(id).controller;
    run_from(
        t,
        c,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::AddTypes(vec![CardType::Creature]),
                Modification::SetPT(Some(Value::Const(p)), Some(Value::Const(q))),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

/// Exiles the card (followed across zone changes) from wherever it is, and settles.
pub fn exile_card(t: &mut TestGame, id: ObjectId) {
    move_to(t, id, Zone::Exile);
}

/// Whether the object (followed across zone changes) is in `zone` now.
pub fn is_in(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    t.zone(id) == zone
}

/// The cards named `name` in `p`'s graveyard.
pub fn gy_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.find_in_zone(Zone::Graveyard(p), name)
}
