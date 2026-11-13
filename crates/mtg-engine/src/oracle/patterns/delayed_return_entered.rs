//! A condition on how the returned permanent entered, after an exile and a delayed return
//! in the form `damage_removal::f_return_exiled` reads ("At the beginning of the next end
//! step, return that card to the battlefield under its owner's control."): "If it entered
//! under your control, put a +1/+1 counter on ~." (Phelia, Exuberant Shepherd) is part of
//! the delayed ability. It's checked as the card returns, about the new object it
//! returned as (CR 400.7, 603.7): not as the exile happens, and not about who controlled
//! the permanent then; a card that doesn't return (a token, CR 111.8) didn't enter.
//! (`delayed_grammar::f_delayed_continues` does the same for the delayed abilities its
//! grammar creates.)

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// What the delayed return `prev` ends with does as it resolves: an `AtNext` whose effect
/// begins by moving the stored card to the battlefield.
fn delayed_return(prev: &mut Effect) -> Option<&mut Effect> {
    let last = match prev {
        Effect::Seq(v) => v.last_mut()?,
        e => e,
    };
    let Effect::AtNext { effect, .. } = last else {
        return None;
    };
    let first = match effect.as_ref() {
        Effect::Seq(v) => v.first()?,
        e => e,
    };
    match first {
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(effect.as_mut()),
        _ => None,
    }
}

fn f_delayed_return_entered(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.starts_with("if it entered ") || delayed_return(prev).is_none() {
        return false;
    }
    // "It" is what the return put onto the battlefield (the move sets `vars::IT` to what
    // entered, nothing if nothing did), checked right after it.
    let n0 = b.targets.len();
    let saved_it = std::mem::replace(&mut b.it, Sel::Var(vars::IT));
    let parsed = parse_sentence(l, b);
    let new_targets = b.targets.len() > n0;
    b.it = saved_it;
    let Some(e) = parsed.filter(|_| !new_targets) else {
        return false;
    };
    let Some(inner) = delayed_return(prev) else {
        return false;
    };
    let returned = std::mem::replace(inner, Effect::Noop);
    *inner = Effect::seq(vec![returned, e]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed return: if it entered under your control", priority: 44, apply: f_delayed_return_entered } }
