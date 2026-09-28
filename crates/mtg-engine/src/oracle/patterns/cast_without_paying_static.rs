//! "You may cast spells from your hand without paying their mana costs." (Omniscience),
//! "You may cast Dragon spells without paying their mana costs." (Dracogenesis): a way to
//! cast the spells a player casts without paying their mana costs — an alternative cost
//! (CR 118.9) for spells matching the description, offered as a way of casting them by
//! `kw/cast_without_paying.rs`.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn cast_without_paying(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l)
        .strip_prefix("you may cast ")?
        .strip_suffix(" without paying their mana costs")?;
    let (phrase, from_hand) = match r.strip_suffix(" from your hand") {
        Some(p) => (p, true),
        None => (r, false),
    };
    let mut parts = Vec::new();
    if phrase != "spells" {
        let (f, plural, tail) = parse_object_phrase(phrase)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        parts.push(f);
    }
    if from_hand {
        parts.push(Filter::InZone(ZoneKind::Hand));
    }
    let filter = if parts.is_empty() {
        Filter::Any
    } else {
        Filter::and(parts)
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Spells(filter),
                who: PlayerRel::You,
                change: CostChange::AlternativeCost(Cost::free()),
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "you may cast [spells] without paying their mana costs", priority: 100, parse: cast_without_paying } }
