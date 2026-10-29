//! Oracle patterns for mana abilities (CR 605): "{T}: Add {G} for each creature you
//! control" and triggered mana abilities ("Whenever enchanted land is tapped for mana, its
//! controller adds an additional {G}", "Whenever a player taps a land for mana, that player
//! adds one mana of any type that land produced").

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn single_symbol(s: &str) -> Option<(ManaType, &str)> {
    let r = s.strip_prefix('{')?;
    let (sym, rest) = r.split_once('}')?;
    if sym.len() != 1 {
        return None;
    }
    let t = ManaType::from_letter(sym.to_uppercase().chars().next()?)?;
    Some((t, rest))
}

/// "add {G} for each [objects]" (e.g. Gaea's Cradle).
fn add_for_each(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("add ")?;
    let (t, rest) = single_symbol(r)?;
    let rest = rest.strip_prefix(" for each ")?;
    let (f, _, tail) = parse_object_phrase(rest)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::Amount(t, Value::Count(f)),
        restriction: None,
    })
}

/// The trigger condition of a "tapped for mana" ability, and who "that player" / "its
/// controller" is.
fn tapped_for_mana_condition(l: &str) -> Option<TriggerCond> {
    if let Some(r) = l.strip_prefix("whenever enchanted ") {
        let what = r.strip_suffix(" is tapped for mana")?;
        let (f, _, tail) = parse_object_phrase(what)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(TriggerCond::TappedForMana {
            who: PlayerRel::Any,
            filter: Filter::And(vec![Filter::AttachedToSource, f]),
        });
    }
    for (prefix, yours) in [
        ("whenever a player taps ", false),
        ("whenever you tap ", true),
    ] {
        if let Some(r) = l.strip_prefix(prefix) {
            let what = r.strip_suffix(" for mana")?;
            let what = what
                .strip_prefix("a ")
                .or_else(|| what.strip_prefix("an "))
                .unwrap_or(what);
            let (f, _, tail) = parse_object_phrase(what)?;
            if !end(tail).is_empty() {
                return None;
            }
            let f = if yours { f.you_control() } else { f };
            return Some(TriggerCond::TappedForMana {
                who: PlayerRel::Any,
                filter: f,
            });
        }
    }
    None
}

fn tapped_for_mana_effect(l: &str) -> Option<Effect> {
    let (who, r) = if let Some(r) = l.strip_prefix("its controller adds ") {
        (PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)), r)
    } else if let Some(r) = l.strip_prefix("that player adds ") {
        (PlayerRef::TriggerPlayer, r)
    } else if let Some(r) = l.strip_prefix("add ") {
        (PlayerRef::You, r)
    } else {
        return None;
    };
    let r = r.strip_prefix("an additional ").unwrap_or(r);
    let r = end(r);
    let mana = if r == "one mana of any color" {
        ManaProduction::AnyOneColor(Value::c(1))
    } else if r == "one mana of any type that land produced"
        || r == "one mana of any type that permanent produced"
    {
        ManaProduction::AnyTypeProduced
    } else {
        let mut types = Vec::new();
        let mut rest = r;
        while !rest.is_empty() {
            let (t, x) = single_symbol(rest)?;
            types.push(t);
            rest = x;
        }
        if types.is_empty() {
            return None;
        }
        ManaProduction::Fixed(types)
    };
    Some(Effect::AddMana {
        who,
        mana,
        restriction: None,
    })
}

/// A triggered mana ability (CR 605.1b): it triggers from a mana ability (being tapped for
/// mana), has no target, and could add mana.
fn tapped_for_mana(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let (cond, eff) = lower.split_once(", ")?;
    let trigger = tapped_for_mana_condition(cond)?;
    let effect = tapped_for_mana_effect(eff)?;
    let mut tr = TriggeredAbility::new(trigger, Body::effect(effect));
    tr.is_mana_ability = true;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { EffectPattern { name: "add mana for each", priority: 0, parse: add_for_each } }
inventory::submit! { AbilityPattern { name: "tapped for mana triggers", priority: 0, parse: tapped_for_mana } }
