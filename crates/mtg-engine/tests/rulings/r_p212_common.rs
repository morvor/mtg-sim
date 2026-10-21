//! Shared helpers for the tests of rulings batch P212 (`r_p212_*.rs`): Equipment (the
//! "keyword: Equip" group). (The helpers of the S batches and of P213 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::give_mana_for;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Gives the permanent the keyword until end of turn (a non-copy effect created now).
pub fn grant(t: &mut TestGame, id: ObjectId, k: KeywordKind) {
    modify_until_eot(t, id, vec![Modification::AddKeyword(Keyword::new(k))]);
}

/// Gives the permanent +p/+t until end of turn (a non-copy effect created now).
pub fn boost(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
    );
}

/// `p` casts the real spell `name` with X = `x` (with lands for it), leaving it on the
/// stack.
pub fn cast_x(t: &mut TestGame, p: PlayerId, name: &str, x: usize) -> ObjectId {
    give_mana_for(t, p, name);
    t.lands(p, "Wastes", x);
    let c = t.hand(p, name);
    t.cast(p, c).x(x as i64).go()
}

/// The real modal double-faced card `name` on the battlefield under `p`'s control with
/// its back face up (as if that face had been cast).
pub fn back_face(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    let o = &mut t.g.objects[id.0 as usize];
    o.face = FaceState::Back;
    o.base = o.card.clone().expect("a card").characteristics(FaceState::Back);
    t.g.recompute();
    id
}

/// Attaches the Equipment (followed across zone changes) to `to`, as an effect would.
pub fn move_eq(t: &mut TestGame, eq: ObjectId, to: ObjectId) {
    let eq = t.g.current(eq);
    assert!(t.g.attach(eq, Entity::Object(t.g.current(to))));
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Whether the Equipment (followed across zone changes) is attached to `to`.
pub fn on(t: &TestGame, eq: ObjectId, to: ObjectId) -> bool {
    t.obj_now(eq).attached_to == Some(Entity::Object(t.g.current(to)))
}
