//! Shared helpers for the tests of CR 702.38–702.51 (amplify, provoke, storm, affinity,
//! entwine, modular, sunburst, bushido, soulshift, splice, offering, ninjutsu, epic,
//! convoke).

#![allow(dead_code)]

use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Stack objects that are copies of spells named `name`.
pub fn spell_copies_on_stack(t: &TestGame, name: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.kind == object::ObjKind::SpellCopy && o.chars.name == name
        })
        .count()
}

/// Spells named `name` on the stack (cards and copies).
pub fn spells_on_stack(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            let o = t.g.obj(*id);
            o.is_spell() && o.chars.name == name
        })
        .collect()
}

/// Triggered abilities on the stack whose text is `text`.
pub fn triggers_named(t: &TestGame, text: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            t.g.obj(*id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, StackKind::Triggered { ability, .. }
                    if ability.text.as_str() == text)
            })
        })
        .collect()
}

/// Gives a custom card a mana cost (and the colors of that cost).
pub fn with_cost(mut def: card::CardDef, cost: &str) -> card::CardDef {
    let m = mana::ManaCost::parse(cost).expect("bad mana cost");
    let chars = &mut def.faces[0].chars;
    chars.colors = m.colors();
    chars.mana_cost = Some(m);
    def
}

/// Casts a spell for `p` targeting `targets` (one per slot) and returns it.
pub fn cast_at(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> ObjectId {
    t.cast_with(p, card, targets).expect("casting failed")
}
