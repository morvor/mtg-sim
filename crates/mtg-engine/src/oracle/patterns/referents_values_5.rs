//! Referent and value grammar V (cards4-referents-statics): results of earlier steps,
//! instead alternatives and values.
//!
//! - "Search your library for a basic land card, reveal it, put it into your hand, then
//!   shuffle. You may put that card onto the battlefield instead of putting it into your
//!   hand if a creature died this turn." (Caravan Vigil): if the condition holds, the
//!   searcher may put the found card onto the battlefield instead, chosen as it's put
//!   ([`SearchDest::or`], CR 608.2c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// [`Builder::named`] key for the players of a "one or more [objects] deal damage to
/// [players]" trigger's batch (`PlayerRef::TriggerPlayers`, CR 603.2c).
pub const BATCH_PLAYERS: &str = "\u{1}batch players";

fn named(b: &Builder, key: &str) -> Option<Sel> {
    b.named
        .iter()
        .rev()
        .find(|(n, _)| n == key)
        .map(|(_, s)| s.clone())
}

/// Whether `r` ends a phrase.
fn phrase_end(r: &str) -> bool {
    r.is_empty() || r.starts_with([' ', ',', '.'])
}

/// The value grammar's atoms of this file:
/// - "the number of opponents dealt damage this way" (Hordewing Skaab): the players of
///   the triggering batch of damage events, each once.
pub fn atom(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    for p in [
        "the number of opponents dealt damage this way",
        "the number of players dealt damage this way",
        "the number of opponents dealt combat damage this way",
        "the number of players dealt combat damage this way",
    ] {
        if let Some(r) = s.strip_prefix(p).filter(|r| phrase_end(r)) {
            let players = named(b, BATCH_PLAYERS)?;
            return Some((Value::CountSel(Box::new(players)), r.to_string()));
        }
    }
    None
}

/// Replaces each JSON value equal to `from` by `to`; the number replaced.
fn replace_json(v: &mut serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> usize {
    if v == from {
        *v = to.clone();
        return 1;
    }
    match v {
        serde_json::Value::Array(a) => a.iter_mut().map(|x| replace_json(x, from, to)).sum(),
        serde_json::Value::Object(m) => m.values_mut().map(|x| replace_json(x, from, to)).sum(),
        _ => 0,
    }
}

/// "exile the top card of each of those opponents' libraries" (Breeches, Brazen
/// Plunderer): the players of the triggering batch, read as "each opponent" is.
fn each_of_those_opponents(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("each of those opponents") || l.contains("each opponent") {
        return None;
    }
    named(b, BATCH_PLAYERS)?;
    let plain = l
        .replace("each of those opponents' libraries", "each opponent's library")
        .replace("each of those opponents' hands", "each opponent's hand")
        .replace("each of those opponents' graveyards", "each opponent's graveyard")
        .replace("each of those opponents", "each opponent");
    let e = parse_sentence(&plain, b)?;
    let mut json = serde_json::to_value(&e).ok()?;
    let from = serde_json::to_value(PlayerRef::EachOpponent).ok()?;
    let to = serde_json::to_value(PlayerRef::TriggerPlayers).ok()?;
    if replace_json(&mut json, &from, &to) == 0 {
        return None;
    }
    // Only "each opponent" as a player reference: a filter's opponents
    // (`PlayerFilter::Opponent`) would stay all opponents.
    if serde_json::to_string(&json).ok()?.contains("Opponent") {
        return None;
    }
    serde_json::from_value(json).ok()
}

inventory::submit! { EffectPattern { name: "referents 5: each of those opponents (the players of the batch)", priority: 100, parse: each_of_those_opponents } }

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

/// "Whenever one or more creature tokens you control deal combat damage to a player,
/// exile target instant or sorcery card ... from their graveyard." (Soundwave, Sonic
/// Spy): "their" is the player dealt damage. The objects dealing it are yours, whose
/// graveyard the text would call "your graveyard".
fn from_their_graveyard_damaged_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !b.in_trigger
        || !matches!(b.it_player, PlayerRef::TriggerPlayer)
        || !l.contains(" from their graveyard")
        || l.contains("that player")
        || l.contains("target player")
        || l.contains("target opponent")
    {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if !(raw.contains(" you control deal combat damage to a player,")
        || raw.contains(" you control deal damage to a player,"))
    {
        return None;
    }
    parse_sentence(
        &l.replace(" from their graveyard", " from that player's graveyard"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "referents 5: from their graveyard (the player your objects dealt damage to)", priority: 100, parse: from_their_graveyard_damaged_player } }
