//! "Mill four cards. You may put up to two creature and/or land cards from among the
//! milled cards into your hand." (Smuggler's Surprise), "... a permanent card from among
//! the cards milled this way ..." (Wasteful Harvest): cards chosen among those the mill
//! put into the graveyard (the new objects there, CR 400.7), or wherever else they went
//! instead if that's a public zone (CR 701.17c).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether the effect ends by milling (which stores the milled cards in `vars::IT`).
fn ends_with_mill(e: &Effect) -> bool {
    match e {
        Effect::Mill { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_mill),
        _ => false,
    }
}

fn put_from_among_milled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (may, r) = match l.strip_prefix("you may put ") {
        Some(r) => (true, r),
        None => match l.strip_prefix("put ") {
            Some(r) => (false, r),
            None => return false,
        },
    };
    let (count, up_to, r) = if let Some(r) = r.strip_prefix("up to ") {
        let Some((n, r)) = parse_number(r) else {
            return false;
        };
        (n, true, r)
    } else if let Some(r) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
        (Value::c(1), may, r)
    } else {
        return false;
    };
    let Some((desc, rest)) = r
        .split_once(" from among the milled cards ")
        .or_else(|| r.split_once(" from among the cards milled this way "))
    else {
        return false;
    };
    if rest != "into your hand" || !ends_with_mill(prev) {
        return false;
    }
    let Some(f) = super::card_flow_search::card_filter(desc.trim(), b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Move {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![f, Filter::In(Box::new(Sel::Var(vars::IT)))]),
                count,
                up_to,
                store: None,
            },
            to: Destination::zone(ZoneKind::Hand),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "card_flow: put cards from among the milled cards into your hand", priority: 90, apply: put_from_among_milled } }
