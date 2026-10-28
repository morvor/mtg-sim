//! Shared helpers for the tests of rulings batch P185 (`r_p185_*.rs`): typal cards —
//! Humans, Knights, Merfolk, Pirates, Slivers, Spirits, Vampires, Wolves and Werewolves.

#![allow(dead_code)]

use crate::r_s01_common::custom_card;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `name` onto the battlefield for `p` through a real zone change, answering its
/// trigger's targets with `targets`, and puts any triggers on the stack.
pub fn enter_with(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// Puts a custom vanilla creature with the given type line onto the battlefield.
pub fn vanilla(t: &mut TestGame, p: PlayerId, type_line: &str, pt: (i32, i32)) -> ObjectId {
    let def = custom_card("Test Creature", type_line, "{2}", Some(pt), "");
    t.custom(p, def, Zone::Battlefield)
}

/// Gives the permanent +`p`/+`q` until end of turn (as a resolving spell would).
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32, q: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(p), Value::c(q))],
    );
}

/// Adds `types` to the permanent's creature types until end of turn.
pub fn add_types(t: &mut TestGame, id: ObjectId, types: &[&str]) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::AddSubtypes(
            types.iter().map(|s| (*s).into()).collect(),
        )],
    );
}

/// Damage marked on the permanent now.
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// The checks of the "damage remains marked … may become lethal if [lord] leaves"
/// rulings: `member` is dealt nonlethal damage while `leaver` is on the battlefield, then
/// `leaver` is destroyed and the marked damage is lethal.
pub fn lethal_when_it_leaves(t: &mut TestGame, member: ObjectId, leaver: ObjectId, what: &str) {
    let (_, tough) = t.pt(member);
    let src = t.g.current(member);
    crate::r_s06_common::damage(t, src, tough - 1, member);
    assert!(t.on_battlefield(member), "{what}: survives while boosted");
    crate::r_s02_common::destroy(t, leaver);
    assert!(!t.on_battlefield(member), "{what}: the marked damage is now lethal");
}
