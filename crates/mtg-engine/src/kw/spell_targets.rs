//! "Whenever you cast a spell with one or more targets, draw that many cards." (Voracious
//! Bibliophile): the number of targets of the spell that triggered the ability — the
//! number of times an object or player was chosen as one of its targets (CR 115.9a). An
//! Aura spell has a target (CR 115.1b). The text is parsed in
//! `oracle/patterns/r115_spells_with_targets.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Value::Custom` name: the number of targets of the triggering spell.
pub const TRIGGERING_SPELL_TARGETS: &str = "number of targets of the triggering spell";

/// The number of times objects or players were chosen as targets of the stack object.
pub fn target_count(g: &Game, id: ObjectId) -> usize {
    g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .map(|cm| cm.targets.iter().map(Vec::len).sum::<usize>())
                .sum()
        })
        .unwrap_or(0)
}

pub struct SpellTargets;

impl KeywordRules for SpellTargets {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != TRIGGERING_SPELL_TARGETS {
            return None;
        }
        let spell = ctx.event.as_ref().and_then(|e| e.spell);
        Some(spell.map_or(0, |s| target_count(g, s)) as i64)
    }
}

inventory::submit! { KeywordRegistration(&SpellTargets) }
