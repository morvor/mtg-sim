//! Oracle patterns for cost changes (CR 118.7): "[Spells] you cast cost {W}{B} less to
//! cast", including "instant and sorcery spells" and "This effect reduces only the amount
//! of colored mana you pay".

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::mana::{ManaCost, ManaSymbol};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::types::CardType;

fn spells_filter(s: &str) -> Option<Filter> {
    let s = s.trim();
    if s == "spells" {
        return Some(Filter::Any);
    }
    if s == "instant and sorcery spells" {
        return Some(Filter::Or(vec![
            Filter::Type(CardType::Instant),
            Filter::Type(CardType::Sorcery),
        ]));
    }
    let (f, _, tail) = parse_object_phrase(s)?;
    end(tail).is_empty().then_some(f)
}

/// "[Spells] you cast cost [mana] less/more to cast[. This effect reduces only the amount
/// of colored mana you pay]".
fn mana_cost_modifier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (l, colored_only) =
        match l.strip_suffix(". this effect reduces only the amount of colored mana you pay") {
            Some(r) => (r, true),
            None => (l, false),
        };
    let (spells, rest) = l.split_once(" cost {")?;
    let (who, spells) = if let Some(s) = spells.strip_suffix(" you cast") {
        (PlayerRel::You, s)
    } else if let Some(s) = spells.strip_suffix(" your opponents cast") {
        (PlayerRel::Opponent, s)
    } else {
        (PlayerRel::Any, spells)
    };
    let filter = spells_filter(spells)?;
    let close = rest.rfind('}')?;
    let mana = ManaCost::parse(&format!("{{{}", &rest[..=close]))?;
    let more = match end(&rest[close + 1..]) {
        "less to cast" => false,
        "more to cast" => true,
        _ => return None,
    };
    let generic_only = mana
        .symbols
        .iter()
        .all(|s| matches!(s, ManaSymbol::Generic(_)));
    let change = match (more, generic_only && !colored_only) {
        (true, true) => CostChange::IncreaseGeneric(Value::c(mana.generic_amount() as i32)),
        (true, false) => CostChange::IncreaseMana(mana),
        (false, true) => CostChange::ReduceGeneric(Value::c(mana.generic_amount() as i32)),
        (false, false) => CostChange::ReduceMana { mana, colored_only },
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Spells(Filter::and(vec![filter, Filter::Spell])),
                who,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "spells cost mana less", priority: 0, parse: mana_cost_modifier } }

/// "mana of any type can be spent to cast that spell" / "... to cast it" / "... to cast
/// them" (CR 118.14): applies to the cards the preceding permission refers to.
fn any_type_mana(l: &str, b: &mut Builder) -> Option<Effect> {
    let rest = end(l).strip_prefix("mana of any type can be spent to cast ")?;
    if !matches!(rest, "that spell" | "it" | "them" | "those spells") {
        return None;
    }
    Some(Effect::SpendAnyTypeMana {
        who: PlayerRef::You,
        what: b.it.clone(),
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "mana of any type can be spent", priority: 0, parse: any_type_mana } }
