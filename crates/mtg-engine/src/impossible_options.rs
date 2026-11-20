//! CR 608.2d: a player who chooses among the options an effect offers as it resolves
//! "can't choose an option that's illegal or impossible" (drawing from an empty library
//! isn't impossible, CR 121.3). For `Effect::ChooseOne` ("you and defending player each
//! discard a card or sacrifice a permanent", Eumidian Wastewaker: "If one of those players
//! can't discard a card (such as by having no cards in hand), that player must sacrifice a
//! permanent."), the options a player can't carry out at all aren't offered while another
//! option can be. A villainous choice is the exception (CR 701.55b; `kwa/villainous.rs`
//! asks its own question).
//!
//! An option counts as impossible only when every instruction in it is one this module
//! knows to be impossible now — discarding with no card in hand that could be discarded,
//! sacrificing with no permanent that could be sacrificed — besides bookkeeping
//! (`Store`, `Noop`). Anything else is treated as possible.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;

/// Whether `e` is known to be impossible to perform now (see the module docs).
fn impossible(g: &Game, e: &Effect, ctx: &Ctx) -> bool {
    match e {
        Effect::Seq(v) => {
            let mut any = false;
            for x in v {
                match x {
                    Effect::Store { .. } | Effect::StoreValue { .. } | Effect::Noop => {}
                    x if impossible(g, x, ctx) => any = true,
                    _ => return false,
                }
            }
            any
        }
        Effect::AsPlayer { who, effect } => match g.eval_player(who, ctx) {
            Some(p) => {
                let mut c = ctx.clone();
                c.controller = p;
                impossible(g, effect, &c)
            }
            None => false,
        },
        Effect::Discard { who, n, filter, .. } => {
            if g.eval_value(n, ctx) <= 0 {
                return false;
            }
            g.eval_players(who, ctx).into_iter().all(|p| {
                !g.player(p)
                    .hand
                    .iter()
                    .any(|c| g.obj(*c).is_card() && g.matches(*c, filter, ctx))
            })
        }
        Effect::Sacrifice { who, filter, count } => {
            if g.eval_value(count, ctx) <= 0 {
                return false;
            }
            let cause = crate::rule_statics::sacrifice_causes::cause_of(ctx);
            g.eval_players(who, ctx).into_iter().all(|p| {
                let mut pctx = ctx.clone();
                pctx.iter_player = Some(p);
                !g.objects_matching(filter, &pctx)
                    .into_iter()
                    .any(|o| g.obj(o).controller == p && !g.sacrifice_forbidden(o, cause.as_ref()))
            })
        }
        _ => false,
    }
}

/// The indices of the options of an `Effect::ChooseOne` its chooser may choose now: those
/// not known to be impossible, or all of them if every one is (CR 608.2d; then the chosen
/// one does as much as it can, CR 609.3).
pub fn choosable(g: &Game, options: &[(String, Effect)], ctx: &Ctx) -> Vec<usize> {
    let ok: Vec<usize> = (0..options.len())
        .filter(|i| !impossible(g, &options[*i].1, ctx))
        .collect();
    if ok.is_empty() {
        (0..options.len()).collect()
    } else {
        ok
    }
}
