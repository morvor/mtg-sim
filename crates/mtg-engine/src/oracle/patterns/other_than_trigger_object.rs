//! "Whenever a creature enters, if there are two or more other creatures on the
//! battlefield, exile that creature" (Portcullis): in the intervening "if" of a trigger
//! about another object, "other creatures" can't mean creatures other than the source
//! when the source is no creature at all (an artifact counting creatures). The only
//! object the "other" can set apart is the one the ability triggered for: "two or more
//! creatures besides that creature". Whether the source can have the type is read from
//! its printed type line, so an animated Portcullis still counts itself (interpretation
//! `other-than-trigger-object`, `docs/interpretations/triggers-durations.md`).

use crate::ability::{Condition, Filter, Sel, TriggerCond, TriggeredAbility, Value};
use crate::oracle::CompileContext;
use crate::types::CardType;

/// Rebinds "other" in the trigger's intervening "if" to "other than the triggering
/// object" where it can't be about the source (see the module docs).
pub fn rebind_other(tr: &mut TriggeredAbility, ctx: &CompileContext) {
    let Some(subject) = trigger_subject(&tr.trigger) else {
        return;
    };
    let Some(cond) = tr.intervening_if.as_mut() else {
        return;
    };
    rebind_in_condition(cond, &subject, ctx);
}

/// The card types the triggering object is required to have: the object filter of a
/// trigger about "a [type]" other than the source.
fn trigger_subject(trigger: &TriggerCond) -> Option<Vec<CardType>> {
    let f = match trigger {
        TriggerCond::EntersBattlefield(f)
        | TriggerCond::LeavesBattlefield(f)
        | TriggerCond::Dies(f) => f,
        _ => return None,
    };
    let types: Vec<CardType> = conjuncts(f)
        .iter()
        .filter_map(|c| match c {
            Filter::Type(t) => Some(*t),
            _ => None,
        })
        .collect();
    let about_source = conjuncts(f)
        .iter()
        .any(|c| matches!(c, Filter::Source | Filter::Other));
    (!types.is_empty() && !about_source).then_some(types)
}

fn conjuncts(f: &Filter) -> &[Filter] {
    match f {
        Filter::And(v) => v,
        other => std::slice::from_ref(other),
    }
}

fn rebind_in_condition(c: &mut Condition, subject: &[CardType], ctx: &CompileContext) {
    match c {
        Condition::Not(x) => rebind_in_condition(x, subject, ctx),
        Condition::And(v) | Condition::Or(v) => {
            for x in v {
                rebind_in_condition(x, subject, ctx);
            }
        }
        Condition::Compare(a, _, b) => {
            rebind_in_value(a, subject, ctx);
            rebind_in_value(b, subject, ctx);
        }
        Condition::Exists(f) => rebind_in_filter(f, subject, ctx),
        _ => {}
    }
}

fn rebind_in_value(v: &mut Value, subject: &[CardType], ctx: &CompileContext) {
    if let Value::Count(f) = v {
        rebind_in_filter(f, subject, ctx);
    }
}

/// "other [type]s" where the source never has that type and the triggering object must:
/// other than the triggering object.
fn rebind_in_filter(f: &mut Filter, subject: &[CardType], ctx: &CompileContext) {
    let Filter::And(v) = f else {
        return;
    };
    let counted: Vec<CardType> = v
        .iter()
        .filter_map(|c| match c {
            Filter::Type(t) => Some(*t),
            _ => None,
        })
        .collect();
    let applies = !counted.is_empty()
        && counted.iter().all(|t| subject.contains(t))
        && counted
            .iter()
            .any(|t| !ctx.type_line.card_types.contains(*t));
    if !applies {
        return;
    }
    for c in v.iter_mut() {
        if matches!(c, Filter::Other) {
            *c = Filter::Not(Box::new(Filter::In(Box::new(Sel::TriggerObject))));
        }
    }
}
