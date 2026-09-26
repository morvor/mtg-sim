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

pub struct DungeonCompletion;

impl KeywordRules for DungeonCompletion {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == DUNGEONS_COMPLETED).then(|| g.player(ctx.controller).dungeons_completed as i64)
    }
}

inventory::submit! { KeywordRegistration(&DungeonCompletion) }
