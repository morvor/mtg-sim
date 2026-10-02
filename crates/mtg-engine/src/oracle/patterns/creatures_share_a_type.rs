//! "if you control three or more creatures that share a creature type" (Littjara
//! Kinseekers), "if you control at least two creatures that share a creature type": some
//! creature type is had by at least that many creatures you control (a creature that is
//! every creature type, CR 702.73a, has each of them).

use super::ConditionPattern;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::oracle::phrases::*;

/// `Condition::Custom` prefix, followed by the number of creatures.
pub const SHARE_A_TYPE: &str = "you control N creatures that share a creature type:";

fn creatures_share_a_type(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    let r = r
        .strip_suffix(" creatures that share a creature type")?
        .trim();
    let num = match r.strip_prefix("at least ") {
        Some(x) => x,
        None => r.strip_suffix(" or more")?,
    };
    let (n, rest) = parse_number(num)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let n = n.as_const()?;
    (n >= 2).then(|| Condition::Custom(format!("{SHARE_A_TYPE}{n}").into()))
}

inventory::submit! { ConditionPattern { name: "you control N creatures that share a creature type", priority: 100, parse: creatures_share_a_type } }

/// Evaluates [`SHARE_A_TYPE`] conditions.
pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    let n: usize = name.strip_prefix(SHARE_A_TYPE)?.parse().ok()?;
    let mine: Vec<&crate::object::Characteristics> = g
        .permanents()
        .filter(|o| o.controller == ctx.controller && o.is_creature())
        .map(|o| &o.chars)
        .collect();
    let every = mine
        .iter()
        .filter(|c| crate::kw::changeling::every_creature_type(c))
        .count();
    // The creature types of the others, each counted once per creature.
    let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
    for c in mine
        .iter()
        .filter(|c| !crate::kw::changeling::every_creature_type(c))
    {
        let mut seen: Vec<&str> = Vec::new();
        for s in c.subtypes.iter().filter(|s| crate::types::is_creature_type(s)) {
            if !seen.contains(&s.as_str()) {
                seen.push(s.as_str());
                *counts.entry(s.as_str()).or_default() += 1;
            }
        }
    }
    let best = counts.values().copied().max().unwrap_or(0);
    // Creatures that are every creature type share any type with the others (and with
    // each other).
    Some(every >= n || (best > 0 && best + every >= n))
}
