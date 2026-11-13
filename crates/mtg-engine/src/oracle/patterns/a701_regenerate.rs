//! Regeneration from a static ability (CR 701.19b): "If ~ would be destroyed, regenerate
//! it." replaces each destruction of it, not just the next one.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn static_regeneration(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "if ~ would be destroyed, regenerate it" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Destroy(Filter::Source),
                action: ReplacementAction::Regenerate,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 static regeneration", priority: 100, parse: static_regeneration } }
