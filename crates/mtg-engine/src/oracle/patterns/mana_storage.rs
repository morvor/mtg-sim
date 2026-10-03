//! Storage lands and mana batteries (CR 107.1c, 118.3): "{T}, Remove any number of
//! storage counters from ~: Add {W} for each storage counter removed this way.", "{T},
//! Remove any number of charge counters from ~: Add {B}, then add an additional {B} for
//! each charge counter removed this way."
//!
//! The number of counters removed is chosen as the cost is paid; it's the X of the
//! ability, so "Remove any number of [kind] counters from ~" is "Remove X [kind] counters
//! from ~" and "for each [kind] counter removed this way" counts X.

use super::AbilityPattern;
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::CompileContext;

/// "{w}" → W.
fn one_symbol(s: &str) -> Option<ManaType> {
    let inner = s.strip_prefix('{')?.strip_suffix('}')?;
    let mut cs = inner.chars();
    let c = cs.next()?;
    if cs.next().is_some() {
        return None;
    }
    ManaType::from_letter(c.to_ascii_uppercase())
}

fn storage_mana(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost_s, eff_s) = crate::oracle::split_cost(block.trim())?;
    let cost_l = cost_s.to_lowercase();
    let eff_l = eff_s.trim().to_lowercase();
    // The cost defines X; it mustn't have another.
    if cost_l.contains("{x}") || cost_l.contains(" x ") {
        return None;
    }
    let i = cost_l.find("remove any number of ")?;
    let after = &cost_l[i + "remove any number of ".len()..];
    let (kind, tail) = after.split_once(' ')?;
    if !tail.starts_with("counters from ~") {
        return None;
    }
    let (cost, loyalty) =
        crate::oracle::costs::parse_cost(&format!("{}remove x {kind} {tail}", &cost_l[..i]))?;
    if loyalty {
        return None;
    }
    let per = format!(" for each {kind} counter removed this way.");
    let r = eff_l.strip_prefix("add ")?.strip_suffix(per.as_str())?;
    let mana = if let Some((first, extra)) = r.split_once(", then add an additional ") {
        // "{b}, then add an additional {b} for each ...": one plus X.
        let t = one_symbol(first)?;
        if one_symbol(extra)? != t {
            return None;
        }
        ManaProduction::Amount(t, Value::Sum(vec![Value::c(1), Value::X]))
    } else if r == "one mana of any color" {
        ManaProduction::AnyCombination(Value::X)
    } else {
        ManaProduction::Amount(one_symbol(r)?, Value::X)
    };
    let mut act = ActivatedAbility::new(
        cost,
        Body::effect(Effect::AddMana {
            who: PlayerRef::You,
            mana,
            restriction: None,
        }),
    );
    // CR 605.1a: no target, could add mana, not a loyalty ability.
    act.is_mana_ability = true;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "mana: counters removed this way", priority: 60, parse: storage_mana } }
