//! Copies of cards (round 5; CR 707.12):
//!
//! * "Whenever you reveal an instant or sorcery card this way, copy that card and you may
//!   cast the copy. That copy costs {2} less to cast." (God-Eternal Kefnet): a copy of the
//!   card the trigger names, wherever it is (here the hand it was drawn into), cast while
//!   the ability resolves (CR 608.2g), with its cost reduced (CR 601.2f).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "copy that card" / "copy it" in a triggered ability whose trigger names a card.
fn copy_trigger_card(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(end(l), "copy that card" | "copy it") || !b.in_trigger {
        return None;
    }
    if !matches!(b.it, Sel::TriggerObject) {
        return None;
    }
    // Only a card revealed as it was drawn ("Whenever you reveal ... this way").
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("whenever you reveal ") || !raw.contains(" as you draw it") {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::TriggerObject,
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "copies 6: copy that card (the card a reveal trigger names)", priority: 50, parse: copy_trigger_card } }

/// "That copy costs {2} less to cast." after "you may cast the copy".
fn that_copy_costs_less(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("that copy costs {")
        .and_then(|r| r.strip_suffix("} less to cast"))
    else {
        return false;
    };
    let Ok(n) = r.parse::<u32>() else {
        return false;
    };
    fn last_cast(e: &mut Effect) -> Option<&mut Effect> {
        match e {
            Effect::CastCard {
                what: Sel::Var(v),
                free: false,
                ..
            } if *v == vars::CREATED => Some(e),
            Effect::Seq(v) => v.last_mut().and_then(last_cast),
            Effect::May { effect, .. } => last_cast(effect),
            _ => None,
        }
    }
    let Some(cast) = last_cast(prev) else {
        return false;
    };
    let inner = std::mem::take(cast);
    *cast = Effect::WithPlayTerms {
        terms: PlayTerms {
            cost_reduction: n,
            ..Default::default()
        },
        effect: Box::new(inner),
    };
    true
}

inventory::submit! { FollowupPattern { name: "copies 6: that copy costs {N} less to cast", priority: 50, apply: that_copy_costs_less } }
