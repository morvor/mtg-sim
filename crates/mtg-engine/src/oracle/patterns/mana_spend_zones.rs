//! Spending restrictions on mana about where the spell is cast from (CR 106.6, 601.2a):
//! "Spend this mana only to cast spells from your graveyard." (Rootcoil Creeper), "...
//! only to cast spells from exile" (Interdimensional Web Watch), "... only to cast a spell
//! from anywhere other than your hand" (Mm'menon, the Right Hand), "... only to cast
//! spells with flashback from a graveyard" (Altar of the Lost), and "This mana can't be
//! spent to cast spells from your hand." (Karolina Dean, Runaway): checked against the
//! spell being paid for, which records the zone it was cast from
//! (`Filter::CastFrom`). "You" is the player spending the mana.

use super::mana_restrictions::restrict;
use super::FollowupPattern;
use crate::ability::*;
use crate::mana::{ManaRestriction, SpendFilter};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "from your graveyard", "from exile", "from a graveyard", "from your hand", "from
/// anywhere other than your hand".
fn cast_from(s: &str) -> Option<Filter> {
    Some(match s {
        // Only its owner's cards are in a player's graveyard or hand (CR 404.2, 402.1).
        "from your graveyard" => Filter::and(vec![
            Filter::CastFrom(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        "from a graveyard" => Filter::CastFrom(ZoneKind::Graveyard),
        "from exile" => Filter::CastFrom(ZoneKind::Exile),
        "from your hand" => Filter::and(vec![
            Filter::CastFrom(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        "from anywhere other than your hand" => Filter::Not(Box::new(Filter::and(vec![
            Filter::CastFrom(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ]))),
        _ => return None,
    })
}

/// "spells from your graveyard", "a spell from exile", "spells with flashback from a
/// graveyard".
fn spells_from(s: &str) -> Option<Filter> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let i = s.find(" from ")?;
    let (spells, zone) = (&s[..i], &s[i + 1..]);
    let from = cast_from(zone)?;
    let quality = match spells {
        "spells" | "spell" => None,
        _ => {
            let (body, tail) = spells
                .strip_prefix("spells ")
                .map(|t| ("cards", t))
                .or_else(|| spells.strip_prefix("spell ").map(|t| ("card", t)))?;
            let phrase = format!("{body} {tail}");
            let (f, _, rest) = parse_object_phrase(&phrase)?;
            if !end(rest).trim().is_empty() {
                return None;
            }
            Some(f)
        }
    };
    Some(match quality {
        Some(q) => Filter::and(vec![q, from]),
        None => from,
    })
}

fn spend_by_zone(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l).trim();
    let r = if let Some(r) = l.strip_prefix("spend this mana only to cast ") {
        spells_from(r).map(|f| ManaRestriction::CastSpell(SpendFilter::new(f)))
    } else if let Some(r) = l.strip_prefix("this mana can't be spent to cast ") {
        spells_from(r).map(|f| ManaRestriction::NotCastSpell(SpendFilter::new(f)))
    } else {
        None
    };
    let Some(r) = r else {
        return false;
    };
    let mut e = prev.clone();
    if !restrict(&mut e, &r) {
        return false;
    }
    *prev = e;
    true
}

inventory::submit! { FollowupPattern { name: "spend this mana only to cast spells from [zone]", priority: 95, apply: spend_by_zone } }
