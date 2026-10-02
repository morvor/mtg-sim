//! "[You may] cast a[n] [quality] spell [with mana value N or less] from your graveyard
//! without paying its mana cost" (Sword of Once and Future), "... from that player's
//! graveyard ..." (Sorcerous Squall): as the effect resolves, its controller chooses a
//! card in that graveyard with that quality and casts it (CR 608.2g) without paying its
//! mana cost (CR 118.9) — or chooses none. Any additional costs may (or, if mandatory,
//! must) still be paid, and no alternative cost can be chosen (CR 118.9a). The hand form
//! is `cast_from_hand_free.rs`'s.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::CardType;

/// Whose graveyard: "your graveyard", "that player's graveyard" (the player the effect
/// refers to, e.g. its target opponent).
fn graveyard_owner(s: &str, b: &Builder) -> Option<PlayerRel> {
    match s {
        "your graveyard" => Some(PlayerRel::You),
        // Any player's (Spectral Arcanist).
        "a graveyard" => Some(PlayerRel::Any),
        "that player's graveyard" => match &b.it_player {
            PlayerRef::Target(n) => Some(PlayerRel::Target(*n)),
            PlayerRef::TriggerPlayer => Some(PlayerRel::TriggerPlayer),
            PlayerRef::DefendingPlayer => Some(PlayerRel::Defending),
            _ => None,
        },
        _ => None,
    }
}

fn cast_from_graveyard_free(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("cast ")?
        .strip_suffix(" without paying its mana cost")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (desc, zone) = r.rsplit_once(" from ")?;
    let owner = graveyard_owner(zone, b)?;
    // "[quality] spell [with ...]": the card it's cast from has that quality.
    let (before, after) = desc.split_once("spell")?;
    let desc = format!("{before}card{after}");
    let (quality, _, tail) = parse_object_phrase(&desc)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        // Choosing none is not casting one.
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
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
    })
}

inventory::submit! { EffectPattern { name: "cast a spell from a graveyard without paying its mana cost", priority: 100, parse: cast_from_graveyard_free } }
