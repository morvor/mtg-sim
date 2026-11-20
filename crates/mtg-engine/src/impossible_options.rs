//! CR 608.2d: a player who chooses among the options an effect offers as it resolves
//! "can't choose an option that's illegal or impossible" (drawing from an empty library
//! isn't impossible, CR 121.3). For `Effect::ChooseOne` ("you and defending player each
//! discard a card or sacrifice a permanent", Eumidian Wastewaker: "If one of those players
//! can't discard a card (such as by having no cards in hand), that player must sacrifice a
//! permanent."), the options a player can't carry out at all aren't offered while another
//! option can be. A villainous choice is the exception (CR 701.55b; `kwa/villainous.rs`
//! asks its own question).
//!
//! An option counts as impossible only when its words are the instruction itself (not a word
//! such as "self" whose consequences follow it, which can always be chosen: Feed the
//! Machine's ruling) and every instruction in it is one this module
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

/// Whether the option's words are the instruction itself ("discard a card", "sacrifice a
/// permanent"), not a word whose consequences the text gives afterward ("chooses self or
/// others. If that player chooses self, ...", Feed the Machine: "The targeted player may
/// choose “self” even if they can’t perform the resulting action."). Only such an option
/// is an impossible choice when its instruction can't be done.
fn names_its_instruction(label: &str) -> bool {
    let first = label.split_whitespace().next().unwrap_or("").to_lowercase();
    matches!(
        first.as_str(),
        "discard" | "discards" | "sacrifice" | "sacrifices"
    )
}

/// The indices of the options of an `Effect::ChooseOne` its chooser may choose now: those
/// not known to be impossible, or all of them if every one is (CR 608.2d; then the chosen
/// one does as much as it can, CR 609.3).
pub fn choosable(g: &Game, options: &[(String, Effect)], ctx: &Ctx) -> Vec<usize> {
    let ok: Vec<usize> = (0..options.len())
        .filter(|i| {
            let (label, e) = &options[*i];
            !(names_its_instruction(label) && impossible(g, e, ctx))
        })
        .collect();
    if ok.is_empty() {
        (0..options.len()).collect()
    } else {
        ok
    }
}

/// "Destroy the creature with the least power. [...] If two or more creatures are tied for
/// least power, you choose one of them." (Drop of Honey, Porphyry Nodes): when a player
/// chooses as the destruction is performed which permanents it destroys (`Sel::Choose`),
/// only permanents that can be destroyed may be chosen: not one with indestructible
/// (CR 702.12b). With none, nothing is destroyed (their rulings: "If one of the creatures
/// with the least power has indestructible, you must destroy one without indestructible. If
/// all creatures with the least power have indestructible, you don't destroy any
/// creature."; interpretation `chosen-permanents-to-destroy-can-be-destroyed`). Any other
/// selection is returned unchanged.
pub fn destroyable_choice(what: &Sel) -> std::borrow::Cow<'_, Sel> {
    match what {
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            store,
        } => std::borrow::Cow::Owned(Sel::Choose {
            chooser: chooser.clone(),
            filter: Filter::and(vec![
                filter.clone(),
                Filter::not(Filter::HasKeyword(
                    crate::keywords::KeywordKind::Indestructible,
                )),
            ]),
            count: count.clone(),
            up_to: *up_to,
            store: store.clone(),
        }),
        _ => std::borrow::Cow::Borrowed(what),
    }
}
