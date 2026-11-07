//! "Whenever a creature enters, if there are two or more other creatures on the
//! battlefield, exile that creature" (Portcullis): in the intervening "if" of a trigger
//! about another object, "other creatures" can't mean creatures other than the source
//! when the source is no creature at all (an artifact counting creatures). The only
//! object the "other" can set apart is the one the ability triggered for: "two or more
//! creatures besides that creature". Whether the source can have the type is read from
//! its printed type line, so an animated Portcullis still counts itself (interpretation
//! `other-than-trigger-object`, `docs/interpretations/triggers-durations.md`). The same
//! holds for subtypes: "Whenever a Mountain you control enters, if you control at least
//! five other Mountains" (Valakut, the Molten Pinnacle, a land that isn't a Mountain)
//! counts the Mountains besides the one that entered.

use crate::ability::{Condition, Filter, Sel, TriggerCond, TriggeredAbility, Value};
use crate::oracle::CompileContext;
use crate::types::{CardType, Subtype};

/// A card type or subtype a filter requires.
#[derive(Clone, PartialEq)]
enum Kind {
    Type(CardType),
    Sub(Subtype),
}

impl Kind {
    fn of(f: &Filter) -> Option<Kind> {
        match f {
            Filter::Type(t) => Some(Kind::Type(*t)),
            Filter::Subtype(s) => Some(Kind::Sub(s.clone())),
            _ => None,
        }
    }
    /// Whether the source's printed type line has it.
    fn printed(&self, ctx: &CompileContext) -> bool {
        match self {
            Kind::Type(t) => ctx.type_line.card_types.contains(*t),
            Kind::Sub(s) => ctx.type_line.subtypes.contains(s),
        }
    }
}

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

/// The card types and subtypes the triggering object is required to have: the object
/// filter of a trigger about "a [type]" other than the source.
fn trigger_subject(trigger: &TriggerCond) -> Option<Vec<Kind>> {
    let f = match trigger {
        TriggerCond::EntersBattlefield(f)
        | TriggerCond::LeavesBattlefield(f)
        | TriggerCond::Dies(f) => f,
        _ => return None,
    };
    let types: Vec<Kind> = conjuncts(f).iter().filter_map(Kind::of).collect();
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

fn rebind_in_condition(c: &mut Condition, subject: &[Kind], ctx: &CompileContext) {
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

fn rebind_in_value(v: &mut Value, subject: &[Kind], ctx: &CompileContext) {
    if let Value::Count(f) = v {
        rebind_in_filter(f, subject, ctx);
    }
}

/// "other [type]s" where the source never has that type and the triggering object must:
/// other than the triggering object.
fn rebind_in_filter(f: &mut Filter, subject: &[Kind], ctx: &CompileContext) {
    let Filter::And(v) = f else {
        return;
    };
    let counted: Vec<Kind> = v.iter().filter_map(Kind::of).collect();
    let applies = !counted.is_empty()
        && counted.iter().all(|t| subject.contains(t))
        && counted.iter().any(|t| !t.printed(ctx));
    if !applies {
        return;
    }
    for c in v.iter_mut() {
        if matches!(c, Filter::Other) {
            *c = Filter::Not(Box::new(Filter::In(Box::new(Sel::TriggerObject))));
        }
    }
}
