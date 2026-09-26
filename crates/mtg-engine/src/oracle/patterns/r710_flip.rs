//! Flip cards (CR 710): "flip this creature" / "flip ~" / "flip it" (the permanent whose
//! ability it is).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn flip_self(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("flip ")?;
    matches!(r, "~" | "it" | "this creature" | "this permanent")
        .then(|| Effect::Custom(crate::flip::FLIP_SELF.into()))
}

inventory::submit! { EffectPattern { name: "r710 flip this permanent", priority: 60, parse: flip_self } }
