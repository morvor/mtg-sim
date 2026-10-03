//! Cost changes for two kinds of spells a player casts (CR 601.2f): "Merfolk spells and
//! Wizard spells you cast cost {1} less to cast." (the Lorwyn Bannerets). A spell of both
//! kinds is one spell matching the description: its cost changes once, not twice.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn two_kinds_of_spells_cost(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (spells, rest) = l.split_once(" cost ")?;
    let (who, spells) = if let Some(s) = spells.strip_suffix(" you cast") {
        (PlayerRel::You, s)
    } else if let Some(s) = spells.strip_suffix(" your opponents cast") {
        (PlayerRel::Opponent, s)
    } else {
        return None;
    };
    let (a, b) = spells.split_once(" spells and ")?;
    let mut kinds = Vec::new();
    for s in [format!("{a} spells"), b.to_string()] {
        let (f, plural, tail) = parse_object_phrase(&s)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        kinds.push(f);
    }
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.strip_prefix('{')?.parse().ok()?;
    let change = match end(tail) {
        "more to cast" => CostChange::IncreaseGeneric(Value::c(n)),
        "less to cast" => CostChange::ReduceGeneric(Value::c(n)),
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Spells(Filter::Or(kinds)),
                who,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "[A] spells and [B] spells you cast cost {N} less to cast", priority: 100, parse: two_kinds_of_spells_cost } }
