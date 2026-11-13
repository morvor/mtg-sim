//! "You may cast Dragon spells without paying their mana costs." (Dracogenesis): a way to
//! cast the spells a player casts without paying their mana costs — an alternative cost
//! (CR 118.9) for spells matching the description, from wherever the player may cast
//! them, offered as a way of casting them by `kw/offered_costs.rs`. The form "...
//! from your hand ..." (Omniscience) is `r601_cast_free_from_hand.rs`'s.
//!
//! "Any player may cast creature spells with mana value 3 or less without paying their
//! mana costs and as though they had flash." (Aluren): the same for every player's spells,
//! with flash only when cast that way (`CostChange::AlternativeCostWithFlash`, CR 601.3c;
//! Aluren ruling: "You either cast the creature normally, or via Aluren without paying the
//! mana cost").

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn cast_without_paying(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (who, r) = match l.strip_prefix("you may cast ") {
        Some(r) => (PlayerRel::You, r),
        None => (PlayerRel::Any, l.strip_prefix("any player may cast ")?),
    };
    let (r, flash) = match r.strip_suffix(" without paying their mana costs") {
        Some(r) => (r, false),
        None => (
            r.strip_suffix(" without paying their mana costs and as though they had flash")?,
            true,
        ),
    };
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
                who,
                change: if flash {
                    CostChange::AlternativeCostWithFlash(Cost::free())
                } else {
                    CostChange::AlternativeCost(Cost::free())
                },
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "you may / any player may cast [spells] without paying their mana costs [and as though they had flash]", priority: 100, parse: cast_without_paying } }
