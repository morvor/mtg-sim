//! Oracle patterns for the keywords of CR 702.179–702.195 (max speed is in
//! `k702_178_max_speed.rs`):
//!
//! * "your speed increases by N" / "increase your speed by N" (CR 702.179c);
//! * "[card] gains harmonize until end of turn. Its harmonize cost is equal to its mana
//!   cost." (CR 702.180a);

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether the effect grants the keyword `kind` without a cost of its own ("gains
/// harmonize until end of turn").
fn grants_costless(e: &Effect, kind: KeywordKind) -> bool {
    match e {
        Effect::Modify { mods, .. } => mods.iter().any(|m| {
            matches!(m, Modification::AddKeyword(k) if k.kind == kind && k.cost.is_none())
        }),
        Effect::Seq(v) => v.iter().any(|e| grants_costless(e, kind)),
        Effect::If { then, .. } => grants_costless(then, kind),
        Effect::May { effect, .. } | Effect::ForEach { effect, .. } => {
            grants_costless(effect, kind)
        }
        _ => false,
    }
}

/// "Its harmonize cost is equal to its mana cost.": a granted harmonize ability without
/// a cost of its own is cast for the card's mana cost (see `kw/harmonize.rs`).
fn harmonize_cost_is_mana_cost(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    matches!(
        end(s),
        "its harmonize cost is equal to its mana cost"
            | "the harmonize cost is equal to its mana cost"
            | "the harmonize cost is equal to that card's mana cost"
    ) && grants_costless(prev, KeywordKind::Harmonize)
}

inventory::submit! {
    FollowupPattern { name: "k702.180: harmonize cost equal to mana cost", priority: 50, apply: harmonize_cost_is_mana_cost }
}

/// "your speed increases by N", "increase your speed by N" (CR 702.179c).
fn speed_increases(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("your speed increases by ")
        .or_else(|| l.strip_prefix("increase your speed by "))?;
    let (n, tail) = parse_number(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let n = n.as_const().filter(|n| *n > 0)?;
    Some(crate::kw::start_your_engines::increase(n as u32))
}

inventory::submit! { EffectPattern { name: "k702.179 speed increases", priority: 60, parse: speed_increases } }
