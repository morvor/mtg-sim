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
/// `Filter::Custom`: attacking the player the trigger is about ("that's attacking that
/// player").
pub const ATTACKING_TRIGGER_PLAYER: &str = "attacking_player:trigger";
/// `Filter::Custom`: attacking the same player, planeswalker or battle as the source
/// ("another target creature attacking the same player or planeswalker", Kitesail
/// Skirmisher).
pub const ATTACKING_SAME_AS_SOURCE: &str = "attacking_same_as_source";

/// "that dealt damage this turn": the object was the source of damage this turn (CR 120).
pub const DEALT_DAMAGE_THIS_TURN: &str = "dealt_damage_this_turn";

/// `Modification::Custom` (layer 4): "loses all land types" (CR 205.3i, 305.7): the
/// object's land subtypes are removed.
pub const LOSE_ALL_LAND_TYPES: &str = "lose_all_land_types";

/// `Modification::Custom` (layer 6) named this prefix followed by an ability's text: the
/// object loses the abilities with that text ("~ loses \"Prevent all damage that would
/// be dealt to ~.\"", Glittering Lion; CR 613.1f).
pub const LOSE_ABILITY_PREFIX: &str = "lose_ability:";

pub struct GrantFilters;

impl KeywordRules for GrantFilters {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_modification(
        &self,
        _g: &Game,
        name: &str,
        chars: &mut crate::object::Characteristics,
        _ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        if let Some(text) = name.strip_prefix(LOSE_ABILITY_PREFIX) {
            let text = text.trim().to_lowercase();
            chars
                .abilities
                .retain(|a| a.text.trim().to_lowercase() != text);
            return true;
        }
        if name != LOSE_ALL_LAND_TYPES {
            return false;
        }
        let land = &crate::types::subtype_lists().land;
        chars.subtypes.retain(|s| !land.contains(s.as_str()));
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        // Bound to the granting object when the ability is acquired (`granted_by`); an
        // unbound one matches nothing.
        if name == crate::granted_by::GRANTER {
            return Some(false);
        }
        if name == DEALT_DAMAGE_THIS_TURN {
            return Some(g.history.damage_sources.iter().any(|(s, _)| *s == id));
        }
        if name == ATTACKING_SAME_AS_SOURCE {
            let target_of = |o| g.combat.as_ref().and_then(|c| c.attack_target(o));
            return Some(ctx.source.is_some_and(|s| {
                let t = target_of(s);
                t.is_some() && t == target_of(id)
            }));
        }
        if !matches!(
            name,
            ATTACKING_YOU | ATTACKING_OPPONENT | ATTACKING_ENCHANTED_PLAYER | ATTACKING_TRIGGER_PLAYER
        ) {
            return None;
        }
        let Some(Entity::Player(p)) = g.combat.as_ref().and_then(|c| c.attack_target(id)) else {
            return Some(false);
        };
        Some(match name {
            ATTACKING_YOU => p == ctx.controller,
            ATTACKING_OPPONENT => g.are_opponents(ctx.controller, p),
            ATTACKING_TRIGGER_PLAYER => ctx.event.as_ref().and_then(|e| e.player) == Some(p),
            _ => ctx
                .source
                .is_some_and(|s| g.obj(s).attached_to == Some(Entity::Player(p))),
        })
    }
}

inventory::submit! { KeywordRegistration(&GrantFilters) }
