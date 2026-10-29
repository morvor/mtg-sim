//! Modes already chosen for a modal ability, for "choose one that hasn't been chosen" and
//! "choose one that hasn't been chosen this turn" ([`ModeChooser::Unchosen`], CR 700.2).
//!
//! * The history belongs to one object's ability: a permanent that leaves the battlefield
//!   and returns is a new object with no memory of the modes chosen for it (CR 400.7), and
//!   two permanents with the same ability track their modes separately.
//! * A mode counts as chosen once the ability is put on the stack with it (CR 700.2b), even
//!   if the ability is later countered or its targets become illegal.
//! * With every mode already chosen, no mode can be chosen, so the ability is removed from
//!   the stack (CR 700.2b, 603.3c).

use crate::ability::ModeChooser;
use crate::game::Game;
use crate::object::StackKind;
use crate::types::ObjectId;
use std::collections::BTreeMap;

/// Which modes have been chosen for each object's modal ability, and on which turn.
#[derive(Clone, Debug, Default)]
pub struct ModalHistory {
    /// (source object, ability uid) → (mode index, turn number) for each choice.
    pub chosen: BTreeMap<(ObjectId, u64), Vec<(usize, u32)>>,
}

/// Whose ability a stack object is: the source and the ability's uid; a spell is its own
/// source.
fn key(g: &Game, stack_obj: ObjectId) -> (ObjectId, u64) {
    match g.obj(stack_obj).stack.as_ref().map(|s| &s.kind) {
        Some(StackKind::Activated { source, ability })
        | Some(StackKind::Triggered { source, ability }) => (*source, ability.uid),
        _ => (stack_obj, 0),
    }
}

/// Whether mode `i` may be chosen for the stack object under this chooser.
pub fn may_choose(g: &Game, stack_obj: ObjectId, chooser: &ModeChooser, i: usize) -> bool {
    let ModeChooser::Unchosen { this_turn } = chooser else {
        return true;
    };
    let turn = g.turn.number;
    !g.modal_history
        .chosen
        .get(&key(g, stack_obj))
        .is_some_and(|v| v.iter().any(|(m, t)| *m == i && (!this_turn || *t == turn)))
}

/// Records the modes chosen for the stack object (as it's put on the stack).
pub fn record(g: &mut Game, stack_obj: ObjectId, chooser: &ModeChooser, modes: &[usize]) {
    if !matches!(chooser, ModeChooser::Unchosen { .. }) {
        return;
    }
    let k = key(g, stack_obj);
    let turn = g.turn.number;
    g.modal_history
        .chosen
        .entry(k)
        .or_default()
        .extend(modes.iter().map(|m| (*m, turn)));
}
