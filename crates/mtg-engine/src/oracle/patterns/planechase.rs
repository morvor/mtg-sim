//! Planechase trigger conditions (CR 311.7, 901.9): "Whenever chaos ensues" (chaos
//! abilities; older wording "Whenever you roll {CHAOS}", CR 107.12) and "Whenever you roll
//! the planar die".

use crate::ability::*;
use crate::oracle::patterns::TriggerPattern;
use crate::oracle::phrases::end;
use crate::planechase::{CHAOS_ENSUES, ROLLED_PLANAR_DIE};
use smol_str::SmolStr;

fn planar_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (name, who) = match end(r) {
        "chaos ensues" => (CHAOS_ENSUES, PlayerRel::Any),
        "you roll {chaos}" => (CHAOS_ENSUES, PlayerRel::You),
        "you roll the planar die" => (ROLLED_PLANAR_DIE, PlayerRel::You),
        "a player rolls the planar die" => (ROLLED_PLANAR_DIE, PlayerRel::Any),
        _ => return None,
    };
    Some((
        TriggerCond::PlayerAction {
            name: SmolStr::new(name),
            who,
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "planechase chaos and planar die triggers", priority: 100, parse: planar_trigger } }
