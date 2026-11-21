//! Oracle patterns for the monarch (CR 725): "The monarch controls enchanted creature."
//! A continuous effect determined by who is the monarch does nothing while there is no
//! monarch (CR 725.5): `PlayerRef::Monarch` then names no player.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

/// "The monarch controls enchanted creature" (layer 2, CR 613.1b).
fn monarch_controls(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let what = l.trim().strip_prefix("the monarch controls ")?;
    if !matches!(
        what,
        "enchanted creature" | "enchanted permanent" | "enchanted artifact"
    ) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::AttachedToSource,
            mods: vec![Modification::SetController(PlayerRef::Monarch)],
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r725 the monarch controls enchanted creature", priority: 60, parse: monarch_controls } }
