//! CR 702.14 Landwalk.
//!
//! "[Type]walk" compiles to a `Landwalk` keyword whose filter describes the land (the
//! land type, "nonbasic land", "snow Swamp", ...; CR 702.14a, c). The evasion itself is
//! checked in `Game::can_block` (`combat.rs`). This module holds the effects that let
//! creatures with landwalk be blocked as though they didn't have it (Undertow, Staff of
//! the Ages).

use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};

/// Prefix of a `StaticEffect::Custom` name: "Creatures with [type]walk can be blocked as
/// though they didn't have [type]walk." The rest of the name is the landwalk's text in
/// lowercase ("islandwalk"), or empty for "landwalk abilities" (all of them).
pub const BLOCKABLE_AS_THOUGH_NO_LANDWALK: &str = "blockable as though no landwalk:";

/// Whether an attacking creature's landwalk ability is ignored for blocking.
pub fn landwalk_ignored(g: &Game, kw: &Keyword) -> bool {
    if kw.kind != KeywordKind::Landwalk {
        return false;
    }
    let text = kw.text.as_deref().unwrap_or("").to_lowercase();
    g.statics.customs.iter().any(|(_, _, name)| {
        name.strip_prefix(BLOCKABLE_AS_THOUGH_NO_LANDWALK)
            .is_some_and(|which| which.is_empty() || which == text)
    })
}
