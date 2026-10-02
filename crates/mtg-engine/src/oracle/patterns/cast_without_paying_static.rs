//! "You may cast Dragon spells without paying their mana costs." (Dracogenesis): a way to
//! cast the spells a player casts without paying their mana costs — an alternative cost
//! (CR 118.9) for spells matching the description, from wherever the player may cast
//! them, offered as a way of casting them by `kw/offered_costs.rs`. The form "...
//! from your hand ..." (Omniscience) is `r601_cast_free_from_hand.rs`'s.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn cast_without_paying(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l)
        .strip_prefix("you may cast ")?
        .strip_suffix(" without paying their mana costs")?;
    if r.ends_with(" from your hand") {
        return None;
    }
    let filter = if r == "spells" {
        Filter::Any
    } else {
        let (f, plural, tail) = parse_object_phrase(r)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        f
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
