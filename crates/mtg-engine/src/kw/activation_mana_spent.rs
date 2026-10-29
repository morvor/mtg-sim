//! "When you next activate an ability that isn't a mana ability this turn by spending
//! four or more mana to activate it, copy that ability." (Dynaheir, Invoker Adept): the
//! mana spent on the costs of the activated ability whose activation triggered (CR 602.2b,
//! 601.2h), not mana spent on mana abilities activated while paying them.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Condition::Custom` prefix, followed by N: N or more mana was spent to activate the
/// ability whose activation triggered this one.
pub const MANA_SPENT_TO_ACTIVATE: &str = "activation paid:mana spent>=";

struct ActivationManaSpent;

impl KeywordRules for ActivationManaSpent {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let n: usize = name.strip_prefix(MANA_SPENT_TO_ACTIVATE)?.parse().ok()?;
        let id = ctx.event.as_ref().and_then(|e| e.spell);
        Some(id.is_some_and(|id| {
            g.obj(id)
                .stack
                .as_deref()
                .is_some_and(|si| si.cast.mana_spent.len() >= n)
        }))
    }
}

inventory::submit! { KeywordRegistration(&ActivationManaSpent) }
