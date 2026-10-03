//! Cost changes for the spells a player casts that depend on a number (CR 601.2f):
//! "Noncreature spells you cast cost {X} less to cast, where X is your speed." (Samut, the
//! Driving Force), "Creature spells you cast cost {1} less to cast for each +1/+1 counter
//! on ~." The number is counted as the total cost is determined.

use super::StaticPattern;
use crate::ability::*;
use crate::mana::{ManaCost, ManaSymbol};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;
use crate::oracle::CompileContext;

fn spells_cost_counted(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (spells, rest) = l.split_once(" cost ")?;
    let (who, spells) = if let Some(s) = spells.strip_suffix(" you cast") {
        (PlayerRel::You, s)
    } else if let Some(s) = spells.strip_suffix(" your opponents cast") {
        (PlayerRel::Opponent, s)
    } else {
        return None;
    };
    let (filter, plural, tail) = parse_object_phrase(spells)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    let close = rest.find('}')?;
    let mana = ManaCost::parse(&rest[..=close])?;
    let r = rest[close + 1..].trim_start();
    let (more, r) = if let Some(r) = r.strip_prefix("less to cast") {
        (false, r)
    } else {
        (true, r.strip_prefix("more to cast")?)
    };
    let amount = match mana.symbols.as_slice() {
        [ManaSymbol::X] => {
            // "{X} less to cast, where X is [value]".
            let v = r.strip_prefix(", where x is ")?;
            let (v, rest) = parse_value_phrase(v, &mut Builder::new(ctx))?;
            if !end(&rest).is_empty() {
                return None;
            }
            v
        }
        [ManaSymbol::Generic(n)] => {
            // "{1} less to cast for each [thing counted]".
            let fe = end(r.strip_prefix(" for each ")?);
            let history = super::value_results::whole_history_count(fe);
            if history.is_none() && (fe.contains(" this turn") || fe.contains("target")) {
                return None;
            }
            let t = match history {
                Some(v) => v,
                None => super::statics::parse_for_each(fe, Some(&Sel::This))?,
            };
            match *n {
                1 => t,
                n => Value::Mul(Box::new(Value::c(n as i32)), Box::new(t)),
            }
        }
        _ => return None,
    };
    let change = if more {
        CostChange::IncreaseGeneric(amount)
    } else {
        CostChange::ReduceGeneric(amount)
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Spells(filter),
                who,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "spells you cast cost {X} less, where X is / for each", priority: 100, parse: spells_cost_counted } }
