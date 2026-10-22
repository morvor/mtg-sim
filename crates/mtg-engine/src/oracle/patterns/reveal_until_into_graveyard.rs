//! "Target player reveals cards from the top of their library until they reveal a land
//! card, then puts those cards into their graveyard." (Balustrade Spy, Undercity Informer,
//! Destroy the Evidence, Consuming Aberration): every revealed card — the one found and
//! the ones before it, or the whole library if none is found — goes to that player's
//! graveyard (CR 701.20a, 401.1).

use super::card_flow_search::card_filter;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::end;

fn reveal_until_into_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, r) =
        l.split_once(" reveals cards from the top of their library until they reveal ")?;
    let desc = r.strip_suffix(", then puts those cards into their graveyard")?;
    let desc = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an "))?;
    // The card description is read before the subject adds a target.
    let filter = card_filter(desc, b)?;
    let (who, rest) = player_ref(subject, b)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let graveyard = Destination::zone(ZoneKind::Graveyard);
    Some(Effect::RevealUntil {
        who,
        filter: Filter::And(vec![Filter::Card, filter]),
        found_to: graveyard.clone(),
        rest_to: graveyard,
    })
}

inventory::submit! { EffectPattern { name: "reveal cards until they reveal a [card], then put those cards into their graveyard", priority: 90, parse: reveal_until_into_graveyard } }
