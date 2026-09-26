//! A spell's own conditional cost changes and alternative costs (CR 601.2b, 601.2f,
//! 118.9): "This spell costs {2} less to cast if it targets a tapped creature.", "If you
//! control a Swamp, you may pay 4 life rather than pay this spell's mana cost."

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// "for each target beyond the first" (strive): how many targets the spell `ctx.source`
/// has beyond the first, once they're chosen (CR 601.2c); none before that.
pub const TARGETS_BEYOND_FIRST: &str = "spell_targets_beyond_first";

pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    if name != TARGETS_BEYOND_FIRST {
        return None;
    }
    let n: usize = ctx
        .source
        .and_then(|s| g.obj(s).stack.as_deref())
        .map_or(0, |si| {
            si.chosen
                .iter()
                .map(|cm| cm.targets.iter().map(Vec::len).sum::<usize>())
                .sum()
        });
    Some(n.saturating_sub(1) as i64)
}

/// Whether a condition depends on choices made while the spell is proposed (its targets,
/// or which optional costs are paid, CR 601.2b–c), which aren't known before then.
fn depends_on_choices(c: &Condition) -> bool {
    match c {
        Condition::SelMatches(Sel::This, Filter::Targets(_)) | Condition::CostPaid(_) => true,
        Condition::Not(c) => depends_on_choices(c),
        Condition::And(v) | Condition::Or(v) => v.iter().any(depends_on_choices),
        _ => false,
    }
}

fn is_reduction(c: &CostChange) -> bool {
    matches!(
        c,
        CostChange::ReduceGeneric(_)
            | CostChange::ReduceColored(..)
            | CostChange::ReduceMana { .. }
    )
}

/// Whether the spell's own cost change `cm` (an ability `s` of the spell `card`)
/// applies to its total cost (CR 601.2f). The condition is checked once the spell's
/// targets and other choices are made; before the spell is proposed (while checking
/// whether it could be cast), a change that depends on those choices is assumed to
/// apply if it's a reduction and not to apply if it's an increase.
pub fn own_change_applies(
    g: &Game,
    card: ObjectId,
    s: &StaticAbility,
    cm: &CostModifier,
    ctx: &Ctx,
) -> bool {
    let Some(cond) = &s.condition else {
        return true;
    };
    if g.obj(card).zone != Zone::Stack && depends_on_choices(cond) {
        return is_reduction(&cm.change);
    }
    g.eval_cond(cond, ctx)
}

/// Whether an alternative cost with this condition may be chosen for `card` now
/// ("If you control a Swamp, you may pay 4 life rather than pay this spell's mana cost").
/// The condition is checked as the spell is proposed (CR 601.2b).
pub fn alternative_cost_allowed(g: &Game, p: PlayerId, card: ObjectId, s: &StaticAbility) -> bool {
    match &s.condition {
        None => true,
        Some(c) => g.eval_cond(c, &Ctx::new(Some(card), p)),
    }
}
