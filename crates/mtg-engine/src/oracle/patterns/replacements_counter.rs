//! Countered spells put somewhere other than the graveyard (CR 701.6a, 614.15):
//!
//! - "Counter target spell. If that spell is countered this way, exile it instead of
//!   putting it into its owner's graveyard." (Dissipate, Assert Authority)
//! - "... put it on top of its owner's library instead of into that player's graveyard."
//!   (Memory Lapse)
//! - "... put it on the bottom of its owner's library instead of into that player's
//!   graveyard." (Spell Crumple)
//! - "... put it into its owner's hand instead of into that player's graveyard." (Remand)
//!
//! The follow-up sentence is a self-replacement effect of the counter effect (CR 614.15):
//! the counter is wrapped in [`Effect::SelfReplace`] with a zone-change replacement for
//! the countered spell moving from the stack to a graveyard. If the spell isn't countered
//! (it can't be, or its controller paid for "unless its controller pays"), nothing moves
//! and the replacement is gone once the effect is done.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;

/// "exile it instead of putting it into its owner's graveyard" etc. → where the spell goes.
fn destination(r: &str) -> Option<Destination> {
    Some(match r {
        "exile it instead of putting it into its owner's graveyard" => {
            Destination::zone(ZoneKind::Exile)
        }
        "put it on top of its owner's library instead of into that player's graveyard" => {
            Destination::library_top()
        }
        "put it on the bottom of its owner's library instead of into that player's graveyard" => {
            Destination::library_bottom()
        }
        "put it into its owner's hand instead of into that player's graveyard" => {
            Destination::zone(ZoneKind::Hand)
        }
        _ => return None,
    })
}

/// The (single) counter effect of `e`: a plain counter, or the "otherwise" branch of
/// "counter target spell unless its controller pays {N}".
fn counter_of(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::CounterSpell { .. } => Some(e),
        Effect::PayOptional {
            then, otherwise, ..
        } if matches!(**then, Effect::Noop) => counter_of(otherwise),
        _ => None,
    }
}

fn f_countered_instead(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = l
        .strip_prefix("if that spell is countered this way, ")
        .or_else(|| l.strip_prefix("if the spell is countered this way, "))
    else {
        return false;
    };
    let Some(dest) = destination(r) else {
        return false;
    };
    let Some(counter) = counter_of(prev) else {
        return false;
    };
    let Effect::CounterSpell { what } = counter else {
        return false;
    };
    // Only a countered spell moves; a countered ability just ceases to exist.
    let filter = Filter::In(Box::new(what.clone()));
    let inner = std::mem::replace(counter, Effect::Noop);
    *counter = Effect::SelfReplace {
        replacement: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter,
                from: Some(ZoneKind::Stack),
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(dest),
            self_replacement: true,
            optional: false,
        },
        effect: Box::new(inner),
    };
    true
}

inventory::submit! { FollowupPattern { name: "replacements: countered this way, put it elsewhere instead", priority: 50, apply: f_countered_instead } }
