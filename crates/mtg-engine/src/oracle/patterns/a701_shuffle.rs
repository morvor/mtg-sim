//! Library positions and shuffling (CR 701.24):
//!
//! * "If ~ would die, put it on top of its owner's library instead." (Gravebane Zombie),
//!   "... on the bottom of its owner's library instead", "... shuffle it into its owner's
//!   library instead": a replacement effect (CR 614.1a). An object put into a position of
//!   a library that's shuffled at the same time keeps that position (CR 701.24g).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn die_to_library(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ~ would die, ")?;
    let position = match r {
        "put it on top of its owner's library instead"
        | "put ~ on top of its owner's library instead" => LibraryPosition::Top,
        "put it on the bottom of its owner's library instead"
        | "put ~ on the bottom of its owner's library instead" => LibraryPosition::Bottom,
        "shuffle it into its owner's library instead"
        | "shuffle ~ into its owner's library instead" => LibraryPosition::Shuffled,
        _ => return None,
    };
    let mut to = Destination::zone(ZoneKind::Library);
    to.position = position;
    let def = ReplacementDef {
        event: ReplacementEvent::ZoneChange {
            filter: Filter::Source,
            from: Some(ZoneKind::Battlefield),
            to: Some(ZoneKind::Graveyard),
        },
        action: ReplacementAction::MoveInstead(to),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 would die, put into library instead", priority: 100, parse: die_to_library } }
