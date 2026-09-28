//! Copying a card the effect exiled: "Exile up to one target instant or sorcery card with
//! mana value 2 or less from your graveyard. Copy it. You may cast the copy without paying
//! its mana cost." (Roving Actuator), "exile target noncreature, nonland card with mana
//! value less than ~'s power from a graveyard and copy it" (Narset, Enlightened Exile).
//! The copy is created in exile (CR 707.12) and "the copy" is `vars::CREATED` (see
//! `r707_copy_cards.rs` for casting it).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// Whether the effect ends by exiling targeted cards face up.
fn ends_with_exiling_a_target(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::Target(_),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exiling_a_target),
        _ => false,
    }
}

fn copy_of_exiled() -> Effect {
    Effect::CopyCard {
        what: Sel::Var(vars::IT),
        named: None,
    }
}

/// "Copy it." / "Copy that card." after exiling a target card.
fn copy_it(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(end(l), "copy it" | "copy that card" | "copy the exiled card")
        || !ends_with_exiling_a_target(prev)
    {
        return false;
    }
    *prev = Effect::seq(vec![std::mem::take(prev), copy_of_exiled()]);
    true
}

/// "exile target [card] from a graveyard and copy it".
fn exile_and_copy_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" and copy it")?;
    if !r.starts_with("exile target ") {
        return None;
    }
    let e = parse_clause(r, b)?;
    if !ends_with_exiling_a_target(&e) {
        return None;
    }
    Some(Effect::seq(vec![e, copy_of_exiled()]))
}

inventory::submit! { FollowupPattern { name: "copy it (the exiled card)", priority: 90, apply: copy_it } }
inventory::submit! { EffectPattern { name: "exile target card and copy it", priority: 90, parse: exile_and_copy_it } }
