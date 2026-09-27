//! State kept by the rules of keyword abilities (CR 702) that isn't a characteristic or
//! status of any object: designations of players and what happened earlier in the game.
//! Kept in [`crate::game::Game::kw_state`].

use crate::types::*;
use smol_str::SmolStr;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub struct KeywordState {
    /// Players who have an enduring story (CR 702.195b): once a player gets it, they keep
    /// it for the rest of the game.
    pub enduring_story: BTreeSet<PlayerId>,
    /// The names of the spells each player controlled that have resolved this game, for
    /// "if this is the first time a spell you control with this spell's name has resolved
    /// this game" (paradigm, CR 702.192a).
    pub resolved_spell_names: BTreeMap<PlayerId, BTreeSet<SmolStr>>,
}
