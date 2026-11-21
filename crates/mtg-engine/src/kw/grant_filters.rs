//! Object filters of group grants. "Creatures attacking you" and "creatures attacking
//! your opponents" are `Filter::AttackingPlayer` (the player only, CR 506.2, 508.1b);
//! here: "creatures attacking enchanted player" (the player the source is attached to),
//! "attacking your opponents and/or planeswalkers they control" (Roar of Resistance),
//! "attacking the same player or planeswalker" and a few non-combat qualifiers.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// Attacking the player the source is attached to ("creatures attacking enchanted
/// player").
pub const ATTACKING_ENCHANTED_PLAYER: &str = "attacking_player:enchanted";
/// Attacking an opponent of the controller or a planeswalker an opponent controls
/// ("creatures attacking your opponents and/or planeswalkers they control").
pub const ATTACKING_OPPONENT_OR_THEIR_PLANESWALKER: &str = "attacking_player_or_pw:opponent";
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
        let target = g.combat.as_ref().and_then(|c| c.attack_target(id));
        if name == ATTACKING_OPPONENT_OR_THEIR_PLANESWALKER {
            return Some(match target {
                Some(Entity::Player(p)) => g.are_opponents(ctx.controller, p),
                Some(Entity::Object(o)) => {
                    g.obj(o).chars.card_types.contains(CardType::Planeswalker)
                        && g.are_opponents(ctx.controller, g.obj(o).controller)
                }
                None => false,
            });
        }
        if name != ATTACKING_ENCHANTED_PLAYER {
            return None;
        }
        let Some(Entity::Player(p)) = target else {
            return Some(false);
        };
        Some(
            ctx.source
                .is_some_and(|s| g.obj(s).attached_to == Some(Entity::Player(p))),
        )
    }
}

inventory::submit! { KeywordRegistration(&GrantFilters) }
