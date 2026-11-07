//! Reflexive triggered abilities keyed to an exiled card (CR 603.12): "When a [quality]
//! card is exiled this way, [effect]" (Agatha's Soul Cauldron: "{T}: Exile target card
//! from a graveyard. When a creature card is exiled this way, put a +1/+1 counter on
//! target creature you control."). The ability triggers only if the preceding instruction
//! exiled a card with that quality; it has its own targets, chosen as it's put on the
//! stack. "That card" in it refers to the exiled card.

use super::r600_triggers::reflexive_body;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

fn when_exiled_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when ")?;
    let (card, effect) = r.split_once(" is exiled this way, ")?;
    let card = card
        .strip_prefix("a ")
        .or_else(|| card.strip_prefix("an "))?;
    let filter = if card == "card" {
        None
    } else {
        let (f, plural, tail) = parse_object_phrase(card)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        Some(f)
    };
    let rewritten = effect.contains("that card's");
    let effect = effect.replace("that card's", "its").replace("that card", "it");
    // The rewritten "its" still means that card (see `Builder::its_is_it`).
    let saved = b.its_is_it;
    b.its_is_it = saved || rewritten;
    let body = reflexive_body(&effect, b);
    b.its_is_it = saved;
    let body = body?;
    // The exile just performed stored the exiled cards as "it" and recorded whether
    // anything was exiled.
    let exiled = Sel::Var(vars::IT);
    let mut conds = vec![
        Condition::PrevHappened,
        Condition::SelNonEmpty(exiled.clone()),
    ];
    if let Some(f) = filter {
        // The card as it is in exile (CR 400.7): a "creature card" there.
        conds.push(Condition::SelMatches(exiled, f));
    }
    Some(Effect::If {
        cond: Condition::And(conds),
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "reflexive: when a card is exiled this way", priority: 0, parse: when_exiled_this_way } }
