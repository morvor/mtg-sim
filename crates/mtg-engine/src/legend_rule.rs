//! Exemptions from the legend rule (CR 704.5j): "The "legend rule" doesn't apply." (Mirror
//! Gallery), "The "legend rule" doesn't apply to tokens you control." (Cadric, Soul
//! Kindler). A permanent a static ability exempts isn't counted when the legend rule
//! looks for legendary permanents with the same name controlled by the same player.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::ObjectId;

/// Whether an active static ability exempts the permanent `id` from the legend rule.
pub fn exempt(g: &Game, id: ObjectId) -> bool {
    g.statics.other.iter().any(|(src, ctl, e)| match e {
        StaticEffect::LegendRuleExempt(f) => {
            let ctx = Ctx::new(Some(*src), *ctl);
            g.matches(id, f, &ctx)
        }
        _ => false,
    })
}
