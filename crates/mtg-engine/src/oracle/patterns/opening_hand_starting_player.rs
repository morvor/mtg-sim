//! "If ~ is in your opening hand and you're not the starting player, you may reveal it. If
//! you do, you become the starting player." (Impatient Iguana): an opening-hand action
//! (CR 103.6), see [`crate::opening_hand::BECOME_STARTING_PLAYER`].

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

fn become_starting_player(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if t.to_lowercase()
        != "if ~ is in your opening hand and you're not the starting player, you may reveal it. if you do, you become the starting player."
    {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(SmolStr::new(
        crate::opening_hand::BECOME_STARTING_PLAYER,
    )));
    s.zone = FunctionZone::Hand;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
}

inventory::submit! { AbilityPattern { name: "opening hand: become the starting player", priority: 0, parse: become_starting_player } }
