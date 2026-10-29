//! Exile "edicts": a player chooses cards or permanents of their own and exiles them.
//!
//! - "Target opponent exiles a card from their hand.", "Target opponent exiles two cards
//!   from their hand.", "Each opponent exiles a card from their hand."
//! - "Target player exiles a card from their graveyard."
//! - "Target opponent exiles a creature they control."
//!
//! The player chooses as the effect resolves (CR 608.2d); a player with fewer such
//! objects exiles all of them. Several players choose and exile one after another in
//! turn order (hand and graveyard choices are hidden from and irrelevant to the others);
//! the battlefield form is only accepted for a single player.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

fn p_exile_edict(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, rest) = player_ref(l, b)?;
    let rest = rest.trim().strip_prefix("exiles ")?;
    // "the top card of their library" is a different effect.
    if rest.starts_with("the ") {
        return None;
    }
    let (n, r) = parse_number(rest)?;
    if !matches!(n, Value::Const(_)) {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    let tail = end(tail).trim();
    let many = matches!(
        who,
        PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
    );
    // Whose objects: the single player, or each player in turn.
    let (chooser, rel) = if many {
        (PlayerRef::Iterated, PlayerRel::Iterated)
    } else {
        let rel = match &who {
            PlayerRef::Target(n) => PlayerRel::Target(*n),
            PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
            PlayerRef::You => PlayerRel::You,
            _ => return None,
        };
        (who.clone(), rel)
    };
    let zone_filter = match tail {
        "from their hand" | "from your hand" => {
            Filter::and(vec![Filter::InZone(ZoneKind::Hand), Filter::OwnedBy(rel)])
        }
        "from their graveyard" | "from your graveyard" => Filter::and(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(rel),
        ]),
        "they control" | "you control" if !many => Filter::ControlledBy(rel),
        _ => return None,
    };
    let exile = Effect::Exile {
        what: Sel::Choose {
            chooser,
            filter: Filter::and(vec![f, zone_filter]),
            count: n,
            up_to: false,
            store: None,
        },
        face_down: false,
        link: false,
    };
    Some(if many {
        Effect::ForEachPlayer {
            who,
            effect: Box::new(exile),
        }
    } else {
        exile
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: [player] exiles [objects] of their own", priority: 60, parse: p_exile_edict } }
