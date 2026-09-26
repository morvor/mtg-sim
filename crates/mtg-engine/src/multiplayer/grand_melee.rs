//! The Grand Melee variant's turn markers and stacks (CR 807.4, 807.5).

use crate::game::Game;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Grand Melee bookkeeping.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GrandMelee {}

/// A player left the game (CR 807.4c, 807.4e).
pub fn player_left(_g: &mut Game, _p: PlayerId) {}
