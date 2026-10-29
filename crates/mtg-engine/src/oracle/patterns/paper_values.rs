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
    // An object ("it"), not a player.
    if matches!(noun, "player" | "opponent") {
        return None;
    }
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

/// "if [object]'s mana value is less than or equal to [value], [instruction]" (Transmute
/// Artifact: "If that card's mana value is less than or equal to the sacrificed
/// artifact's mana value, put it onto the battlefield."): a comparison of two values.
fn mana_value_compares(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("if ")?;
    let (c, x) = r.split_once(", ")?;
    let (obj, rhs) = c.split_once("'s mana value is ")?;
    let (cmp, v) = [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
        ("equal to ", Cmp::Eq),
    ]
    .iter()
    .find_map(|(p, cmp)| rhs.strip_prefix(p).map(|v| (*cmp, v)))?;
    // Only a value phrase (a number is the core grammar's).
    if parse_number(v).is_some() {
        return None;
    }
    let first = b.targets.len();
    let (sel, tail) = crate::oracle::effects::object_ref(obj, b)?;
    if !tail.trim().is_empty() || b.targets.len() != first || matches!(sel, Sel::This) {
        b.targets.truncate(first);
        return None;
    }
    let (rv, tail) = crate::oracle::statics::parse_value_phrase(v, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    // "it" in the instruction is the object compared.
    b.it = sel.clone();
    let e = parse_clause(x, b)?;
    Some(Effect::If {
        cond: Condition::Compare(Value::ManaValueOf(Box::new(sel)), cmp, rv),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

/// As a sentence after the instruction that found the object (read before the leading-if
/// grammar, which has no referent for "that card" after a search inside "If you do").
fn if_mana_value_compares(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(e) = mana_value_compares(l, b) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "if [object]'s mana value is less than or equal to [value]", priority: 50, apply: if_mana_value_compares } }

/// The last instruction of an effect.
fn last_mut(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.last_mut(),
        e => Some(e),
    }
}

/// "If it's greater, you may pay {X}, where X is the difference." after "If [a] is less
/// than or equal to [b], ...": otherwise, a payment of the difference (Transmute
/// Artifact's ruling: you pay as the spell resolves).
fn if_greater_pay_difference(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "if it's greater, you may pay {x}, where x is the difference" {
        return false;
    }
    let Some(Effect::If {
        cond: Condition::Compare(a, Cmp::Le, c),
        otherwise,
        ..
    }) = last_mut(prev)
    else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) || !matches!(a, Value::ManaValueOf(_)) {
        return false;
    }
    let Some(one) = crate::mana::ManaCost::parse("{1}") else {
        return false;
    };
    // {X} where X is the difference: that much generic mana (CR 107.1b: a negative
    // difference is 0).
    let diff = Value::Max(
        Box::new(Value::Diff(Box::new(a.clone()), Box::new(c.clone()))),
        Box::new(Value::c(0)),
    );
    **otherwise = Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost::free().with(CostPart::Repeated {
            cost: Box::new(Cost::mana(one)),
            times: diff,
        }),
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "if it's greater, you may pay {X}, where X is the difference", priority: 50, apply: if_greater_pay_difference } }

/// The payment of the difference above, if `e` is that "if ... less than or equal to"
/// instruction.
fn difference_payment(e: &mut Effect) -> Option<&mut Effect> {
    let Effect::If { otherwise, .. } = e else {
        return None;
    };
    match &mut **otherwise {
        p @ Effect::PayOptional { .. } => Some(p),
        _ => None,
    }
}

/// "If you do, [instruction]." / "If you don't, [instruction]." after that payment.
fn if_paid_difference(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (did, r) = if let Some(r) = l.strip_prefix("if you do, ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("if you don't, ") {
        (false, r)
    } else {
        return false;
    };
    let Some(Effect::PayOptional {
        then, otherwise, ..
    }) = last_mut(prev).and_then(difference_payment)
    else {
        return false;
    };
    let slot = if did { then } else { otherwise };
    if !matches!(**slot, Effect::Noop) {
        return false;
    }
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    **slot = e;
    true
}

inventory::submit! { super::FollowupPattern { name: "if you do / don't, after paying the difference", priority: 50, apply: if_paid_difference } }
