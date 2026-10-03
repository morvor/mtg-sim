//! Oracle pattern for reflexive triggered abilities keyed to a discarded card (CR 603.12):
//! "When you discard a [quality] card this way, [effect]" (Evie Frye: "Draw a card, then
//! discard a card. When you discard a creature card this way, target creature you control
//! can't be blocked this turn."). The ability triggers only if the preceding instruction
//! discarded a card with that quality; it has its own targets, chosen as it's put on the
//! stack. "That card" in it refers to the discarded card.

use super::r600_triggers::reflexive_body;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

fn when_you_discard_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when you discard ")?;
    let (card, effect) = r.split_once(" this way, ")?;
    let card = card
        .strip_prefix("a ")
        .or_else(|| card.strip_prefix("an "))?;
    let filter = if card == "card" {
        None
    } else {
        let (f, _, tail) = parse_object_phrase(card)?;
        if !end(tail).is_empty() {
            return None;
        }
        Some(f)
    };
    // "... equal to that card's mana value" (Narset of the Ancient Way): its mana value.
    let rewritten = effect.contains("that card's");
    let effect = effect.replace("that card's", "its").replace("that card", "it");
    // The rewritten "its" still means that card (see `Builder::its_is_it`).
    let saved = b.its_is_it;
    b.its_is_it = saved || rewritten;
    let body = reflexive_body(&effect, b);
    b.its_is_it = saved;
    let body = body?;
    // The discard just performed stored the discarded cards as "it" and recorded whether
    // anything was discarded (an optional discard that was declined discards nothing).
    let discarded = Sel::Var(vars::IT);
    let mut conds = vec![
        Condition::PrevHappened,
        Condition::SelNonEmpty(discarded.clone()),
    ];
    if let Some(f) = filter {
        conds.push(Condition::SelMatches(discarded, f));
    }
    Some(Effect::If {
        cond: Condition::And(conds),
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "reflexive: when you discard a card this way", priority: 0, parse: when_you_discard_this_way } }
