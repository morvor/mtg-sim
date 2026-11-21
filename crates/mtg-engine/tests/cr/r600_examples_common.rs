//! Shared helpers for the tests of the CR 600–611 worked examples (`r60N_examples.rs`).

#![allow(dead_code)]

use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::Action;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r506_common::custom_card;

/// Asserts that a real card the example names compiles completely.
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// Whether `p` may begin to cast `card` with `method` right now (a legal action).
pub fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: &CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, method: m } if *c == card && m == method))
}

/// The targets chosen for each target slot of a spell or ability on the stack.
pub fn chosen_targets(t: &TestGame, id: ObjectId) -> Vec<Vec<Entity>> {
    t.obj(id)
        .stack
        .as_ref()
        .expect("not on the stack")
        .chosen
        .iter()
        .flat_map(|c| c.targets.clone())
        .collect()
}

/// A custom card compiled from oracle text (see [`custom_card`]) with a mana cost.
pub fn custom_costed(name: &str, type_line: &str, cost: &str, text: &str) -> CardDef {
    let mut def = custom_card(name, type_line, None, text);
    let m = mtg_engine::mana::ManaCost::parse(cost).expect("bad mana cost");
    def.faces[0].chars.colors = m.colors();
    def.faces[0].chars.mana_cost = Some(m);
    def
}
