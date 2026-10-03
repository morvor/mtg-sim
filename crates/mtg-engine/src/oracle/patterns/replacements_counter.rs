//! Countered spells put somewhere other than the graveyard (CR 701.6a, 614.15):
//!
//! - "Counter target spell. If that spell is countered this way, exile it instead of
//!   putting it into its owner's graveyard." (Dissipate, Assert Authority)
//! - "... put it on top of its owner's library instead of into that player's graveyard."
//!   (Memory Lapse)
//! - "... put it on the bottom of its owner's library instead of into that player's
//!   graveyard." (Spell Crumple)
//! - "... put it into its owner's hand instead of into that player's graveyard." (Remand)
//! - "... exile it with three time counters on it instead of putting it into its owner's
//!   graveyard." (Delay)
//! - "... put that card on your choice of the top or bottom of its owner's library instead
//!   of into that player's graveyard." (Hinder: the counter spell's controller chooses as
//!   the card moves)
//! - "If an artifact or creature spell is countered this way, put that card onto the
//!   battlefield under your control instead of into its owner's graveyard." (Desertion)
//!
//! The replacement moves the card to the whole destination: with its counters, to the
//! chosen position, under the spell's controller's control (CR 614.6, `destinations.rs`).
//!
//! The follow-up sentence is a self-replacement effect of the counter effect (CR 614.15):
//! the counter is wrapped in [`Effect::SelfReplace`] with a zone-change replacement for
//! the countered spell moving from the stack to a graveyard. If the spell isn't countered
//! (it can't be, or its controller paid for "unless its controller pays"), nothing moves
//! and the replacement is gone once the effect is done.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::parse_number;

/// "exile it instead of putting it into its owner's graveyard" etc. → where the spell goes.
fn destination(r: &str) -> Option<Destination> {
    // "that card" names the countered spell's card, as "it" does.
    let r = &r.replacen("put that card ", "put it ", 1);
    // "exile it with three time counters on it instead of putting it into its owner's
    // graveyard" (Delay).
    if let Some(c) = r
        .strip_prefix("exile it with ")
        .and_then(|c| c.strip_suffix(" on it instead of putting it into its owner's graveyard"))
    {
        let (n, rest) = parse_number(c)?;
        let (kind, rest) = crate::oracle::costs::counter_kind(rest)?;
        if !matches!(rest.trim(), "counter" | "counters") {
            return None;
        }
        let mut d = Destination::zone(ZoneKind::Exile);
        d.with_counters = vec![(kind, n)];
        return Some(d);
    }
    Some(match r.as_str() {
        // Hinder: the controller of the counter spell chooses as the card moves.
        "put it on your choice of the top or bottom of its owner's library instead of into that player's graveyard" => {
            let mut d = Destination::library_top();
            d.position_choice = vec![LibraryPosition::Top, LibraryPosition::Bottom];
            d
        }
        // Desertion: it enters under the counter spell's controller's control (CR 110.2a).
        "put it onto the battlefield under your control instead of into its owner's graveyard" => {
            Destination::battlefield().under_your_control()
        }
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
    // "If a permanent spell is countered this way, ..." (Thranduil's Decree): only a
    // permanent spell moves elsewhere.
    // "If an artifact or creature spell is countered this way, ..." (Desertion): only such
    // a spell moves elsewhere.
    let (r, only) = match l
        .strip_prefix("if that spell is countered this way, ")
        .or_else(|| l.strip_prefix("if the spell is countered this way, "))
    {
        Some(r) => (r, None),
        None => {
            let Some((kind, r)) = l
                .strip_prefix("if a ")
                .or_else(|| l.strip_prefix("if an "))
                .and_then(|r| r.split_once(" spell is countered this way, "))
            else {
                return false;
            };
            let f = match kind {
                "permanent" => Filter::PermanentCard,
                _ => match crate::oracle::keywords::quality_phrase(kind) {
                    Some(f) if f.zone().is_none() => f,
                    _ => return false,
                },
            };
            (r, Some(f))
        }
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
    let mut filter = Filter::In(Box::new(what.clone()));
    if let Some(f) = only {
        filter = Filter::and(vec![filter, f]);
    }
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
