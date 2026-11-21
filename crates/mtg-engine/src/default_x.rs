//! Which values of X a spell being cast could be cast with (CR 601.2b, 601.2c, 601.2h): X
//! is announced with the costs and targets it implies, and the spell must then have enough
//! legal targets for that X ("Tap X target creatures") and a total cost that could be paid
//! with it, the parts that use X included ("Discard X cards", "Sacrifice X Mountains",
//! "Remove X loyalty counters from among planeswalkers you control"). The default X (and
//! the greatest X a player is offered) is the greatest such value (CLAUDE.md: a default
//! answer falls back to a legal choice).

use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// Whether `n` could be announced as X for the spell `id` that `p` is casting with `opt`,
/// with the additional costs `extra` announced so far: an optimistic check (the cost could
/// be paid, the targets could be chosen), never ruling out a value that could work.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spell_x_feasible(
    g: &Game,
    p: PlayerId,
    id: ObjectId,
    chars: &Characteristics,
    opt: &CastOption,
    extra: &Cost,
    cast: &CastInfo,
    n: i64,
) -> bool {
    let mut ctx = Ctx::new(Some(id), p);
    ctx.x = n as i32;
    ctx.cast = Some(cast.clone());
    let cost = g.total_cost_with(p, id, chars, opt, n.max(0) as u32, extra);
    let mut pay = ctx.clone();
    pay.cost_of = Some(crate::rule_statics::payment::CostOf::Spell);
    if !g.can_pay_cost_optimistic_in(p, &cost, Some(id), chars, &pay) {
        return false;
    }
    // A face-down spell has no targets (CR 708.2).
    if g.obj(id).face_down {
        return true;
    }
    let body = g.spell_body(id);
    match &body.modal {
        None => g.targets_possible(&body.targets, &ctx, id),
        // As many modes with legal targets as it must have (CR 700.2a).
        Some(modal) => {
            let min = g.eval_value(&modal.min, &ctx).max(0) as usize;
            let needed = if modal.allow_repeat { min.min(1) } else { min };
            modal.optional
                || modal
                    .modes
                    .iter()
                    .filter(|m| g.targets_possible(&m.targets, &ctx, id))
                    .count()
                    >= needed
        }
    }
}
