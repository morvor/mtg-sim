//! [`crate::ability::Value::OfChosen`]: "the [characteristic] of a [object]" naming one object picked from
//! several ("the mana value of a commander you own on the battlefield or in the command
//! zone", Stinging Study). Its controller chooses the object as the effect resolves (CR
//! 608.2d: "If an effect of a spell or ability offers any choices other than choices
//! already made as part of casting the spell, activating the ability, or otherwise putting
//! the spell or ability on the stack, the player announces these while applying the
//! effect."; with two commanders, CR 702.124e: "If an effect refers to your commander
//! while you have two commanders, it refers to either one."), and the value is a
//! characteristic of that object. Stinging
//! Study's ruling (2021-04-16): "If you own multiple commanders on the battlefield and/or in
//! the command zone, you choose one of them as Stinging Study resolves to determine the
//! value of X." Decided by the user, Nov 10 (interpretation `a-commander-you-own-is-chosen`).
//!
//! Values are evaluated without changing the game ([`Game::eval_value`]), so the choice is
//! made first, by the instruction that defines X ("where X is ...", the shape every such
//! card has: [`crate::ability::Effect::StoreValue`], [`crate::ability::Effect::SetX`]), which calls [`choose`]; the chosen
//! object is kept in the resolution's context under the value's `store`. Evaluated before
//! the choice is made (a preview, or a value no instruction chose for), the value is that
//! of the first candidate, the default answer (policy `default-first-listed-choice`).

use crate::ability::{Filter, Value, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// The value of `of` for the object chosen for `store`, or for the first candidate while
/// none is chosen; 0 with no candidate.
pub fn eval(g: &Game, filter: &Filter, of: &Value, store: Var, ctx: &Ctx) -> i64 {
    let chosen = ctx
        .vars
        .get(&store)
        .and_then(|v| v.iter().find_map(|e| e.object()))
        .or_else(|| g.objects_matching(filter, ctx).first().copied());
    chosen.map_or(0, |o| g.eval_value(of, &crate::relational::testing(ctx, o)))
}

/// Makes the choices of the [`crate::ability::Value::OfChosen`] values in `v` that aren't made yet: the
/// controller chooses one of the objects each value's filter matches now (the answer is
/// validated by the choice itself; the default is the first candidate), and the choice is
/// kept in `ctx.vars` under the value's `store`. A value with no candidate gets no choice
/// (it evaluates to 0).
pub fn choose(g: &mut Game, v: &Value, ctx: &mut Ctx) {
    for (filter, store) in of_chosen(v) {
        if ctx.vars.contains_key(&store) {
            continue;
        }
        let cands = g.objects_matching(&filter, ctx);
        if cands.is_empty() {
            continue;
        }
        let picked = crate::zones::choose_objects(
            g,
            ctx.controller,
            ctx.source,
            "Choose the object whose characteristic is used",
            cands,
            1,
            1,
        );
        ctx.vars
            .insert(store, picked.into_iter().map(Entity::Object).collect());
    }
}

/// The filters and stores of the `OfChosen` values anywhere in `v`.
fn of_chosen(v: &Value) -> Vec<(Filter, Var)> {
    fn walk(j: &serde_json::Value, out: &mut Vec<(Filter, Var)>) {
        match j {
            serde_json::Value::Object(m) => {
                if let Some(c) = m.get("OfChosen") {
                    if let (Some(f), Some(s)) =
                        (c.get("filter"), c.get("store").and_then(|s| s.as_u64()))
                    {
                        if let Ok(f) = serde_json::from_value::<Filter>(f.clone()) {
                            out.push((f, s as Var));
                        }
                    }
                }
                m.values().for_each(|x| walk(x, out));
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let Ok(j) = serde_json::to_value(v) else {
        return vec![];
    };
    let mut out = Vec::new();
    walk(&j, &mut out);
    out
}
