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

/// Static ability: "Creatures your opponents control can be the targets of spells and
/// abilities as though they didn't have hexproof. Ward abilities of those creatures don't
/// trigger." (Nowhere to Run): spells and abilities of any player.
pub const OPPONENT_CREATURES_TARGETABLE_BY_ANY: &str =
    "opponents' creatures targetable by any player as though no hexproof; their ward doesn't trigger";

/// Player effect (`PlayerModification::Custom`): "Until end of turn, your opponents and
/// creatures your opponents control with hexproof can be the targets of spells and
/// abilities you control as though they didn't have hexproof." (Detection Tower).
pub const OPPONENTS_AND_CREATURES_AS_THOUGH_NO_HEXPROOF: &str =
    "opponents and their creatures targetable as though no hexproof";

/// Whether spells and abilities `by` controls may target `target` as though it didn't
/// have hexproof.
pub fn hexproof_ignored(g: &Game, target: Entity, by: PlayerId) -> bool {
    // (whose effect, which effect)
    let from_statics = g.statics.customs.iter().map(|(_, c, n)| (*c, n.as_str()));
    let from_players = g.players.iter().flat_map(|pl| {
        pl.mods.iter().filter_map(move |m| match m {
            crate::ability::PlayerModification::Custom(n) => Some((pl.id, n.as_str())),
            _ => None,
        })
    });
    from_statics.chain(from_players).any(|(ctl, name)| {
        let (creatures_only, players, anyone) = match name {
            OPPONENT_CREATURES_AS_THOUGH_NO_HEXPROOF => (true, false, false),
            OPPONENTS_AND_PERMANENTS_AS_THOUGH_NO_HEXPROOF => (false, true, false),
            OPPONENTS_AND_CREATURES_AS_THOUGH_NO_HEXPROOF => (true, true, false),
            OPPONENT_CREATURES_TARGETABLE_BY_ANY => (true, false, true),
            _ => return false,
        };
        if !anyone && ctl != by {
            return false;
        }
        match target {
            Entity::Player(p) => players && g.are_opponents(ctl, p),
            Entity::Object(o) => {
                let ob = g.obj(o);
                g.are_opponents(ctl, ob.controller) && (!creatures_only || ob.is_creature())
            }
        }
    })
}

/// Whether the ward abilities of `obj` don't trigger ("Ward abilities of those creatures
/// don't trigger.", Nowhere to Run).
pub fn ward_suppressed(g: &Game, obj: crate::types::ObjectId) -> bool {
    let ob = g.obj(obj);
    ob.is_creature()
        && g.statics.customs.iter().any(|(_, c, n)| {
            n.as_str() == OPPONENT_CREATURES_TARGETABLE_BY_ANY && g.are_opponents(*c, ob.controller)
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
