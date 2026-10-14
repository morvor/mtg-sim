//! "Target opponent reveals their hand. You may cast an instant or sorcery spell from
//! among those cards without paying its mana cost." (Mindclaw Shaman): as the ability
//! resolves, its controller may choose one of the revealed cards with that quality and
//! cast it (CR 608.2g) without paying its mana cost (CR 118.9).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::CardType;

/// Whose hand the effect revealed last.
fn revealed_hand(e: &Effect) -> Option<PlayerRel> {
    match e {
        Effect::RevealHand { who } => match who {
            PlayerRef::Target(n) => Some(PlayerRel::Target(*n)),
            PlayerRef::TriggerPlayer => Some(PlayerRel::TriggerPlayer),
            PlayerRef::DefendingPlayer => Some(PlayerRel::Defending),
            _ => None,
        },
        Effect::Seq(v) => v.last().and_then(revealed_hand),
        _ => None,
    }
}

fn cast_from_revealed_hand(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l.trim())
        .strip_prefix("you may cast ")
        .and_then(|r| {
            r.strip_suffix(" from among those cards without paying its mana cost")
                .or_else(|| r.strip_suffix(" from among them without paying its mana cost"))
        })
        .and_then(|r| r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")))
    else {
        return false;
    };
    let Some(owner) = revealed_hand(prev) else {
        return false;
    };
    // "[quality] spell": the card has that quality.
    let Some(desc) = r.strip_suffix("spell") else {
        return false;
    };
    let quality = match desc.trim() {
        "" => Filter::Any,
        d => match parse_object_phrase(&format!("{d} card")) {
            Some((f, _, tail)) if end(tail).trim().is_empty() => f,
            _ => return false,
        },
    };
    let cast = Effect::CastCard {
        who: PlayerRef::You,
        // Choosing none is not casting one ("you may").
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(owner),
                Filter::Not(Box::new(Filter::Type(CardType::Land))),
                quality,
            ]),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    };
    *prev = Effect::seq(vec![std::mem::take(prev), cast]);
    true
}

inventory::submit! { FollowupPattern { name: "you may cast a [quality] spell from among the revealed hand without paying its mana cost", priority: 100, apply: cast_from_revealed_hand } }
