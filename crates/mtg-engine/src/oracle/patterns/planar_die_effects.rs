//! "Roll the planar die" as an effect (Fractured Powerstone). Such a roll isn't the
//! special action of rolling the planar die, so it doesn't count toward that action's
//! cost (CR 116.2i, 901.9).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::planechase::ROLL_PLANAR_DIE_EFFECT;
use smol_str::SmolStr;

fn roll_planar_die(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "roll the planar die" | "you roll the planar die" => {
            Some(Effect::Custom(SmolStr::new(ROLL_PLANAR_DIE_EFFECT)))
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "roll the planar die", priority: 0, parse: roll_planar_die } }
