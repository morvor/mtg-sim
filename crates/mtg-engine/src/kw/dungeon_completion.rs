//! Completed dungeons (CR 309.7): a global hook giving the number of dungeons a player has
//! completed this game, for "if you've completed a dungeon" / "as long as you've completed
//! a dungeon" (`Value::Custom(DUNGEONS_COMPLETED)`). "Whenever you complete a dungeon"
//! triggers on the `dungeons::COMPLETED` player action.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Value::Custom` name: how many dungeons the player has completed this game.
pub const DUNGEONS_COMPLETED: &str = "dungeons completed";
/// `Value::Custom` name prefix: "dungeon completed:NAME" is how many times the player has
/// completed the dungeon named NAME this game ("if you haven't completed Tomb of
/// Annihilation").
pub const DUNGEON_NAMED_COMPLETED: &str = "dungeon completed:";

pub struct DungeonCompletion;

impl KeywordRules for DungeonCompletion {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let player = g.player(ctx.controller);
        if name == DUNGEONS_COMPLETED {
            return Some(player.dungeons_completed as i64);
        }
        let dungeon = name.strip_prefix(DUNGEON_NAMED_COMPLETED)?;
        Some(
            player
                .completed_dungeons
                .iter()
                .filter(|n| n.eq_ignore_ascii_case(dungeon))
                .count() as i64,
        )
    }
}

inventory::submit! { KeywordRegistration(&DungeonCompletion) }
