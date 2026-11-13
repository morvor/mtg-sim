//! "Create a tapped and attacking token that's a copy of that card, except ... At the
//! beginning of the next end step, exile that token unless ~ is your Ring-bearer."
//! (Sauron, the Necromancer): right after creating a token, "that token" is the token
//! created (not the card it copies), and the condition is part of the delayed
//! instruction, checked as it resolves at that step (CR 603.7, 608.2c).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn f_created_token_delayed_unless(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if super::tokens_copies_create::last_create(prev).is_none() {
        return false;
    }
    let Some((verb, r, step)) = super::damage_removal::delayed_parts(end(l)) else {
        return false;
    };
    let Some(c) = ["that token unless ", "the token unless "]
        .iter()
        .find_map(|p| r.strip_prefix(p))
    else {
        return false;
    };
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let Some(mut e) =
        super::damage_removal::delayed_removal(verb, Sel::Var(vars::CREATED), "", step)
    else {
        return false;
    };
    // The removal happens unless the condition is true then.
    let Effect::Seq(v) = &mut e else {
        return false;
    };
    let Some(Effect::AtNext { effect, .. }) = v.last_mut() else {
        return false;
    };
    let inner = std::mem::replace(effect.as_mut(), Effect::Noop);
    **effect = Effect::If {
        cond: Condition::Not(Box::new(cond)),
        then: Box::new(inner),
        otherwise: Box::new(Effect::Noop),
    };
    super::tokens_copies_create::append_after_create(prev, e)
}

inventory::submit! { FollowupPattern { name: "created token: delayed removal unless", priority: 44, apply: f_created_token_delayed_unless } }
