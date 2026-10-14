//! "You may pay {1} and sacrifice an artifact.", "you may pay {2}{R} and sacrifice a
//! nonland permanent", "you may pay {1} and discard a card", "you may pay 2 life and
//! exile it", "you may pay {3} and sacrifice it": an optional cost of several parts paid
//! as the ability resolves (CR 118.12): the player either pays all of it or none of it, and
//! can't pay it unless every part can be paid (CR 118.3) — a sacrifice needs a permanent
//! to sacrifice, "exile it" needs the object to still be where it was. A following "If you
//! do" reads whether it was paid.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// The action part: "sacrifice an artifact", "discard a card", "sacrifice it", "exile it".
fn action_part(a: &str, b: &mut Builder) -> Option<CostPart> {
    let a = end(a);
    for (verb, sacrifice) in [("sacrifice ", true), ("exile ", false)] {
        let Some(r) = a.strip_prefix(verb) else {
            continue;
        };
        // An object the text refers to ("it", "that creature").
        if r == "~" && sacrifice {
            return Some(CostPart::SacrificeSelf);
        }
        let saved = b.targets.len();
        if let Some((what, rest)) = object_ref(r, b) {
            if b.targets.len() == saved && end(&rest).is_empty() && names_one_object(&what) {
                if sacrifice && matches!(what, Sel::This) {
                    return Some(CostPart::SacrificeSelf);
                }
                let e = if sacrifice {
                    Effect::SacrificeObjects { what }
                } else {
                    Effect::Exile {
                        what,
                        face_down: false,
                        link: false,
                    }
                };
                return Some(CostPart::Effect(Box::new(e)));
            }
        }
        b.targets.truncate(saved);
    }
    // "sacrifice an artifact", "discard a card": parts the cost grammar reads.
    if !(a.starts_with("sacrifice ") || a.starts_with("discard ")) {
        return None;
    }
    let part = crate::oracle::costs::parse_cost_part(a)?;
    matches!(
        part,
        CostPart::Sacrifice { .. } | CostPart::Discard { .. } | CostPart::SacrificeSelf
    )
    .then_some(part)
}

/// One object a pronoun names: the source, the object a trigger is about.
fn names_one_object(s: &Sel) -> bool {
    matches!(
        s,
        Sel::This | Sel::TriggerObject | Sel::TriggerLki | Sel::Target(_)
    )
}

fn may_pay_and(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let (pay, action) = r.split_once(" and ")?;
    let mut cost = super::counters_resources_pay::resolution_cost(pay)?;
    let part = action_part(action, b)?;
    cost.parts.push(part);
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "you may pay [resources] and [sacrifice/discard/exile ...]", priority: 100, parse: may_pay_and } }
