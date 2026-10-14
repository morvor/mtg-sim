//! "The top [type] card of [whose] graveyard" (CR 404.2: a graveyard's order is fixed;
//! the top card is the one put there most recently):
//!
//! * "sacrifice ~ unless you exile the top creature card of your graveyard" (Barrow Ghoul,
//!   Circling Vultures);
//! * "return the top creature card of your graveyard to the battlefield" (Mistmoon
//!   Griffin);
//! * "put the top creature card of defending player's graveyard onto the battlefield
//!   under your control" (Bone Dancer).
//!
//! The filter is `kw/graveyard_order.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::graveyard_order::top_card_of_graveyard;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// "the top [type] card of [whose] graveyard", and what follows it.
fn top_card_phrase(s: &str) -> Option<(Filter, &str)> {
    let r = s.strip_prefix("the top ")?;
    let (ty, r) = r.split_once(" card of ")?;
    let t = CardType::from_word(ty)?;
    let (whose, rest) = if let Some(rest) = r.strip_prefix("your graveyard") {
        (PlayerRel::You, rest)
    } else if let Some(rest) = r.strip_prefix("defending player's graveyard") {
        (PlayerRel::Defending, rest)
    } else {
        return None;
    };
    Some((top_card_of_graveyard(t, whose), rest))
}

fn graveyard_order_effects(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l).trim();
    // "sacrifice ~ unless you exile the top creature card of your graveyard"
    if let Some(r) = l.strip_prefix("sacrifice ~ unless you exile ") {
        let (filter, rest) = top_card_phrase(r)?;
        if !rest.is_empty() {
            return None;
        }
        return Some(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost {
                mana: None,
                parts: vec![CostPart::Exile {
                    filter,
                    zone: ZoneKind::Graveyard,
                    count: Value::c(1),
                }],
            },
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
        });
    }
    // "return the top creature card of your graveyard to the battlefield", "put the top
    // creature card of defending player's graveyard onto the battlefield under your
    // control"
    let (r, verb_return) = match l.strip_prefix("return ") {
        Some(r) => (r, true),
        None => (l.strip_prefix("put ")?, false),
    };
    let (filter, rest) = top_card_phrase(r)?;
    let to = match (verb_return, rest.trim()) {
        (true, "to the battlefield") | (false, "onto the battlefield") => {
            Destination::battlefield()
        }
        (true, "to the battlefield under your control")
        | (false, "onto the battlefield under your control") => {
            Destination::battlefield().under_your_control()
        }
        _ => return None,
    };
    Some(Effect::Move {
        what: Sel::All(filter),
        to,
    })
}

inventory::submit! { EffectPattern { name: "the top [type] card of [whose] graveyard", priority: 100, parse: graveyard_order_effects } }
