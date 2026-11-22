//! Triggers whose subject is the source or another object: "Whenever ~ or an enchanted
//! creature you control deals combat damage to a player" (Calix, Guided by Fate).

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "~ or [a | an | another] [objects] deals combat damage to a player".
fn source_or_other_deals_combat_damage(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let x = end(r)
        .strip_suffix(" deals combat damage to a player")?
        .strip_prefix("~ or ")?;
    let x = x
        .strip_prefix("a ")
        .or_else(|| x.strip_prefix("an "))
        .unwrap_or(x);
    let (f, _, tail) = parse_object_phrase(x)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::DealsDamage {
            source: Filter::Or(vec![Filter::Source, f]),
            to: DamageRecipient::Player(PlayerRel::Any),
            combat_only: true,
        },
        Sel::TriggerOtherObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "~ or [objects] deals combat damage to a player", priority: 100, parse: source_or_other_deals_combat_damage } }
