//! "each player being attacked" (CR 506.2, 508.1b): a global hook giving the number of
//! players that at least one attacking creature is attacking in the current combat (a
//! creature attacking a planeswalker or battle doesn't attack its controller or
//! protector), for "draw a card for each player being attacked"
//! (`Value::Custom(PLAYERS_BEING_ATTACKED)`).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use std::collections::BTreeSet;

/// `Value::Custom` name: the number of players being attacked.
pub const PLAYERS_BEING_ATTACKED: &str = "players being attacked";

/// The players being attacked in the current combat.
pub fn players_being_attacked(g: &Game) -> BTreeSet<PlayerId> {
    g.combat
        .as_ref()
        .map(|c| {
            c.attackers
                .iter()
                .filter_map(|a| match a.target {
                    Some(Entity::Player(p)) => Some(p),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

pub struct PlayersBeingAttacked;

impl KeywordRules for PlayersBeingAttacked {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<i64> {
        (name == PLAYERS_BEING_ATTACKED).then(|| players_being_attacked(g).len() as i64)
    }
}

inventory::submit! { KeywordRegistration(&PlayersBeingAttacked) }
