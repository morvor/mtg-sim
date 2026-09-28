//! "If it's a[n] [kind of] card, [effect]." about a card just exiled, revealed, or looked
//! at: "Exile the top card of your library. ... If it's an instant or sorcery card, you
//! may cast it without paying its mana cost." (Hidetsugu and Kairi). The condition is
//! checked as that part of the effect happens (CR 608.2c).

use super::card_flow_search::card_filter;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

fn if_its_a_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("if it's a ")
        .or_else(|| l.strip_prefix("if it's an "))?;
    let (desc, rest) = r.split_once(", ")?;
    if !desc.ends_with(" card") {
        return None;
    }
    // "It" is a card an earlier instruction of the effect found.
    if !matches!(b.it, Sel::Var(_)) {
        return None;
    }
    let it = b.it.clone();
    let filter = card_filter(desc, b)?;
    let then = parse_sentence(rest, b)?;
    Some(Effect::If {
        cond: Condition::SelMatches(it, filter),
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "r608 if it's a [kind of] card, [effect]", priority: 100, parse: if_its_a_card } }
