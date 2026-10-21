//! Conditions of delayed triggered abilities about the objects they return (CR 603.4,
//! 603.7c): "At the beginning of the next end step, if you haven't cast the card, return
//! it to its owner's hand." is read as "if that card is still in exile" (see
//! `oracle/patterns/delayed_grammar_2.rs` and the interpretation
//! `delayed-if-not-played-means-still-exiled`).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::Entity;

/// `Condition::Custom` prefix, followed by a variable number: one of the objects the
/// variable holds (followed to the card the creating ability moved, as the delayed
/// instruction finds it) is in exile.
pub const STILL_EXILED: &str = "still exiled:";

struct DelayedConditions;

impl KeywordRules for DelayedConditions {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let var: crate::ability::Var = name.strip_prefix(STILL_EXILED)?.parse().ok()?;
        let objs = ctx.vars.get(&var).cloned().unwrap_or_default();
        Some(objs.into_iter().any(|e| {
            let Entity::Object(o) = g.found_after_move(e, ctx) else {
                return false;
            };
            g.is_live(o) && g.obj(o).zone == Zone::Exile
        }))
    }
}

inventory::submit! { KeywordRegistration(&DelayedConditions) }
