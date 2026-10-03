//! Milling (CR 701.17):
//!
//! * ("If an opponent would mill one or more cards, they mill twice that many cards
//!   instead." is compiled by `replacement_grammar_events.rs`, CR 701.17d);
//! * "You may put a land card milled this way into your hand." (the milled card is found
//!   where it went, CR 701.17c).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// "[you may] put a land card milled this way into your hand": one of the milled cards
/// (as they are where they went), put into your hand.
fn put_milled_into_hand(l: &str, _b: &mut Builder) -> Option<Effect> {
    // ("You may" has already been parsed off: the effect is optional.)
    let r = end(l).strip_prefix("put ")?;
    let r = r.strip_suffix(" milled this way into your hand")?;
    let (n, r) = parse_number(r)?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, Filter::In(Box::new(Sel::Var(vars::IT)))]),
            count: n,
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "a701 put a milled card into your hand", priority: 100, parse: put_milled_into_hand } }

/// "two cards that share all their card types were milled this way" (Demonic Covenant),
/// "two cards that share a card type were milled this way" (The Tale of Tamiyo): about the
/// two cards the preceding instruction milled (CR 701.17c).
fn milled_cards_share_types(c: &str) -> Option<Condition> {
    let name = match end(c) {
        "two cards that share all their card types were milled this way" => {
            crate::mill_rules::TWO_MILLED_SHARE_ALL_TYPES
        }
        "two cards that share a card type were milled this way" => {
            crate::mill_rules::TWO_MILLED_SHARE_A_TYPE
        }
        _ => return None,
    };
    Some(Condition::Custom(SmolStr::new(name)))
}

inventory::submit! { super::ConditionPattern { name: "a701 two milled cards share card types", priority: 100, parse: milled_cards_share_types } }
