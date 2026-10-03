//! X in a non-mana cost paid while a spell or ability resolves ("you may tap X untapped
//! Myr you control. If you do, ..." — Myr Battlesphere): if nothing defined X yet, the
//! player chooses its value as they pay (CR 107.1c, 608.2d), no greater than they could
//! pay. The value is then X for the rest of the resolution ("gets +X/+0", "deals X
//! damage").

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;

/// Asks the controller for X if `cost` (with no {X} in its mana part) uses an undefined X
/// in the number of objects one of its parts taps.
pub fn choose_x_for_optional_cost(g: &mut Game, cost: &Cost, ctx: &mut Ctx) {
    if ctx.x_defined
        || ctx
            .stack_obj
            .and_then(|s| g.try_obj(s))
            .and_then(|o| o.stack.as_ref())
            .is_some_and(|si| si.x.is_some())
    {
        return;
    }
    let p = ctx.controller;
    let mut max: Option<i64> = None;
    for part in &cost.parts {
        let CostPart::TapUntapped { filter, count } = part else {
            continue;
        };
        if !matches!(count, Value::X) || crate::casting::filter_mentions_x(filter) {
            continue;
        }
        // Summoning sickness doesn't matter: this isn't {T} (CR 302.6).
        let n = g
            .objects_matching(filter, ctx)
            .into_iter()
            .filter(|o| {
                let ob = g.obj(*o);
                ob.controller == p && !ob.tapped
            })
            .count() as i64;
        max = Some(max.map_or(n, |m| m.min(n)));
    }
    let Some(max) = max else {
        return;
    };
    let choice = crate::decision::Decision::ChooseX {
        source: ctx.source.or(ctx.stack_obj).unwrap_or(crate::types::ObjectId(0)),
        min: 0,
        max,
    };
    ctx.x = match g.ask(p, choice) {
        crate::decision::Answer::Number(n) if (0..=max).contains(&n) => n as i32,
        crate::decision::Answer::Number(n) if n > max => max as i32,
        _ => 0,
    };
    ctx.x_defined = true;
}
