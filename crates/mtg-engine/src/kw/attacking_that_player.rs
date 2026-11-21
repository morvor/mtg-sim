//! "each opponent attacking that player" (the Curses of Commander 2017: "Whenever enchanted
//! player is attacked, you gain 1 life. Each opponent attacking that player does the
//! same."). A player is attacking another player if they control a creature that's
//! attacking that player (CR 506.2, 802.1); attacking a planeswalker that player controls
//! doesn't count. Whether a player is attacking is checked as the ability resolves.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: a creature attacking the player of the event the ability responds to
/// (the attacked player of "whenever [a player] is attacked").
pub const ATTACKING_THAT_PLAYER: &str = "attacking the event's player";

/// The players attacking the event's player who are opponents of the ability's
/// controller ("each opponent attacking that player").
pub fn opponents_attacking_that_player() -> PlayerRef {
    PlayerRef::Each(PlayerFilter::And(vec![
        PlayerFilter::Opponent,
        PlayerFilter::Controls(
            Box::new(Filter::Custom(ATTACKING_THAT_PLAYER.into())),
            Cmp::Ge,
            Box::new(Value::c(1)),
        ),
    ]))
}

pub struct AttackingThatPlayer;

impl KeywordRules for AttackingThatPlayer {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != ATTACKING_THAT_PLAYER {
            return None;
        }
        let Some(p) = ctx.event.as_ref().and_then(|e| e.player) else {
            return Some(false);
        };
        Some(
            g.combat
                .as_ref()
                .and_then(|c| c.attack_target(id))
                .is_some_and(|t| t == Entity::Player(p)),
        )
    }
}

inventory::submit! { KeywordRegistration(&AttackingThatPlayer) }
