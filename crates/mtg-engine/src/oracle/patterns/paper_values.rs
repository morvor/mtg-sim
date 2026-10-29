//! Values and conditions of older Oracle wordings:
//!
//! - "there are no more [kind] counters on it" (All Hallow's Eve): none left;
//! - two damage amounts of one source in one sentence: "~ deals half X damage, rounded
//!   down, to any target, and half X damage, rounded up, to you." (Banshee);
//! - a leading condition that introduces the spell's target: "If target creature has
//!   toughness 5 or greater, it gets +4/-4 until end of turn. Otherwise, ..." (Blood Lust:
//!   the target is chosen as the spell is cast whatever the condition, CR 601.2c; the
//!   condition is checked as it resolves, as in "Target creature gets +4/-4 until end of
//!   turn if it has toughness 5 or greater");
//! - an optional payment whose amount is counted: "You may pay {2} for each wage counter
//!   on it. If you don't, ..." (Rogue Skycaptain: all of it or none, its ruling).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "if there are no more [kind] counters on [object], [effect]": "if there are no [kind]
/// counters on [object], [effect]".
fn no_more_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("if there are no more ")?;
    let (c, _) = r.split_once(", ")?;
    if !c.contains(" counters on ") {
        return None;
    }
    crate::oracle::effects::parse_sentence(&format!("if there are no {r}"), b)
}

inventory::submit! { EffectPattern { name: "if there are no more [kind] counters on it", priority: 300, parse: no_more_counters } }

/// "~ deals [amount] to [recipient], and [amount] to [recipient]": two instructions of the
/// same source.
fn deals_and_deals(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("~ deals ")?;
    let (first, second) = r.split_once(", and ")?;
    if !(second.contains(" damage") && second.contains(" to ")) || second.contains(" deals ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let a = parse_clause(&format!("~ deals {first}"), b);
    let c = a
        .as_ref()
        .and_then(|_| parse_clause(&format!("~ deals {second}"), b));
    match (a, c) {
        (Some(a), Some(c))
            if matches!(a, Effect::DealDamage { .. }) && matches!(c, Effect::DealDamage { .. }) =>
        {
            Some(Effect::Seq(vec![a, c]))
        }
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "~ deals [amount] to [target], and [amount] to [player]", priority: 300, parse: deals_and_deals } }

/// "if target [object] has|is [quality], it [instruction]": "target [object]
/// [instruction] if it has|is [quality]".
fn leading_if_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("if target ")?;
    let (cond, rest) = r.split_once(", it ")?;
    let (noun, verb, quality) = [" has ", " is "]
        .iter()
        .find_map(|v| cond.split_once(v).map(|(n, q)| (n, *v, q)))?;
    // One noun ("creature", "artifact creature"), not a phrase with its own verb.
    if noun.contains(' ') && parse_object_phrase(noun).is_none_or(|(_, _, t)| !end(t).is_empty()) {
        return None;
    }
    let text = format!("target {noun} {rest} if it{verb}{quality}");
    let e = parse_clause(&text, b)?;
    matches!(e, Effect::If { .. }).then_some(e)
}

inventory::submit! { EffectPattern { name: "if target [object] has [quality], it [instruction]", priority: 300, parse: leading_if_target } }

/// "you may pay [cost] for each [counted thing]": the total is determined as the
/// instruction is performed and paid all at once or not at all.
fn may_pay_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let (c, each) = r.split_once(" for each ")?;
    let cost = super::counters_resources_pay::resolution_cost(c)?;
    let it = b.it.clone();
    let times = super::statics::parse_for_each(each, Some(&it))?;
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost::free().with(CostPart::Repeated {
            cost: Box::new(cost),
            times,
        }),
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "you may pay [cost] for each [counted thing]", priority: 300, parse: may_pay_for_each } }
