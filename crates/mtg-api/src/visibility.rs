//! What a player may see (CR 400.2, 401.2, 402.3, 406.3, 708.5, 723.4).
//!
//! A *viewer* is `Some(player)` for a player's view of the game, or `None` for the
//! omniscient view used for logging and debugging.

use mtg_engine::object::Zone;
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

/// Whether the object's identity is public: anyone may see it.
pub fn is_public(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    !o.face_down && o.zone.is_public()
}

/// Whether `viewer` may see the cards in `owner`'s hand (all of them).
pub fn sees_hand(g: &Game, viewer: Viewer, owner: PlayerId) -> bool {
    match viewer {
        None => true,
        Some(p) => {
            p == owner
                || mtg_engine::reveal::hand_revealed(g, owner)
                || g.player(owner).hand.iter().all(|c| can_see(g, viewer, *c))
        }
    }
}

/// Whether `id` is in a hidden zone (library, hand, outside the game) or face down.
pub fn is_hidden_kind(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.face_down || matches!(o.zone, Zone::Library(_) | Zone::Hand(_) | Zone::Outside(_))
}
