//! Cost changes and alternative costs for activated abilities (CR 602.2b, 601.2f, 118.9);
//! see `activation_costs.rs`:
//! * "Activated abilities [of creatures you control] cost {2} less to activate [unless
//!   they're mana abilities]. [This effect can't reduce the mana in that cost to less
//!   than one mana.]" (Training Grounds, Heartstone, Suppression Field),
//! * "Enchanted artifact's activated abilities cost {2} less to activate. ..." (Power
//!   Artifact),
//! * "Abilities you activate that aren't mana abilities cost {2} less to activate. ..."
//!   (Zirda, the Dawnwaker),
//! * "Abilities your opponents activate that target a Merfolk you control cost {2} more to
//!   activate." (Kopala, Warden of Waves),
//! * "Equip abilities you activate that target ~ cost {2} less to activate." (Dwarven
//!   Mauler), "Activated abilities of Equipment you control that target ~ cost {2} less
//!   to activate." (Bladegraft Aspirant),
//! * "You may pay {0} rather than pay the equip cost of the first equip ability you
//!   activate [each turn | during each of your turns]." (Kíli the Resourceful, Forge
//!   Anew), "... the unearth cost of the first unearth ability you activate each turn."
//!   (Highway Reaver), "You may pay {0} rather than pay cycling costs." (New
//!   Perspectives).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

const FLOOR: &str = ". this effect can't reduce the mana in that cost to less than one mana";

/// "{2}" → 2.
fn generic_amount(s: &str) -> Option<u32> {
    s.strip_prefix('{')?.strip_suffix('}')?.parse().ok()
}

/// A group of sources: "creatures you control", "~", "enchanted artifact". Without
/// "card", it names permanents (CR 110.1): the abilities of creature cards in other zones
/// (cycling, unearth) aren't affected.
pub(crate) fn sources(s: &str) -> Option<Filter> {
    if s == "~" {
        return Some(Filter::Source);
    }
    // "sources with the chosen name": objects in any zone (Pithing Needle-style).
    if s == "sources with the chosen name" {
        return Some(Filter::ChosenName);
    }
    // "that target a Merfolk you control".
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(if s.contains("card") {
        f
    } else {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
    })
}

/// "{N} less to activate" / "{N} more to activate" (with the floor sentence, if any).
fn change(r: &str, floor: bool, ctx: &CompileContext) -> Option<CostChange> {
    // "{X} less to activate, where X is ~'s power".
    let (r, x) = match r.split_once(", where x is ") {
        Some((head, v)) => {
            let (v, rest) = crate::oracle::statics::parse_value_phrase(
                v,
                &mut crate::oracle::effects::Builder::new(ctx),
            )?;
            if !end(&rest).is_empty() {
                return None;
            }
            (head, Some(v))
        }
        None => (r, None),
    };
    let (amount, less) = if let Some(m) = r.strip_suffix(" less to activate") {
        (m, true)
    } else {
        (r.strip_suffix(" more to activate")?, false)
    };
    let n = match x {
        Some(v) if amount == "{x}" => v,
        Some(_) => return None,
        None => Value::c(generic_amount(amount)? as i32),
    };
    Some(match (less, floor) {
        (true, true) => CostChange::ReduceGenericMinOne(n),
        (true, false) => CostChange::ReduceGeneric(n),
        // The floor only limits reductions.
        (false, false) => CostChange::IncreaseGeneric(n),
        (false, true) => return None,
    })
}

/// "that target [object]" at the end of a subject.
fn split_targeting(s: &str) -> Option<(&str, Option<Filter>)> {
    match s.split_once(" that target ") {
        Some((head, t)) => Some((head, Some(sources(t)?))),
        None => Some((s, None)),
    }
}

