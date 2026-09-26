//! Oracle patterns for ending turns and phases (CR 724): "End the turn." (Time Stop,
//! Sundial of the Infinite), "The player whose turn it is may end the turn." (Obeka,
//! Brute Chronologist) and "End the combat phase." (Mandate of Peace).

use super::EffectPattern;
use crate::ability::*;
use crate::end_turn::{end_the_combat_phase_effect, end_the_turn_effect};
use crate::oracle::effects::Builder;

fn end_turn_or_phase(l: &str, _b: &mut Builder) -> Option<Effect> {
    match l.trim().trim_end_matches('.') {
        "end the turn" => Some(end_the_turn_effect()),
        "end the combat phase" => Some(end_the_combat_phase_effect()),
        "the player whose turn it is may end the turn" => Some(Effect::May {
            who: PlayerRef::ActivePlayer,
            effect: Box::new(end_the_turn_effect()),
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r724 end the turn or combat phase", priority: 60, parse: end_turn_or_phase } }
