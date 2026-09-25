//! Casual variants (CR 900–905) and supplemental card types: Archenemy schemes,
//! Planechase planes, Vanguard, dungeons, and attractions. Hook points called by the
//! turn structure and SBAs.

use crate::game::Game;
use crate::types::*;

/// CR 505.3 / 904: the archenemy sets a scheme in motion at the start of their
/// precombat main phase.
pub fn archenemy_main_phase(g: &mut Game, p: PlayerId) {
    let _ = (g, p);
}

/// CR 505.5 / 701.52: roll to visit attractions.
pub fn roll_to_visit_attractions(g: &mut Game, p: PlayerId) {
    let _ = (g, p);
}

/// CR 704.5t: dungeon completion. Returns true if an action was performed.
pub fn dungeon_sba(g: &mut Game) -> bool {
    let _ = g;
    false
}

/// CR 704.6e/f: archenemy scheme and planechase phenomenon SBAs.
pub fn variant_sbas(g: &mut Game) -> bool {
    let _ = g;
    false
}
