//! "The next time one or more [permanents] enter this turn, [they enter as copies of the
//! chosen creature / each enters with N additional counters on it]." (Mystic Reflection,
//! Storyweave): a replacement effect that modifies how permanents enter the battlefield
//! (CR 614.1c), used up the next time one or more matching permanents enter — once for
//! all of them entering at the same time (see `ZoneState::entry_batch_uses`). An object
//! the effect refers to ("the chosen creature") is locked in as the effect is created
//! (`prevention::lock_def`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

fn next_time_one_or_more_enter(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("the next time one or more ")?;
    let (subj, rest) = r.split_once(" enter this turn, ")?;
    let (filter, plural, tail) = parse_object_phrase(subj)?;
    if !plural || !tail.trim().is_empty() {
        return None;
    }
    let action = if let Some(of) = rest.strip_prefix("they enter as copies of ") {
        let saved = b.targets.len();
        let (sel, after) = object_ref(of, b)?;
        if b.targets.len() != saved || !after.trim().is_empty() {
            b.targets.truncate(saved);
            return None;
        }
        // Only a chosen target: what a later sentence's "it" would mean isn't a fixed
        // object as the permanents enter.
        if !matches!(sel, Sel::Target(_)) {
            return None;
        }
        ReplacementAction::EnterAsCopy {
            filter: Filter::In(Box::new(sel)),
            optional: false,
            tapped: false,
        }
    } else {
        let counters = rest.strip_prefix("each enters with ")?;
        let (kind, n) = super::etb_replacement_grammar::counters_on(counters)?;
        ReplacementAction::EnterWithCounters(kind, n)
    };
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(filter),
            action,
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "the next time one or more [permanents] enter this turn, they enter [modified]", priority: 150, parse: next_time_one_or_more_enter } }
