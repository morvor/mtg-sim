//! Shared helpers for the tests of item `rulings-residue-type-change`
//! (`r_residue_type_change_*.rs`): the open rulings left in batches P187 and P188 —
//! type-changing effects and layers. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p108_common::obj;
pub use crate::r_p119_common::{giant_growth, has_kw, stack};
pub use crate::r_p120_common::{give_plus1, plus1};
pub use crate::r_p121_common::{cast_resolve, power};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, custom_card, place, supported, triggers_on_stack,
};
pub use crate::r_s02_common::destroy;
pub use crate::r_s24_common::{choose_creature_type, controller, tapped};
pub use crate::r_s25_common::{cast_new, lands_for_cost, legendary};

/// Whether the object (followed across zone changes) has the subtype now.
pub fn subtype(t: &mut TestGame, id: ObjectId, s: &str) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.has_subtype(s)
}

/// The creature subtypes of the object (followed across zone changes) now.
pub fn creature_types(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    let lists = subtype_lists();
    let mut v: Vec<String> = t
        .obj_now(id)
        .chars
        .subtypes
        .iter()
        .filter(|s| lists.creature.iter().any(|c| c == s.as_str()))
        .map(|s| s.to_string())
        .collect();
    v.sort();
    v
}

/// The power and toughness of the object (followed across zone changes) now.
pub fn pt_now(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(t.g.current(id))
}

/// Whether the object (followed across zone changes) has the color now.
pub fn has_color(t: &mut TestGame, id: ObjectId, c: Color) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.colors.contains(c)
}

/// Whether the object (followed across zone changes) has the card type now.
pub fn is_type(t: &mut TestGame, id: ObjectId, ty: CardType) -> bool {
    t.g.recompute();
    t.obj_now(id).is(ty)
}

/// P0 activates `source`'s `index`th activated ability with X = `x` (Wastes for the X are
/// added; the rest of the cost is paid from lands already there).
pub fn activate_x(t: &mut TestGame, source: ObjectId, index: usize, x: i64, targets: &[Entity]) {
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, source, index, targets).unwrap();
}

/// P0 casts Diminish ("Target creature has base power and toughness 1/1 until end of
/// turn.") on `target` and only it resolves.
pub fn diminish(t: &mut TestGame, target: ObjectId) {
    cast_new(t, P0, "Diminish", &[obj(target)]);
    t.resolve();
}

/// P0's creature with flying attacks P1 with Shadow Puppeteers on P0's battlefield: the
/// Puppeteers' trigger is on the stack (P0 will choose to have it become a Dragon).
pub fn puppeteers_attack(t: &mut TestGame, flier: ObjectId) {
    t.set_step(P0, Step::PrecombatMain);
    attack_with(t, &[(flier, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    assert_eq!(triggers_on_stack(t, "Dragon"), 1);
}

/// Mills P1's top card, a real `name`, and returns the milled card in P1's graveyard.
pub fn milled(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.library_top(P1, name);
    t.g.mill(P1, 1);
    t.settle();
    t.g.current(c)
}
