//! "If you cast this spell during your main phase, [effect]" (addendum, an ability word,
//! CR 207.2c) and "When ~ enters, if you cast it during your main phase, ...": whether the
//! spell was cast (not copied or put onto the stack otherwise) during its caster's main
//! phase (CR 505.1). Checked as the spell or ability resolves (CR 608.2b), from the cast
//! information the spell or permanent keeps.

use super::ConditionPattern;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::oracle::phrases::end;

pub const CAST_DURING_YOUR_MAIN_PHASE: &str = "cast during your main phase";

fn cast_during_main_phase(c: &str) -> Option<Condition> {
    let spell = end(c)
        .strip_prefix("you cast ")?
        .strip_suffix(" during your main phase")?;
    matches!(spell, "~" | "it" | "this spell")
        .then(|| Condition::Custom(CAST_DURING_YOUR_MAIN_PHASE.into()))
}

inventory::submit! { ConditionPattern { name: "r505 cast during your main phase", priority: 60, parse: cast_during_main_phase } }

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    (name == CAST_DURING_YOUR_MAIN_PHASE)
        .then(|| g.cast_info(ctx).is_some_and(|c| c.was_cast && c.main_phase))
}
