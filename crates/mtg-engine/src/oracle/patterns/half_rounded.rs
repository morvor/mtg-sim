//! "Target opponent sacrifices half the creatures they control of their choice, rounded
//! up." / "Target opponent discards half the cards in their hand, rounded up." (Rush of
//! Dread): half a number, rounded as the text says (CR 107.1a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

/// ", rounded up" / ", rounded down".
fn rounding(s: &str) -> Option<bool> {
    match end(s).trim() {
        ", rounded up" | "rounded up" => Some(true),
        ", rounded down" | "rounded down" => Some(false),
        _ => None,
    }
}

/// The relation for "they control" for the player `p`.
fn rel_of(p: &PlayerRef) -> Option<PlayerRel> {
    Some(match p {
        PlayerRef::You => PlayerRel::You,
        PlayerRef::Target(n) => PlayerRel::Target(*n),
        PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
        PlayerRef::DefendingPlayer => PlayerRel::Defending,
        _ => return None,
    })
}

fn half_edict_or_discard(l: &str, b: &mut Builder) -> Option<Effect> {
    let saved = b.targets.len();
    let (who, rest) = player_ref(end(l), b)?;
    let rest = rest.trim();
    let parsed = (|| {
        if let Some(r) = rest.strip_prefix("discards half the cards in their hand") {
            let up = rounding(r)?;
            return Some(Effect::Discard {
                who: who.clone(),
                n: Value::Div(Box::new(Value::HandSize(who.clone())), 2, up),
                random: false,
                filter: Filter::Any,
            });
        }
        let r = rest.strip_prefix("sacrifices half the ")?;
        let (noun, r) = r.split_once(" they control")?;
        let r = r.strip_prefix(" of their choice").unwrap_or(r);
        let up = rounding(r)?;
        let (f, true, tail) = parse_object_phrase(noun)? else {
            return None;
        };
        if !end(tail).is_empty() {
            return None;
        }
        let theirs = Filter::and(vec![f.clone(), Filter::ControlledBy(rel_of(&who)?)]);
        Some(Effect::Sacrifice {
            who: who.clone(),
            filter: f,
            count: Value::Div(Box::new(Value::Count(theirs)), 2, up),
        })
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "half the [objects] they control / cards in their hand, rounded", priority: 60, parse: half_edict_or_discard } }
