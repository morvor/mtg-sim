//! Delayed triggered abilities, grammar III (cards3-triggers-durations).
//!
//! The cards a player exiled, returned later by that player: "Target player exiles all
//! cards from their hand face down. At the beginning of the end step of that player's next
//! turn, that player returns those cards to their hand." (Suppress; CR 603.7a, 603.7c: the
//! delayed ability refers to the cards the first instruction exiled).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether the effect ends with an exile instruction (possibly performed by another
/// player).
fn ends_with_exile(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile),
        Effect::AsPlayer { effect, .. } => ends_with_exile(effect),
        Effect::Exile { .. } => true,
        _ => false,
    }
}

fn those_exiled_cards_returned_later(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_exile(prev) {
        return false;
    }
    let s = end(s);
    let Some((delay, instr)) = s.split_once(", ") else {
        return false;
    };
    if !delay.starts_with("at the beginning of ")
        || !matches!(
            instr,
            "that player returns those cards to their hand" | "they return those cards to their hand"
        )
    {
        return false;
    }
    // "Those cards": what the exile instruction exiled (its "it", CR 603.7c).
    let saved = b.it.clone();
    b.it = Sel::Var(vars::IT);
    let e = super::delayed_grammar::delayed_instruction(
        &format!("{delay}, return those cards to their owner's hand"),
        b,
    );
    b.it = saved;
    let Some(e) = e else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar 3: that player returns those (exiled) cards to their hand later", priority: 100, apply: those_exiled_cards_returned_later } }
