//! An activated ability with a choice of two costs, "[Cost A] or [Cost B]: [Effect]."
//! (Crystal Shard: "{3}, {T} or {U}, {T}: Return target creature to its owner's hand
//! unless its controller pays {1}."). Its controller pays either cost, not both, to
//! activate it (CR 602.2b, 601.2f–h): it's compiled as one activated ability per cost,
//! each with the same effect.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

/// The verbs a cost part starts with.
const COST_VERBS: [&str; 8] = [
    "sacrifice ",
    "discard ",
    "exile ",
    "pay ",
    "tap ",
    "return ",
    "remove ",
    "put ",
];

/// "[shared parts], [verb A ...] or [verb B ...]" (Bullseye, Death Dealer: "{3}, {T},
/// Sacrifice an artifact or discard a nonland card"): the two complete costs.
fn either_last_part(cost: &str) -> Option<(String, String)> {
    let (head, last) = match cost.rsplit_once(", ") {
        Some((h, l)) => (Some(h), l),
        None => (None, cost),
    };
    let lower = last.to_lowercase();
    if !COST_VERBS.iter().any(|v| lower.starts_with(v)) {
        return None;
    }
    let i = lower
        .match_indices(" or ")
        .map(|(i, _)| i)
        .find(|i| COST_VERBS.iter().any(|v| lower[i + 4..].starts_with(v)))?;
    let (a, b) = (&last[..i], &last[i + 4..]);
    let b = format!("{}{}", b[..1].to_uppercase(), &b[1..]);
    Some(match head {
        Some(h) => (format!("{h}, {a}"), format!("{h}, {b}")),
        None => (a.to_string(), b),
    })
}

fn either_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost, effect) = crate::oracle::split_cost(text)?;
    // Limits on how often it's activated count both costs' activations together.
    let limited = effect.to_lowercase().contains(" once") || effect.contains("times each");
    let (a, b) = match either_last_part(cost).filter(|_| !limited) {
        Some(ab) => ab,
        None => {
            let (a, b) = cost.split_once(" or ")?;
            let (a, b) = (a.trim(), b.trim());
            // Both sides are complete costs ("{3}, {T}" and "{U}, {T}"), not a cost that
            // mentions "or" ("Sacrifice an artifact or creature").
            if !a.starts_with('{') || !b.starts_with('{') {
                return None;
            }
            (a.to_string(), b.to_string())
        }
    };
    let (a, b) = (a.as_str(), b.as_str());
    crate::oracle::costs::parse_cost(a)?;
    crate::oracle::costs::parse_cost(b)?;
    let mut out = Vec::new();
    for c in [a, b] {
        let abilities = crate::oracle::parse_ability(&format!("{c}: {effect}"), ctx)?;
        if !abilities
            .iter()
            .all(|x| matches!(x.kind, AbilityKind::Activated(_)))
        {
            return None;
        }
        out.extend(abilities);
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "either of two activation costs", priority: 0, parse: either_cost } }
