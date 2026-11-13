//! Oracle patterns for keyword costs the generic keyword parser doesn't handle:
//!
//! * cumulative upkeep (CR 702.24a): "Cumulative upkeep {G} or {W}" — a choice made
//!   separately for each age counter, the same as paying a hybrid {G/W} for each one;
//!   "Cumulative upkeep—Pay {B} and 1 life."; "Cumulative upkeep—Draw a card." (an action
//!   performed as the cost, once for each age counter);
//! * ward (CR 702.21a): "Ward—You get two poison counters." (an action the player who
//!   pays the cost performs).

use super::AbilityPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::keywords::parse_keyword_cost;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "{g} or {w}" → the hybrid symbol {G/W}.
fn either_mana(s: &str) -> Option<Cost> {
    let (a, b) = s.trim().split_once(" or ")?;
    let sym = |x: &str| {
        let x = x.trim();
        let inner = x.strip_prefix('{')?.strip_suffix('}')?;
        (inner.len() == 1 && "wubrg".contains(inner)).then(|| inner.to_uppercase())
    };
    let (a, b) = (sym(a)?, sym(b)?);
    Some(Cost::mana(ManaCost::parse(&format!("{{{a}/{b}}}"))?))
}

/// "pay {b} and 1 life" → {B} + pay 1 life.
fn pay_and(s: &str) -> Option<Cost> {
    let r = s.strip_prefix("pay ")?;
    let (a, b) = r.split_once(" and ")?;
    let mut cost = parse_keyword_cost(a)?;
    let rest = parse_keyword_cost(&format!("pay {b}"))?;
    match (&mut cost.mana, rest.mana) {
        (Some(m), Some(n)) => m.add(&n),
        (None, Some(n)) => cost.mana = Some(n),
        _ => {}
    }
    cost.parts.extend(rest.parts);
    Some(cost)
}

/// An action performed as a cost ("draw a card", "an opponent gains 1 life"). Costs
/// can't have targets.
fn action_cost(s: &str, ctx: &CompileContext) -> Option<Cost> {
    let mut b = Builder::new(ctx);
    let e = parse_effect_text(s, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    Some(Cost {
        mana: None,
        parts: vec![CostPart::Effect(Box::new(e))],
    })
}

/// "Ward—[action]": the player paying the cost performs the action ("you" is that
/// player; "get five poison counters" means they get them).
fn ward_action(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let rest = lower.strip_prefix("ward—")?.trim().trim_end_matches('.');
    if parse_keyword_cost(rest).is_some() {
        return None;
    }
    let action = if rest.starts_with("get ") {
        format!("you {rest}")
    } else {
        rest.to_string()
    };
    let cost = action_cost(&action, ctx)?;
    let kw = Keyword {
        cost: Some(cost),
        text: Some(SmolStr::new(text)),
        ..Keyword::new(KeywordKind::Ward)
    };
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), text)])
}

inventory::submit! { AbilityPattern { name: "k702.21 ward action costs", priority: 60, parse: ward_action } }

fn cumulative_upkeep(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let rest = lower.strip_prefix("cumulative upkeep")?;
    let rest = rest.trim().trim_end_matches('.');
    // Costs the keyword parser understands are left to it.
    if parse_keyword_cost(rest).is_some() {
        return None;
    }
    let cost = if let Some(r) = rest.strip_prefix('—') {
        let r = r.trim();
        pay_and(r).or_else(|| action_cost(r, ctx))?
    } else {
        either_mana(rest)?
    };
    let kw = Keyword {
        cost: Some(cost),
        text: Some(SmolStr::new(text)),
        ..Keyword::new(KeywordKind::CumulativeUpkeep)
    };
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), text)])
}

inventory::submit! { AbilityPattern { name: "k702.24 cumulative upkeep costs", priority: 60, parse: cumulative_upkeep } }
