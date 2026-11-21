//! "Create a 0/0 green Ooze creature token with trample. The token enters with X +1/+1
//! counters on it, where X is the number of other creatures you control." (Printlifter
//! Ooze): the token enters with the counters (CR 122.6), so abilities that trigger on it
//! entering see them ([`Effect::TokensEnterWithCounters`]).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CounterKind;

/// Whether the effect ends by creating tokens.
fn ends_with_token(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. }
        | Effect::CreateTokenWithPT { .. }
        | Effect::CreateTokenCopy { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_token),
        Effect::DelayedTrigger { body, .. } => ends_with_token(&body.effect),
        _ => false,
    }
}

/// The object the last token copy `e` creates copies.
fn last_copied(e: &Effect) -> Option<Sel> {
    match e {
        Effect::CreateTokenCopy { of, .. } => Some(of.clone()),
        Effect::Seq(v) => v.last().and_then(last_copied),
        Effect::DelayedTrigger { body, .. } => last_copied(&body.effect),
        _ => None,
    }
}

/// "The token enters with half that many +1/+1 counters on it, rounded down." (Ochre
/// Jelly): half the counters of that kind the object the token copies had as it last
/// existed before it died (CR 400.7).
fn half_that_many(r: &str, prev: &Effect) -> Option<Vec<(CounterKind, Value)>> {
    let r = r.strip_prefix("half that many ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counters on it, rounded ")?;
    let up = match r {
        "up" => true,
        "down" => false,
        _ => return None,
    };
    let of = last_copied(prev)?;
    Some(vec![(
        kind.clone(),
        Value::Div(
            Box::new(Value::CountersOn(
                Box::new(Sel::Before(Box::new(of))),
                Some(kind),
            )),
            2,
            up,
        ),
    )])
}

fn token_enters_with_counters(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l)
        .strip_prefix("the token enters with ")
        .or_else(|| end(l).strip_prefix("that token enters with "))
    else {
        return false;
    };
    if !ends_with_token(prev) {
        return false;
    }
    if let Some(c) = half_that_many(r, prev) {
        return wrap_last_create(prev, c);
    }
    let Some((counters, rest)) = r.split_once(" on it") else {
        return false;
    };
    let saved = b.it.clone();
    b.it = Sel::Var(vars::CREATED);
    let text = format!("put {counters} on it{rest}");
    let parsed = crate::oracle::effects::parse_sentence(&text, b);
    // "where X is the number of other creatures you control": other than the token that
    // enters with them, whether or not the source is still there.
    let Some(Effect::AddCounters {
        what: Sel::Var(vars::CREATED),
        kind,
        n,
    }) = parsed
    else {
        b.it = saved;
        return false;
    };
    let Some(n) = other_than_created(&n) else {
        b.it = saved;
        return false;
    };
    wrap_last_create(prev, vec![(kind, n)])
}

/// Makes the tokens the last instruction of `e` creates enter with `counters`.
fn wrap_last_create(e: &mut Effect, counters: Vec<(CounterKind, Value)>) -> bool {
    match e {
        Effect::CreateToken { .. }
        | Effect::CreateTokenWithPT { .. }
        | Effect::CreateTokenCopy { .. } => {
            let create = std::mem::take(e);
            *e = Effect::TokensEnterWithCounters {
                counters,
                effect: Box::new(create),
            };
            true
        }
        Effect::Seq(v) => v
            .last_mut()
            .is_some_and(|last| wrap_last_create(last, counters)),
        Effect::DelayedTrigger { body, .. } => wrap_last_create(&mut body.effect, counters),
        _ => false,
    }
}

/// `n` with "other" (than the source) meaning other than the created token.
fn other_than_created(n: &Value) -> Option<Value> {
    fn replace(j: &mut serde_json::Value, with: &serde_json::Value) {
        match j {
            serde_json::Value::String(s) if s == "Other" => *j = with.clone(),
            serde_json::Value::Array(v) => v.iter_mut().for_each(|x| replace(x, with)),
            serde_json::Value::Object(m) => m.values_mut().for_each(|x| replace(x, with)),
            _ => {}
        }
    }
    let with = serde_json::to_value(Filter::Not(Box::new(Filter::In(Box::new(Sel::Var(
        vars::CREATED,
    ))))))
    .ok()?;
    let mut j = serde_json::to_value(n).ok()?;
    replace(&mut j, &with);
    serde_json::from_value(j).ok()
}

inventory::submit! { FollowupPattern { name: "tokens: the token enters with N counters on it", priority: 90, apply: token_enters_with_counters } }
