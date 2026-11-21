//! "If enchanted land would be destroyed, instead sacrifice ~ and that land gains
//! indestructible until end of turn." (Crackling Emergence, Harmonious Emergence): a
//! replacement effect (CR 614.1a, 614.6) on the destruction of the object the Aura is
//! attached to, for any reason (a destroy instruction, lethal damage, deathtouch). It
//! doesn't apply to sacrificing it, which isn't destroying (CR 701.8b, 701.21a). "That
//! [land]" is the object that would have been destroyed.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn attached_destroy_instead(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let rest = [
        "if enchanted land would be destroyed, instead ",
        "if enchanted creature would be destroyed, instead ",
        "if enchanted permanent would be destroyed, instead ",
        "if equipped creature would be destroyed, instead ",
    ]
    .into_iter()
    .find_map(|p| l.strip_prefix(p))?;
    let body = crate::oracle::effects::parse_body_with_it(rest, ctx, Sel::TriggerObject)?;
    if !body.targets.is_empty() || body.modal.is_some() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Destroy(Filter::AttachedToSource),
                action: ReplacementAction::Instead(Box::new(body.effect)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "if enchanted [permanent] would be destroyed, instead [effect]", priority: 100, parse: attached_destroy_instead } }
