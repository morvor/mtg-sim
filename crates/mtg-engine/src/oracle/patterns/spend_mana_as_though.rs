//! Spending mana as though it were mana of any color for some costs (CR 609.4b): "You may
//! spend mana as though it were mana of any color to cast planeswalker spells." (Oath of
//! Nissa), "... to pay the activation costs of ~'s abilities" (Manascape Refractor), "You
//! may spend blue mana as though it were mana of any color to pay the activation costs of
//! ~'s abilities." (Quicksilver Elemental), "... to activate abilities of creatures you
//! control" (Agatha's Soul Cauldron), "... to cast spells you don't own or to activate
//! abilities of permanents you control but don't own" (Nathan Drake, Treasure Hunter):
//! `StaticEffect::SpendAsAnyColor` for those spells or abilities only; colored mana
//! symbols of those costs can be paid with that mana (see `Game::any_color_mana`).

use super::StaticPattern;
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// "mana", "white mana", "blue mana", "colorless mana": the mana that may be spent so
/// (empty: any).
fn mana_kind(s: &str) -> Option<Vec<ManaType>> {
    Some(match s {
        "mana" => vec![],
        "white mana" => vec![ManaType::W],
        "blue mana" => vec![ManaType::U],
        "black mana" => vec![ManaType::B],
        "red mana" => vec![ManaType::R],
        "green mana" => vec![ManaType::G],
        "colorless mana" => vec![ManaType::C],
        _ => return None,
    })
}

/// "[quality] spells", "spells you don't own" as a filter of spells.
fn spells(s: &str) -> Option<Filter> {
    if s == "spells" {
        return Some(Filter::Any);
    }
    let card = if let Some(r) = s.strip_suffix(" spells") {
        format!("{r} cards")
    } else {
        s.replacen("spells ", "cards ", 1)
    };
    let (f, _, tail) = parse_object_phrase(&card)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(f)
}

/// What the mana may be spent for: "to cast [spells]", "to pay the activation costs of
/// ~'s abilities", "to activate abilities of [permanents]".
fn applies_to(s: &str) -> Option<CostTarget> {
    if let Some(r) = s.strip_prefix("cast ") {
        return Some(CostTarget::Spells(spells(r)?));
    }
    if s == "pay the activation costs of ~'s abilities" || s == "activate ~'s abilities" {
        return Some(CostTarget::Abilities(Filter::Source));
    }
    let r = s.strip_prefix("activate abilities of ")?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(CostTarget::Abilities(f))
}

fn spend_as_any_color_for(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let r = end(l).trim().strip_prefix("you may spend ")?;
    let (kind, r) = r.split_once(" as though it were mana of any color to ")?;
    let types = mana_kind(kind)?;
    let mut targets = Vec::new();
    for part in r.split(" or to ") {
        targets.push(applies_to(part.trim())?);
    }
    Some(
        targets
            .into_iter()
            .map(|applies_to| {
                AbilityDef::new(
                    AbilityKind::Static(StaticAbility::new(StaticEffect::SpendAsAnyColor {
                        applies_to,
                        types: types.clone(),
                    })),
                    text,
                )
            })
            .collect(),
    )
}

inventory::submit! { StaticPattern { name: "spend [mana] as though it were mana of any color to cast/activate [what]", priority: 120, parse: spend_as_any_color_for } }
