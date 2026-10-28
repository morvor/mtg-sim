//! Referent and value grammar V (cards4-referents-statics): results of earlier steps,
//! instead alternatives and values.
//!
//! - "Search your library for a basic land card, reveal it, put it into your hand, then
//!   shuffle. You may put that card onto the battlefield instead of putting it into your
//!   hand if a creature died this turn." (Caravan Vigil): if the condition holds, the
//!   searcher may put the found card onto the battlefield instead, chosen as it's put
//!   ([`SearchDest::or`], CR 608.2c).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The last instruction of an effect, mutably, past the bookkeeping that records its
/// referents ([`Effect::Store`]).
fn last_mut(e: &mut Effect) -> &mut Effect {
    let i = match &*e {
        Effect::Seq(v) => v.iter().rposition(|x| !matches!(x, Effect::Store { .. })),
        _ => None,
    };
    match (e, i) {
        (Effect::Seq(v), Some(i)) => last_mut(&mut v[i]),
        (other, _) => other,
    }
}

/// "You may put that card onto the battlefield instead of putting it into your hand if
/// [condition]." and "If [condition], you may put that card onto the battlefield instead
/// of putting it into your hand." after a search that puts one card into its searcher's
/// hand.
fn search_destination_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    const ALT: &str = "you may put that card onto the battlefield instead of putting it into your hand";
    let l = end(l);
    let cond_s = if let Some(r) = l.strip_prefix(ALT) {
        match r.strip_prefix(" if ") {
            Some(c) => c,
            None => return false,
        }
    } else if let Some(r) = l.strip_prefix("if ") {
        match r.strip_suffix(&format!(", {ALT}")) {
            Some(c) => c,
            None => return false,
        }
    } else {
        return false;
    };
    let Some(cond) = crate::oracle::statics::parse_condition(cond_s, b.ctx) else {
        return false;
    };
    let last = last_mut(prev);
    // The plain one-card search, as the general search it is.
    let plain_search = matches!(&*last, Effect::Search {
        who: PlayerRef::You,
        whose: PlayerRef::You,
        count,
        to,
        ..
    } if count.as_const() == Some(1) && to.zone == ZoneKind::Hand);
    if let (
        true,
        Effect::Search {
            filter,
            count,
            to,
            reveal,
            shuffle,
            ..
        },
    ) = (plain_search, &*last)
    {
        let filter = match filter {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|f| !matches!(f, Filter::InZone(ZoneKind::Library)))
                    .cloned()
                    .collect(),
            ),
            f => f.clone(),
        };
        *last = Effect::SearchCards(Box::new(SearchSpec {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            zones: vec![ZoneKind::Library],
            zones_optional: false,
            parts: vec![SearchPart {
                filter,
                count: count.clone(),
                up_to: false,
                all: false,
            }],
            distinct_names: false,
            optional: false,
            reveal: *reveal,
            dests: vec![SearchDest {
                count: None,
                to: to.clone(),
                or: vec![],
            }],
            shuffle: if *shuffle {
                SearchShuffle::After
            } else {
                SearchShuffle::No
            },
        }));
    }
    let Effect::SearchCards(spec) = last else {
        return false;
    };
    // One card, found by you, put into your hand.
    let one = spec.parts.len() == 1
        && spec.parts[0].count.as_const() == Some(1)
        && matches!(spec.who, PlayerRef::You);
    let [dest] = spec.dests.as_slice() else {
        return false;
    };
    if !one || dest.to.zone != ZoneKind::Hand || !dest.or.is_empty() {
        return false;
    }
    let mut alt = spec.clone();
    alt.dests[0].or.push(Destination::battlefield());
    let plain = std::mem::replace(last, Effect::Noop);
    *last = Effect::If {
        cond,
        then: Box::new(Effect::SearchCards(alt)),
        otherwise: Box::new(plain),
    };
    true
}

inventory::submit! { FollowupPattern { name: "referents 5: you may put that card onto the battlefield instead of putting it into your hand if [condition]", priority: 100, apply: search_destination_instead } }
