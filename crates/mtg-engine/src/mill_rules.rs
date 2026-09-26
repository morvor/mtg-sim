//! CR 701.17 Mill.
//!
//! * A player mills as many cards as they can (CR 701.17b); a player can't choose to mill
//!   more cards than their library has, or pay such a cost (see `draw_rules::can_choose`
//!   and the mill cost part).
//! * The milled cards are found wherever they went from the library, if that's a public
//!   zone (CR 701.17c): "the milled card" can be a card exiled instead.
//! * Replacement effects can change how many cards are milled ("If an opponent would mill
//!   one or more cards, they mill twice that many cards instead."); information about "the
//!   milled card" then comes from each of them, summed for a value (CR 701.17d).

use crate::game::Game;
use crate::types::*;

/// `StaticEffect::Custom` prefix: "mill multiplier:[k]:[who]", with who "opponents",
/// "you", or "each": "If an opponent would mill one or more cards, they mill twice that
/// many cards instead."
pub const MILL_MULTIPLIER: &str = "mill multiplier:";

/// How many cards `p` mills instead of `n`, after the replacement effects that multiply
/// it (each applies once, CR 616.1).
pub fn replaced_count(g: &Game, p: PlayerId, n: u32) -> u32 {
    let mut n = n;
    if n == 0 {
        return 0;
    }
    for (_, ctl, name) in &g.statics.customs {
        let Some(spec) = name.strip_prefix(MILL_MULTIPLIER) else {
            continue;
        };
        let Some((k, who)) = spec.split_once(':') else {
            continue;
        };
        let Ok(k) = k.parse::<u32>() else {
            continue;
        };
        let applies = match who {
            "opponents" => g.are_opponents(*ctl, p),
            "you" => *ctl == p,
            "each" => true,
            _ => false,
        };
        if applies {
            n = n.saturating_mul(k);
        }
    }
    n
}
