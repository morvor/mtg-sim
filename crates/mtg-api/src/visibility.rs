//! What a player may see (CR 400.2, 401.2, 402.3, 406.3, 708.5, 723.4).
//!
//! A *viewer* is `Some(player)` for a player's view of the game, or `None` for the
//! omniscient view used for logging and debugging.

use mtg_engine::{Game, ObjectId, PlayerId};

/// A player's point of view, or `None` for an omniscient one.
pub type Viewer = Option<PlayerId>;

/// Whether `viewer` may see the object `id` as it is now (or, for an object that has
/// left its zone, as it last was there): objects in public zones that aren't face down,
/// the viewer's own hand, revealed cards, a library's top card the viewer may look at,
/// face-down permanents and spells the viewer controls (CR 708.5), face-down exiled cards
/// the viewer may look at (CR 406.3), and what a player the viewer controls may see
/// (CR 723).
pub fn can_see(g: &Game, viewer: Viewer, id: ObjectId) -> bool {
    let Some(p) = viewer else {
        return true;
    };
    if (id.0 as usize) >= g.objects.len() {
        return false;
    }
    mtg_engine::player_control::can_see(g, p, id) || mtg_engine::reveal::is_revealed(g, id)
}
