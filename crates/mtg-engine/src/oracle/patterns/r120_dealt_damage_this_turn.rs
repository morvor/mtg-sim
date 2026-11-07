//! Oracle conditions about the damage the ability's source dealt this turn (CR 120):
//! "~ dealt damage to an opponent this turn" (Dunerider Outlaw: "At the beginning of each
//! end step, if this creature dealt damage to an opponent this turn, put a +1/+1 counter on
//! it."), "~ dealt damage to a player this turn". See `kw/dealt_damage_this_turn.rs`.

use super::ConditionPattern;
use crate::ability::*;
use crate::kw::dealt_damage_this_turn::{DEALT_DAMAGE_TO_OPPONENT, DEALT_DAMAGE_TO_PLAYER};
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn source_dealt_damage_this_turn(c: &str) -> Option<Condition> {
    let name = match end(c) {
        "~ dealt damage to an opponent this turn" => DEALT_DAMAGE_TO_OPPONENT,
        "~ dealt damage to a player this turn" => DEALT_DAMAGE_TO_PLAYER,
        _ => return None,
    };
    Some(Condition::Custom(SmolStr::new(name)))
}

inventory::submit! { ConditionPattern { name: "r120 ~ dealt damage to a player this turn", priority: 100, parse: source_dealt_damage_this_turn } }
