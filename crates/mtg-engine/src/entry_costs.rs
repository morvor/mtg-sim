//! Costs paid as replacement effects modify how permanents enter the battlefield
//! (CR 614.12a–b).
//!
//! When several objects enter the battlefield at the same time, the choices for the
//! replacement effects that modify how each of them enters are all made before any of the
//! life or energy those choices call for is paid: a player may not choose to pay costs
//! whose combined total couldn't be paid (CR 614.12b), and an effect that checks the game
//! state ("this land enters tapped unless a player has 13 or less life") sees it as it was
//! before those payments. They're then paid together, once every entering object's
//! replacement effects have been applied (see [`Game::move_objects`]).
//!
//! Other costs (returning, sacrificing, exiling something) are paid as they're chosen, so
//! a later choice sees what the earlier payments used up and the combined costs stay
//! payable.

use crate::ability::{Cost, CostPart, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::PlayerId;
use serde::{Deserialize, Serialize};

/// A cost a player chose to pay while a replacement effect modified how a permanent
/// entering the battlefield along with other objects enters, to be paid once all their
/// replacement effects have been applied.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntryPayment {
    pub player: PlayerId,
    pub cost: Cost,
    pub ctx: Ctx,
}

/// Whether a replacement effect is modifying how a permanent enters ([`Ctx::entering`])
/// while other objects enter the battlefield at the same time.
fn simultaneous(g: &Game, ctx: &Ctx) -> bool {
    ctx.entering.is_some() && g.entering.len() > 1
}

/// Whether a cost chosen now is paid only once all the entering objects' choices are made:
/// it's made only of life and energy payments, whose combined total is checked exactly.
fn deferred(g: &Game, cost: &Cost, ctx: &Ctx) -> bool {
    simultaneous(g, ctx)
        && cost.mana.is_none()
        && !cost.parts.is_empty()
        && cost
            .parts
            .iter()
            .all(|p| matches!(p, CostPart::PayLife(_) | CostPart::PayEnergy(_)))
}

/// `cost` together with the costs `p` already chose to pay for objects entering at the same
/// time (life and energy payments summed).
fn combined(g: &Game, p: PlayerId, cost: &Cost, ctx: &Ctx) -> Cost {
    let mut total = Cost::default();
    let (mut life, mut energy) = (0i32, 0i32);
    let pending = g
        .zones
        .entry_payments
        .iter()
        .filter(|e| e.player == p)
        .map(|e| (&e.cost, &e.ctx))
        .chain(std::iter::once((cost, ctx)));
    for (c, cx) in pending {
        for part in &c.parts {
            match part {
                CostPart::PayLife(v) => life += g.eval_value(v, cx).max(0) as i32,
                CostPart::PayEnergy(v) => energy += g.eval_value(v, cx).max(0) as i32,
                other => total.parts.push(other.clone()),
            }
        }
        crate::casting::add_cost(
            &mut total,
            &Cost {
                mana: c.mana.clone(),
                parts: vec![],
            },
        );
    }
    if life > 0 {
        total.parts.push(CostPart::PayLife(Value::Const(life)));
    }
    if energy > 0 {
        total.parts.push(CostPart::PayEnergy(Value::Const(energy)));
    }
    total
}

/// Whether `p` may choose to pay `cost` now (CR 614.12b: together with the costs already
/// chosen for objects entering at the same time).
pub fn can_pay(g: &Game, p: PlayerId, cost: &Cost, ctx: &Ctx) -> bool {
    if simultaneous(g, ctx) {
        g.can_pay_cost(p, &combined(g, p, cost, ctx), ctx.source, ctx)
    } else {
        g.can_pay_cost(p, cost, ctx.source, ctx)
    }
}

/// `p` pays `cost` — or, for life and energy while several objects enter the battlefield
/// at the same time, commits to paying it once all their replacement effects have been
/// applied. Returns whether it's paid.
pub fn pay(g: &mut Game, p: PlayerId, cost: &Cost, ctx: &Ctx) -> bool {
    if deferred(g, cost, ctx) {
        g.zones.entry_payments.push(EntryPayment {
            player: p,
            cost: cost.clone(),
            ctx: ctx.clone(),
        });
        return true;
    }
    g.pay_cost(p, cost, ctx.source, ctx)
}

/// Pays the costs chosen while applying the replacement effects of objects that entered the
/// battlefield at the same time.
pub fn pay_deferred(g: &mut Game, payments: Vec<EntryPayment>) {
    for e in payments {
        let mut ctx = e.ctx;
        ctx.entering = None;
        g.pay_cost(e.player, &e.cost, ctx.source, &ctx);
    }
}
