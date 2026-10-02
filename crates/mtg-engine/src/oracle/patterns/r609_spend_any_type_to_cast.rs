//! "You can spend mana of any type to cast creature spells." (Vizier of the Menagerie):
//! a static ability changing how the player may pay for those spells (CR 609.4b,
//! 118.14) — each mana symbol of their costs can be paid with mana of any type; the costs
//! themselves don't change. See `cost_rules::may_spend_any_type`.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn spend_any_type_to_cast(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let kind = l
        .strip_prefix("you can spend mana of any type to cast ")
        .or_else(|| l.strip_prefix("you may spend mana as though it were mana of any type to cast "))?;
    let what = if kind == "spells" {
        Filter::Any
    } else {
        let (f, _, tail) = parse_object_phrase(kind)?;
        if !end(tail).is_empty() || !kind.ends_with(" spells") {
            return None;
        }
        f
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Spells(Filter::and(vec![what, Filter::Spell])),
                who: PlayerRel::You,
                change: CostChange::SpendAnyType,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r609 you can spend mana of any type to cast [spells]", priority: 60, parse: spend_any_type_to_cast } }
