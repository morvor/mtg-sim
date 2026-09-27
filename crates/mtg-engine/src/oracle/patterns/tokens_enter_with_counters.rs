//! "Create a 0/0 green Ooze creature token with trample. The token enters with X +1/+1
//! counters on it, where X is the number of other creatures you control." (Printlifter
//! Ooze): the counters are put on the token as it's created, before anything else can
//! happen (CR 122.6: a permanent entering with counters has them put on it).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether the effect ends by creating tokens.
fn ends_with_token(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_token),
        _ => false,
    }
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
    let Some((counters, rest)) = r.split_once(" on it") else {
        return false;
    };
    let saved = b.it.clone();
    b.it = Sel::Var(vars::CREATED);
    let text = format!("put {counters} on it{rest}");
    let Some(mut e) = crate::oracle::effects::parse_sentence(&text, b) else {
        b.it = saved;
        return false;
    };
    // "where X is the number of other creatures you control": other than the token that
    // enters with them (which is on the battlefield as they're put on it here), whether or
    // not the source is still there.
    if let Effect::AddCounters {
        what: Sel::Var(v),
        n,
        ..
    } = &mut e
    {
        if *v == vars::CREATED {
            let Some(other_than_token) = other_than_created(n) else {
                b.it = saved;
                return false;
            };
            *n = other_than_token;
        }
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
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
