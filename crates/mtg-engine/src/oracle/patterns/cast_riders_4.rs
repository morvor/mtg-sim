//! Riders on a spell cast as an effect resolves (cards3-cast-zones), after "you may cast
//! [it]" (CR 608.2g):
//!
//! * "If you cast a spell this way, put a +1/+1 counter on ~." (Helmut Zemo, Mastermind):
//!   done, as part of the effect, if a spell was cast;
//! * "When you cast that spell, ~ gets +X/+0 until end of turn, where X is that spell's
//!   mana value." (Ogre Battlecaster): a reflexive triggered ability (CR 603.12) that
//!   triggers when the spell is cast, about that spell (CR 400.7h).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Whether the effect ends by casting a card as it resolves (perhaps followed by a
/// replacement effect for the spell cast: "If that spell would be put into your graveyard,
/// exile it instead.").
fn ends_casting(e: &Effect) -> bool {
    match e {
        Effect::CastCard { .. } => true,
        Effect::WithPlayTerms { effect, .. } => ends_casting(effect),
        // "... if it's a spell with mana value less than ..." (not cast otherwise).
        Effect::If {
            then, otherwise, ..
        } => matches!(**otherwise, Effect::Noop) && ends_casting(then),
        Effect::Seq(v) => match v.last() {
            Some(Effect::AddReplacement { .. }) => {
                v.len() >= 2 && ends_casting(&v[v.len() - 2])
            }
            Some(last) => ends_casting(last),
            None => false,
        },
        _ => false,
    }
}

fn f_if_cast_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l).trim();
    let Some(r) = l
        .strip_prefix("if you cast a spell this way, ")
        .or_else(|| l.strip_prefix("if you cast it this way, "))
    else {
        return false;
    };
    if !ends_casting(prev) {
        return false;
    }
    let n0 = b.targets.len();
    let Some(e) = parse_sentence(r, b) else {
        return false;
    };
    if b.targets.len() != n0 {
        return false;
    }
    // (Whether a spell was cast: see `Effect::CastCard`.)
    let e = Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), e]);
    true
}

fn f_when_you_cast_that_spell(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l).trim();
    let Some(r) = l.strip_prefix("when you cast that spell, ") else {
        return false;
    };
    if !ends_casting(prev) {
        return false;
    }
    let text = r.replace("that spell's ", "its ");
    let Some(body) =
        super::r600_triggers::reflexive_body_about(&text, b, Sel::Var(vars::IT))
    else {
        return false;
    };
    if !body.targets.is_empty() {
        return false;
    }
    let e = Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), e]);
    true
}

/// "If you don't cast it this way, you may cast it this turn." (Rashmi and Ragavan): after
/// "you may cast it", "if you don't" (whether the spell was cast as the effect resolved).
fn f_if_you_dont_cast_it_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l).trim();
    let reworded = if let Some(r) = l.strip_prefix("if you don't cast it this way, ") {
        format!("if you don't, {r}")
    } else {
        return false;
    };
    if !ends_casting(prev) {
        return false;
    }
    if crate::oracle_ext::apply_followup_ext(&reworded, prev, b) {
        return true;
    }
    let n0 = b.targets.len();
    let Some(e) = parse_sentence(&reworded, b) else {
        return false;
    };
    if b.targets.len() != n0 {
        return false;
    }
    *prev = Effect::seq(vec![std::mem::take(prev), e]);
    true
}

inventory::submit! { FollowupPattern { name: "cast riders 4: if you cast a spell this way, X (after a spell cast as the effect resolves)", priority: 90, apply: f_if_cast_this_way } }
inventory::submit! { FollowupPattern { name: "cast riders 4: if you don't cast it this way, X", priority: 90, apply: f_if_you_dont_cast_it_this_way } }
inventory::submit! { FollowupPattern { name: "cast riders 4: when you cast that spell, X (after a spell cast as the effect resolves)", priority: 90, apply: f_when_you_cast_that_spell } }
