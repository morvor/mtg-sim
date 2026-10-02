//! "search your library for a card with the same mana value as that card, reveal it, put
//! it into your hand, then shuffle" (Disciple of Deceit, after "you may discard a nonland
//! card. If you do, ..."): the search finds only cards whose mana value equals that of the
//! card the previous instruction affected (`vars::IT`, as it now exists — a discarded
//! card in the graveyard has X = 0, CR 202.3e, and a split card the combined mana value
//! of its halves, CR 709.4b). Compare transmute (CR 702.53a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn search_same_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("search ") {
        return None;
    }
    let (head, tail) = l.split_once(" with the same mana value as ")?;
    let rest = tail
        .strip_prefix("that card")
        .or_else(|| tail.strip_prefix("the discarded card"))?;
    let mut e = parse_clause(&format!("{head}{rest}"), b)?;
    let Effect::Search { filter, .. } = &mut e else {
        return None;
    };
    let same = Filter::ManaValue(
        Cmp::Eq,
        Box::new(Value::ManaValueOf(Box::new(Sel::Var(vars::IT)))),
    );
    *filter = Filter::and(vec![filter.clone(), same]);
    Some(e)
}

inventory::submit! { EffectPattern { name: "search for a card with the same mana value as that card", priority: 70, parse: search_same_mana_value } }
