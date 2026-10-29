//! "For as long as" durations about objects the resolving spell or ability names (CR
//! 611.2b): "~ becomes a copy of that creature for as long as that creature remains
//! tapped" (Zygon Infiltrator), "gain control of target creature ... for as long as ~
//! remains tapped and that creature's power remains less than or equal to ~'s power" (Old
//! Man of the Sea).
//!
//! A continuous effect keeps no targets of its own, so the objects a duration's condition
//! names (a target, the triggering object, an object chosen or affected earlier in the
//! resolution) are fixed as the effect begins: each reference becomes the set of those
//! objects. An object that later leaves its zone is a new object (CR 400.7), which the
//! condition no longer sees ("remains tapped" is then false).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;

/// Whether `s` is a reference to objects the resolution names (and so needs fixing).
fn is_referent(s: &Sel) -> bool {
    match s {
        Sel::Target(_) | Sel::TriggerObject | Sel::TriggerOtherObject | Sel::TriggerObjects => true,
        Sel::Var(v) => *v != vars::AFFECTED,
        _ => false,
    }
}

fn bind_sel(g: &mut Game, s: &Sel, ctx: &mut Ctx) -> Sel {
    if is_referent(s) {
        Sel::All(Filter::Objects(g.resolve_objects(s, ctx)))
    } else {
        s.clone()
    }
}

fn bind_value(g: &mut Game, v: &Value, ctx: &mut Ctx) -> Value {
    match v {
        Value::PowerOf(s) => Value::PowerOf(Box::new(bind_sel(g, s, ctx))),
        Value::ToughnessOf(s) => Value::ToughnessOf(Box::new(bind_sel(g, s, ctx))),
        Value::ManaValueOf(s) => Value::ManaValueOf(Box::new(bind_sel(g, s, ctx))),
        v => v.clone(),
    }
}

/// A filter naming such objects ("attached to that creature").
fn bind_filter(g: &mut Game, f: &Filter, ctx: &mut Ctx) -> Filter {
    match f {
        Filter::AttachedTo(s) => Filter::AttachedTo(Box::new(bind_sel(g, s, ctx))),
        Filter::And(v) => Filter::And(v.iter().map(|f| bind_filter(g, f, ctx)).collect()),
        f => f.clone(),
    }
}

fn bind_cond(g: &mut Game, c: &Condition, ctx: &mut Ctx) -> Condition {
    match c {
        Condition::And(v) => Condition::And(v.iter().map(|c| bind_cond(g, c, ctx)).collect()),
        Condition::Or(v) => Condition::Or(v.iter().map(|c| bind_cond(g, c, ctx)).collect()),
        Condition::Not(c) => Condition::Not(Box::new(bind_cond(g, c, ctx))),
        Condition::SelMatches(s, f) => {
            Condition::SelMatches(bind_sel(g, s, ctx), bind_filter(g, f, ctx))
        }
        Condition::SelNonEmpty(s) => Condition::SelNonEmpty(bind_sel(g, s, ctx)),
        Condition::Compare(a, cmp, b) => {
            Condition::Compare(bind_value(g, a, ctx), *cmp, bind_value(g, b, ctx))
        }
        c => c.clone(),
    }
}

/// The duration `d` with the objects its condition names fixed as the effect begins. Other
/// durations are returned unchanged.
pub fn bind_duration_objects(g: &mut Game, d: &Duration, ctx: &mut Ctx) -> Duration {
    match d {
        Duration::WhileCondition(c) => Duration::WhileCondition(bind_cond(g, c, ctx)),
        d => d.clone(),
    }
}
