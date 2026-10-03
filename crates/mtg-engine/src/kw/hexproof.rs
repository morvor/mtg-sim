//! CR 702.11 Hexproof.
//!
//! Hexproof ("hexproof from [quality]" is the same keyword with a quality filter) is
//! checked when targets are chosen (`Game::object_untargetable`, `player_untargetable`
//! in `stack.rs`). This module holds the effects that let a player target as though
//! permanents or players didn't have hexproof; like the rule says, they also ignore
//! "hexproof from [quality]" (CR 702.11e).

use crate::ability::Filter;
use crate::eval::Ctx;
use crate::game::Game;
use crate::game_terms::{ACTIVATED_ABILITY, TRIGGERED_ABILITY};
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

/// The quality of "hexproof from activated abilities", "hexproof from triggered
/// abilities", "hexproof from activated and triggered abilities": the kind of the
/// targeting ability itself, not of its source (CR 702.11d).
pub fn ability_quality(s: &str) -> Option<Filter> {
    let activated = || Filter::Custom(ACTIVATED_ABILITY.into());
    let triggered = || Filter::Custom(TRIGGERED_ABILITY.into());
    Some(match s.trim() {
        "activated abilities" => activated(),
        "triggered abilities" => triggered(),
        "activated and triggered abilities" | "activated abilities and triggered abilities" => {
            Filter::Or(vec![activated(), triggered()])
        }
        _ => return None,
    })
}

/// Whether a hexproof quality describes the kind of an ability on the stack.
fn describes_ability(f: &Filter) -> bool {
    match f {
        Filter::Custom(n) => n.as_str() == ACTIVATED_ABILITY || n.as_str() == TRIGGERED_ABILITY,
        Filter::Or(v) | Filter::And(v) => v.iter().any(describes_ability),
        _ => false,
    }
}

/// Whether a "hexproof from [quality]" ability of `holder` stops the spell or ability
/// `targeting` (the object on the stack, or the source of an ability not yet there) from
/// targeting it (CR 702.11d): a spell with the quality, an ability from a source with the
/// quality, or an ability that is itself of the kind the quality names.
pub fn quality_stops(g: &Game, quality: &Filter, holder: ObjectId, targeting: ObjectId) -> bool {
    let ctx = Ctx::new(Some(holder), g.obj(holder).controller);
    if describes_ability(quality) {
        return g.obj(targeting).is_stack_ability() && g.matches(targeting, quality, &ctx);
    }
    g.matches(g.ability_source_of(targeting), quality, &ctx)
}
