//! A sentence that follows "When you do, you may cast target [card] ..." and refers to the
//! spell cast by that reflexive triggered ability ("If that spell would be put into your
//! graveyard, exile it instead." — Wishing Well) is part of the reflexive ability
//! (CR 603.12): the replacement effect is created as the reflexive ability resolves, for
//! the spell it cast (CR 400.7h).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The body of the reflexive triggered ability `e` creates, if it's one ("When you do,
/// [effect]").
fn reflexive_body(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::If { then, .. } => reflexive_body(then),
        Effect::Reflexive { body } => Some(body),
        _ => None,
    }
}

/// Whether the effect casts a card (and so sets "that spell").
fn casts(e: &Effect) -> bool {
    match e {
        Effect::CastCard { .. } => true,
        Effect::May { effect, .. } => casts(effect),
        Effect::Seq(v) => v.iter().any(casts),
        _ => false,
    }
}

fn that_spell_in_reflexive(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !end(l).starts_with("if that spell would be put into ") {
        return false;
    }
    let Some(body) = reflexive_body(prev) else {
        return false;
    };
    if !casts(&body.effect) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_sentence(l, b) else {
        return false;
    };
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "that spell cast by a reflexive ability: exile it instead", priority: 60, apply: that_spell_in_reflexive } }
