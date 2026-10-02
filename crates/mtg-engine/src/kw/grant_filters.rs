//! Object filters of group grants ("Creatures attacking you get -1/-0", "Creatures
//! attacking enchanted player have trample", "Creatures attacking your opponents have
//! double strike"): a creature attacking a player, not a planeswalker or battle that
//! player controls or protects (CR 506.2, 508.1b). Evaluated for the static ability's
//! source and controller.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// Attacking the controller of the ability ("creatures attacking you").
pub const ATTACKING_YOU: &str = "attacking_player:you";
/// Attacking an opponent of the controller ("creatures attacking your opponents").
pub const ATTACKING_OPPONENT: &str = "attacking_player:opponent";
/// Attacking the player the source is attached to ("creatures attacking enchanted
/// player").
pub const ATTACKING_ENCHANTED_PLAYER: &str = "attacking_player:enchanted";

/// "that dealt damage this turn": the object was the source of damage this turn (CR 120).
pub const DEALT_DAMAGE_THIS_TURN: &str = "dealt_damage_this_turn";

pub struct GrantFilters;

impl KeywordRules for GrantFilters {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == DEALT_DAMAGE_THIS_TURN {
            return Some(g.history.damage_sources.iter().any(|(s, _)| *s == id));
        }
        if !matches!(
            name,
            ATTACKING_YOU | ATTACKING_OPPONENT | ATTACKING_ENCHANTED_PLAYER
        ) {
            return None;
        }
        let Some(Entity::Player(p)) = g.combat.as_ref().and_then(|c| c.attack_target(id)) else {
            return Some(false);
        };
        Some(match name {
            ATTACKING_YOU => p == ctx.controller,
            ATTACKING_OPPONENT => g.are_opponents(ctx.controller, p),
            _ => ctx
                .source
                .is_some_and(|s| g.obj(s).attached_to == Some(Entity::Player(p))),
        })
    }
}

inventory::submit! { KeywordRegistration(&GrantFilters) }
