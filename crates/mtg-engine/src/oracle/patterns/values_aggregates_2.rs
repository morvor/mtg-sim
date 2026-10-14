//! Aggregates over players and groups, III.
//!
//! Balance and its relatives: "Each player chooses a number of lands they control equal
//! to the number of lands controlled by the player who controls the fewest, then
//! sacrifices the rest. Players discard cards and sacrifice creatures the same way."
//!
//! Each player keeps as many of the objects as the player with the fewest has and
//! sacrifices the rest: they sacrifice their count minus that least count (choosing the
//! ones to sacrifice is choosing the ones to keep). The players choose in APNAP order and
//! the permanents are sacrificed at the same time (CR 101.4; Balance rulings), as
//! `simultaneous.rs` performs an instruction each player does. "The same way" repeats the
//! process for each other kind, in the order the text lists them; each kind is counted
//! during its own part of the process (Balance ruling).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// Each player's own amount for one part of the process.
const OWN: Var = vars::USER + 12_003;

/// "Each player [sacrifices / discards] down to the least amount any player has."
fn down_to_fewest(own: Value, act: Effect) -> Effect {
    let fewest = Value::OverPlayers(AggOp::Min, PlayerFilter::Any, Box::new(own.clone()));
    Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::seq(vec![
            Effect::StoreValue {
                var: OWN,
                value: Value::Diff(Box::new(own), Box::new(fewest)),
            },
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(act),
            },
        ])),
    }
}

/// The permanents part: "lands", "creatures", "permanents".
fn sacrifice_part(f: Filter) -> Effect {
    let mine = Filter::and(vec![
        f.clone(),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::ControlledBy(PlayerRel::Iterated),
    ]);
    down_to_fewest(
        Value::Count(mine),
        Effect::Sacrifice {
            who: PlayerRef::You,
            filter: f,
            count: Value::Var(OWN),
        },
    )
}

/// The cards-in-hand part.
fn discard_part() -> Effect {
    down_to_fewest(
        Value::HandSize(PlayerRef::Iterated),
        Effect::Discard {
            who: PlayerRef::You,
            n: Value::Var(OWN),
            random: false,
            filter: Filter::Any,
        },
    )
}

/// A plural permanent noun ("lands", "creatures", "permanents").
fn permanents(s: &str) -> Option<Filter> {
    if !matches!(
        s,
        "lands" | "creatures" | "permanents" | "artifacts" | "enchantments"
    ) {
        return None;
    }
    let (f, plural, rest) = parse_object_phrase(s)?;
    (plural && rest.trim().is_empty()).then_some(f)
}

/// "each player chooses a number of lands they control equal to the number of lands
/// controlled by the player who controls the fewest, then sacrifices the rest".
fn keep_as_many_as_the_fewest(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("each player chooses a number of ")?;
    let (noun, r) = r.split_once(" they control equal to the number of ")?;
    let (noun2, r) = r.split_once(" controlled by the player who controls the fewest")?;
    if noun != noun2 || r != ", then sacrifices the rest" {
        return None;
    }
    Some(sacrifice_part(permanents(noun)?))
}

inventory::submit! { EffectPattern { name: "aggregates: each player keeps as many as the player with the fewest", priority: 100, parse: keep_as_many_as_the_fewest } }

/// Whether the effect is a part of this process.
fn is_balance(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(is_balance),
        Effect::ForEachPlayer { effect, .. } => matches!(
            &**effect,
            Effect::Seq(v) if matches!(v.first(), Some(Effect::StoreValue { var: OWN, .. }))
        ),
        _ => false,
    }
}

/// "Players discard cards and sacrifice creatures the same way.", "Each player discards
/// cards the same way."
fn the_same_way(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !is_balance(prev) {
        return false;
    }
    let lower = s.to_lowercase();
    let l = end(&lower);
    let Some(r) = l
        .strip_prefix("players ")
        .or_else(|| l.strip_prefix("each player "))
        .and_then(|r| r.strip_suffix(" the same way"))
    else {
        return false;
    };
    let mut parts = Vec::new();
    for item in r.split(" and ") {
        let part = if matches!(item, "discard cards" | "discards cards") {
            discard_part()
        } else if let Some(noun) = item
            .strip_prefix("sacrifice ")
            .or_else(|| item.strip_prefix("sacrifices "))
        {
            match permanents(noun) {
                Some(f) => sacrifice_part(f),
                None => return false,
            }
        } else {
            return false;
        };
        parts.push(part);
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(std::iter::once(old).chain(parts).collect());
    true
}

inventory::submit! { FollowupPattern { name: "aggregates: [players do the other parts] the same way", priority: 100, apply: the_same_way } }