fn cost_modifier(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (l, floor) = match l.strip_suffix(FLOOR) {
        Some(r) => (r, true),
        None => (l, false),
    };
    let (l, nonmana_suffix) = match l.strip_suffix(" unless they're mana abilities") {
        Some(r) => (r, true),
        None => (l, false),
    };
    let (subject, rest) = l
        .split_once(" cost ")
        .or_else(|| l.split_once(" costs "))?;
    // "... if it targets a colorless creature".
    let (rest, if_targets) = match rest.split_once(" if it targets ") {
        Some((head, t)) => (head, Some(sources(t)?)),
        None => (rest, None),
    };
    let change = change(rest, floor, ctx)?;
    let mut who = PlayerRel::Any;
    let mut scope;
    if let Some((group, kind)) = subject.split_once("'s ").filter(|(_, k)| {
        k.ends_with(" ability") || k.ends_with(" abilities")
    }) {
        // "~'s equip abilities", "~'s equip ability".
        let kind = kind
            .strip_suffix(" abilities")
            .or_else(|| kind.strip_suffix(" ability"))?;
        if kind == "activated" {
            return None;
        }
        scope = AbilityScope::new(
            sources(group)?,
            AbilityClass::Keyword(KeywordKind::from_name(kind)?),
        );
    } else if let Some(r) = subject
        .strip_prefix("the first activated ability of ")
        .and_then(|r| r.strip_suffix(" you activate each turn"))
    {
        // "The first activated ability of an artifact you activate each turn".
        who = PlayerRel::You;
        scope = AbilityScope::new(sources(r)?, AbilityClass::Any);
        scope.first_each_turn = true;
    } else if let Some(r) = subject.strip_prefix("activated abilities of ") {
        // "Activated abilities of Equipment you control that target ~".
        let (group, targeting) = split_targeting(r)?;
        scope = AbilityScope::new(sources(group)?, AbilityClass::Any);
        scope.targeting = targeting;
    } else if subject == "activated abilities" {
        scope = AbilityScope::new(Filter::Any, AbilityClass::Any);
    } else if let Some(group) = subject.strip_suffix("'s activated abilities") {
        scope = AbilityScope::new(sources(group)?, AbilityClass::Any);
    } else {
        // "Abilities you activate ...", "Equip abilities you activate ...", "Abilities
        // your opponents activate ...".
        let (kind, r) = subject.split_once("abilities ")?;
        let class = match kind.trim() {
            "" => AbilityClass::Any,
            k => AbilityClass::Keyword(KeywordKind::from_name(k)?),
        };
        let r = if let Some(r) = r.strip_prefix("you activate") {
            who = PlayerRel::You;
            r
        } else if let Some(r) = r.strip_prefix("your opponents activate") {
            who = PlayerRel::Opponent;
            r
        } else {
            return None;
        };
        scope = AbilityScope::new(Filter::Any, class);
        let r = match r.strip_prefix(" that aren't mana abilities") {
            Some(r) => {
                scope.nonmana = true;
                r
            }
            None => r,
        };
        if let Some(t) = r.strip_prefix(" that target ") {
            scope.targeting = Some(sources(t)?);
        } else if let Some(g) = r.strip_prefix(" of ") {
            // "Equip abilities you activate of other Equipment".
            scope.sources = sources(g)?;
        } else if !r.is_empty() {
            return None;
        }
    }
    if let Some(t) = if_targets {
        if scope.targeting.is_some() {
            return None;
        }
        scope.targeting = Some(t);
    }
    scope.nonmana |= nonmana_suffix;
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ActivatedAbilities(Box::new(scope)),
        who,
        change,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "activated abilities cost more or less to activate", priority: 0, parse: cost_modifier } }

/// "You may pay {0} rather than pay the equip cost of the first equip ability you activate
/// each turn", "... rather than pay cycling costs".
fn alternative_cost(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (l, your_turn) = match l.strip_prefix("during your turn, ") {
        Some(r) => (r, true),
        None => (l, false),
    };
    let r = l.strip_prefix("you may pay ")?;
    let (pay, r) = r.split_once(" rather than pay ")?;
    let mana = ManaCost::parse(&pay.to_uppercase())?;
    let (kind, first, during_your_turns) = if let Some(k) = r.strip_suffix(" costs") {
        (k, false, false)
    } else {
        let r = r.strip_prefix("the ")?;
        let (k, r) = r.split_once(" cost of the first ")?;
        let (k2, r) = r.split_once(" ability you activate ")?;
        if k != k2 {
            return None;
        }
        let yours = match r {
            "each turn" => false,
            "during each of your turns" => true,
            _ => return None,
        };
        (k, true, yours)
    };
    let kind = KeywordKind::from_name(kind)?;
    let mut scope = AbilityScope::new(Filter::Any, AbilityClass::Keyword(kind));
    scope.first_each_turn = first;
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ActivatedAbilities(Box::new(scope)),
        who: PlayerRel::You,
        change: CostChange::AlternativeCost(Cost::mana(mana)),
    }));
    if your_turn || during_your_turns {
        s.condition = Some(Condition::YourTurn);
    }
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "pay rather than pay an activation cost", priority: 0, parse: alternative_cost } }
