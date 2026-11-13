//! Rules behind `oracle/patterns/delayed_grammar_3.rs`: "the amount of mana spent to cast
//! that spell" about a spell the ability targeted (Mana Sculpt: CR 601.2h, the mana paid
//! for it, as it last existed on the stack, CR 608.2h), also for a delayed instruction
//! performed later.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Value::Custom` prefix, followed by a target slot: the mana spent to cast the spell in
/// that slot.
pub const MANA_SPENT_ON_TARGET: &str = "mana spent to cast:target:";

struct DelayedGrammar3;

impl KeywordRules for DelayedGrammar3 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let k: usize = name.strip_prefix(MANA_SPENT_ON_TARGET)?.parse().ok()?;
        Some(
            ctx.targets
                .get(k)
                .into_iter()
                .flatten()
                .filter_map(|e| e.object())
                .filter_map(|o| g.obj(o).stack.as_deref())
                .map(|si| si.cast.mana_spent.len() as i64)
                .sum(),
        )
    }
}

inventory::submit! { KeywordRegistration(&DelayedGrammar3) }
