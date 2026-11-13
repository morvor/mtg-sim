//! Oracle patterns for planes (CR 311): "chaos ensues" as an effect (CR 311.7), as in
//! "When you planeswalk to Oteclán and at the beginning of your upkeep, chaos ensues."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn chaos_ensues(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "chaos ensues").then(|| {
        Effect::Custom(SmolStr::new(crate::planechase::CHAOS_ENSUES_EFFECT))
    })
}

inventory::submit! { EffectPattern { name: "r311 chaos ensues", priority: 100, parse: chaos_ensues } }
