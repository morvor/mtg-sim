//! Discarding (CR 701.9): "If an effect causes you to discard a card, discard it, but you
//! may put it on top of your library instead of into your graveyard." (Library of Leng).

use super::StaticPattern;
use crate::ability::*;
use crate::discard_rules::DISCARDED_BY_EFFECT;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn discard_to_library(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix(
        "if an effect causes you to discard a card, discard it, but you may put it ",
    )?;
    let to = match r {
        "on top of your library instead of into your graveyard" => Destination::library_top(),
        "on the bottom of your library instead of into your graveyard" => {
            Destination::library_bottom()
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Discard(
                    PlayerFilter::You,
                    Filter::Custom(DISCARDED_BY_EFFECT.into()),
                ),
                action: ReplacementAction::MoveInstead(to),
                self_replacement: false,
                optional: true,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 discard to library instead", priority: 100, parse: discard_to_library } }
