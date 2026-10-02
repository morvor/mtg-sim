//! "[player] gains control of [object] [until end of turn]": "Target opponent gains
//! control of ~." (Jinxed Idol), "Choose one of your opponents. That player gains control
//! of ~." (Goblin Festival). A control-changing effect (CR 613.1b) of the resolving
//! spell or ability; with no stated duration it lasts until the game ends (CR 611.2a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, player_ref, Builder};
use crate::oracle::phrases::end;

fn player_gains_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = duration_suffix(end(l));
    let (subj, obj) = l.split_once(" gains control of ")?;
    if subj == "you" || subj.contains(' ') && subj.starts_with("~") {
        return None;
    }
    let (who, r) = player_ref(subj, b)?;
    if !r.trim().is_empty() || matches!(who, PlayerRef::You) {
        return None;
    }
    let (what, tail) = object_ref(obj, b)?;
    if !end(&tail).is_empty() || matches!(what, Sel::None) {
        return None;
    }
    Some(Effect::GainControl {
        what,
        who,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "[player] gains control of [object]", priority: 100, parse: player_gains_control } }
