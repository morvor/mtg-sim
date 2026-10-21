//! Vesuvan Doppelganger: "You may have ~ enter as a copy of any creature on the
//! battlefield, except it doesn't copy that creature's color and it has "At the beginning
//! of your upkeep, you may have ~ become a copy of target creature, except it doesn't copy
//! that creature's color and it has this ability.""
//!
//! Both copy effects keep the Doppelganger's own color, blue: an exception that becomes
//! part of the copiable values (CR 707.9b). The rest is the compiled clone and "except it
//! has this ability" grammar (CR 707.9a).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::types::{Color, ColorSet};

const TEXT: &str = "You may have ~ enter as a copy of any creature on the battlefield, except it doesn't copy that creature's color and it has \"At the beginning of your upkeep, you may have ~ become a copy of target creature, except it doesn't copy that creature's color and it has this ability.\"";
const UPKEEP: &str = "At the beginning of your upkeep, you may have ~ become a copy of target creature, except it doesn't copy that creature's color and it has this ability.";

/// "It doesn't copy that creature's color": its color stays the Doppelganger's, blue.
fn own_color() -> Modification {
    Modification::SetColors(ColorSet::single(Color::Blue))
}

/// Adds the color exception to the copy effects in `e` (and in the abilities it grants).
fn keep_color(e: &mut Effect) {
    match e {
        Effect::Seq(v) => v.iter_mut().for_each(keep_color),
        Effect::May { effect, .. } => keep_color(effect),
        Effect::BecomeCopyExcept { exceptions, .. } => exceptions.push(own_color()),
        Effect::EnterCopyExceptions(mods) => {
            for m in mods.iter_mut() {
                if let Modification::AddAbility(a) = m {
                    *a = keep_color_ability(a, UPKEEP);
                }
            }
            mods.push(own_color());
        }
        _ => {}
    }
}

fn keep_color_ability(a: &Ability, text: &str) -> Ability {
    let mut kind = a.kind.clone();
    match &mut kind {
        AbilityKind::Triggered(t) => keep_color(&mut t.body.effect),
        AbilityKind::Static(s) => {
            if let StaticEffect::Replacement(ReplacementDef {
                action: ReplacementAction::AsEnters(e),
                ..
            }) = &mut s.effect
            {
                keep_color(e);
            }
        }
        _ => {}
    }
    AbilityDef::new(kind, text)
}

inventory::submit! { ManualAbility {
    card: "Vesuvan Doppelganger",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "You may have ~ enter as a copy of any creature on the battlefield, except it has \"At the beginning of your upkeep, you may have ~ become a copy of target creature, except it has this ability.\"")
            .iter()
            .map(|a| keep_color_ability(a, TEXT))
            .collect()
    },
    reason: "copies that don't copy the creature's color (keeping the Doppelganger's own): unique",
} }
