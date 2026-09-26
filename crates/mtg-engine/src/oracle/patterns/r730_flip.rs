//! Oracle patterns for flip cards (CR 710) and flipped merged permanents (CR 730.2h):
//! "flip ~", "flip it".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn flip_this(l: &str, b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "flip ~" => {}
        // "It" is the permanent itself, in its own triggered ability.
        "flip it" if matches!(b.it, Sel::This) || b.in_trigger => {}
        _ => return None,
    }
    Some(Effect::Custom(SmolStr::new(crate::merge::FLIP)))
}

inventory::submit! { EffectPattern { name: "r730 flip this permanent", priority: 60, parse: flip_this } }
