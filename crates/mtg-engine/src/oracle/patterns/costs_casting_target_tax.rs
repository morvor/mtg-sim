//! Cost changes for spells that target something (CR 601.2f): "Spells your opponents
//! cast that target this creature cost {2} more to cast.", "Spells your opponents cast
//! that target a Merfolk you control cost {2} more to cast." The spell's targets are
//! chosen before its total cost is determined (CR 601.2c), so the change applies to a
//! spell with at least one such target.

use super::StaticPattern;
use crate::ability::*;
use crate::mana::{ManaCost, ManaSymbol};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "~" or "a Merfolk you control": what a targeted object must be.
fn targeted(s: &str) -> Option<Filter> {
    if s == "~" {
        return Some(Filter::Source);
    }
    let r = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail).is_empty().then_some(f)
}

/// "Spells [your opponents|you] cast that target [object] cost {N} more/less to cast."
fn targeting_spells_cost(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (spells, rest) = l.split_once(" cost {")?;
    let r = spells.strip_prefix("spells ")?;
    let (who, r) = if let Some(r) = r.strip_prefix("your opponents cast ") {
        (PlayerRel::Opponent, r)
    } else if let Some(r) = r.strip_prefix("you cast ") {
        (PlayerRel::You, r)
    } else {
        return None;
    };
    let what = targeted(r.strip_prefix("that target ")?)?;
    let close = rest.rfind('}')?;
    let mana = ManaCost::parse(&format!("{{{}", &rest[..=close]))?;
    if mana.has_x() {
        return None;
    }
    let more = match end(&rest[close + 1..]) {
        "more to cast" => true,
        "less to cast" => false,
        _ => return None,
    };
    let generic_only = mana
        .symbols
        .iter()
        .all(|s| matches!(s, ManaSymbol::Generic(_)));
    let n = Value::c(mana.generic_amount() as i32);
    let change = match (more, generic_only) {
        (true, true) => CostChange::IncreaseGeneric(n),
        (true, false) => CostChange::IncreaseMana(mana),
        (false, true) => CostChange::ReduceGeneric(n),
        (false, false) => CostChange::ReduceMana {
            mana,
            colored_only: false,
        },
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Spells(Filter::and(vec![
                    Filter::Spell,
                    Filter::Targets(Box::new(what)),
                ])),
                who,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "costs_casting: spells that target it cost more", priority: 80, parse: targeting_spells_cost } }
