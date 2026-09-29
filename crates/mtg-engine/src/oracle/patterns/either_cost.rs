//! An activated ability with a choice of two costs, "[Cost A] or [Cost B]: [Effect]."
//! (Crystal Shard: "{3}, {T} or {U}, {T}: Return target creature to its owner's hand
//! unless its controller pays {1}."). Its controller pays either cost, not both, to
//! activate it (CR 602.2b, 601.2f–h): it's compiled as one activated ability per cost,
//! each with the same effect.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn either_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost, effect) = crate::oracle::split_cost(text)?;
    let (a, b) = cost.split_once(" or ")?;
    let (a, b) = (a.trim(), b.trim());
    // Both sides are complete costs ("{3}, {T}" and "{U}, {T}"), not a cost that
    // mentions "or" ("Sacrifice an artifact or creature").
    if !a.starts_with('{') || !b.starts_with('{') {
        return None;
    }
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
