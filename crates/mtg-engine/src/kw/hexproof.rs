//! CR 702.11 Hexproof.
//!
//! Hexproof ("hexproof from [quality]" is the same keyword with a quality filter) is
//! checked when targets are chosen (`Game::object_untargetable`, `player_untargetable`
//! in `stack.rs`). This module holds the effects that let a player target as though
//! permanents or players didn't have hexproof; like the rule says, they also ignore
//! "hexproof from [quality]" (CR 702.11e).

use crate::game::Game;
use crate::types::*;

/// Static ability (a `StaticEffect::Custom`): "Creatures your opponents control with
/// hexproof can be the targets of spells and abilities you control as though they didn't
/// have hexproof."
pub const OPPONENT_CREATURES_AS_THOUGH_NO_HEXPROOF: &str =
    "opponents' creatures targetable as though no hexproof";

/// Static ability: "Your opponents and permanents your opponents control with hexproof
/// can be the targets of spells and abilities you control as though they didn't have
/// hexproof."
pub const OPPONENTS_AND_PERMANENTS_AS_THOUGH_NO_HEXPROOF: &str =
    "opponents and their permanents targetable as though no hexproof";

/// Whether spells and abilities `by` controls may target `target` as though it didn't
/// have hexproof.
pub fn hexproof_ignored(g: &Game, target: Entity, by: PlayerId) -> bool {
    g.statics.customs.iter().any(|(_, ctl, name)| {
        if *ctl != by {
            return false;
        }
        let creatures_only = match name.as_str() {
            OPPONENT_CREATURES_AS_THOUGH_NO_HEXPROOF => true,
            OPPONENTS_AND_PERMANENTS_AS_THOUGH_NO_HEXPROOF => false,
            _ => return false,
        };
        match target {
            Entity::Player(p) => !creatures_only && g.are_opponents(by, p),
            Entity::Object(o) => {
                let ob = g.obj(o);
                g.are_opponents(by, ob.controller) && (!creatures_only || ob.is_creature())
            }
        }
    })
}
