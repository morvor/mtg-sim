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
    let Some(e) = crate::oracle::effects::parse_sentence(&text, b) else {
        b.it = saved;
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "tokens: the token enters with N counters on it", priority: 90, apply: token_enters_with_counters } }
