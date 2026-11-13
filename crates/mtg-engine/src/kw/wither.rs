//! CR 702.80 Wither: damage dealt to a creature by a source with wither isn't marked on
//! it; it causes that source's controller to put that many -1/-1 counters on it
//! (CR 702.80a, 120.3d). The damage rules check whether the source had wither using its
//! last known information if it's no longer in the zone it's expected in (CR 702.80b), in
//! whatever zone it deals damage from (CR 702.80c); several instances are redundant
//! (CR 702.80d).
//!
//! "All damage is dealt as though its source had wither." (Everlasting Torment) is a
//! static ability ([`ALL_DAMAGE_AS_THOUGH_WITHER`]) that the damage rules also check.

use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::GameObject;

/// `StaticEffect::Custom`: "All damage is dealt as though its source had wither."
pub const ALL_DAMAGE_AS_THOUGH_WITHER: &str = "wither:all damage as though its source had wither";

/// Whether damage `src` deals is dealt as though by a source with wither.
pub fn deals_damage_with_wither(g: &Game, src: &GameObject) -> bool {
    src.has_keyword(KeywordKind::Wither)
        || g
            .statics
            .customs
            .iter()
            .any(|(_, _, n)| n == ALL_DAMAGE_AS_THOUGH_WITHER)
}
