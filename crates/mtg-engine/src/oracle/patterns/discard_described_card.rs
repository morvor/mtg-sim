//! "discard a nonland card", "you may discard a creature card. If you do, ...": the
//! controller discards cards of the described kind from their hand (CR 701.9a). With none
//! in hand, nothing is discarded, so a following "if you do" doesn't happen. (Plain
//! "discard a card" and "discard two cards at random" are read by the core parser.)

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

fn discard_described_card(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("discard ")?;
    let (n, r) = parse_number(r)?;
    let r = r.trim();
    if r == "card" || r == "cards" || r.contains(" at random") {
        return None;
    }
    if !r.ends_with(" card") && !r.ends_with(" cards") {
        return None;
    }
    let (filter, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Discard {
        who: PlayerRef::You,
        n,
        random: false,
        filter,
    })
}

inventory::submit! { EffectPattern { name: "discard a [described] card", priority: 70, parse: discard_described_card } }
