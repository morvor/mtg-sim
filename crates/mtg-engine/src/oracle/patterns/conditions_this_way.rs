//! Conditions about the outcome of an earlier instruction of the same effect (CR 608.2c):
//!
//! - "If you do, [effect]" after an instruction that a condition governs ("If that
//!   creature was a Cleric, you may draw a card. If you do, you lose 1 life."): the
//!   follow-up belongs to the conditional part (whether "you do" is about the optional
//!   instruction, which didn't happen if the condition didn't hold).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether an effect ends with an optional instruction ("you may ...", "you may pay").
fn ends_optional(e: &Effect) -> bool {
    match e {
        Effect::May { .. } | Effect::PayOptional { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_optional),
        _ => false,
    }
}

/// "If you do, [effect]." right after "If [condition], you may [instruction].": nested in
/// the conditional part.
fn if_you_do_after_conditional_may(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (neg, r) = if let Some(r) = l.strip_prefix("if you do, ") {
        (false, r)
    } else if let Some(r) = l.strip_prefix("if you don't, ") {
        (true, r)
    } else {
        return false;
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::If {
        then, otherwise, ..
    }) = last
    else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) || !ends_optional(then) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(r, b) else {
        return false;
    };
    let cond = if neg {
        Condition::Not(Box::new(Condition::PrevHappened))
    } else {
        Condition::PrevHappened
    };
    let inner = std::mem::replace(&mut **then, Effect::Noop);
    **then = Effect::seq(vec![
        inner,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if you do, after if [condition], you may", priority: 150, apply: if_you_do_after_conditional_may } }
