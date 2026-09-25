//! Casual variants (CR 900–905) and supplemental card types: Archenemy schemes,
//! Planechase planes, Vanguard, dungeons, and attractions. Hook points called by the
//! turn structure and SBAs.

use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// CR 613.7i, 613.7j: at the beginning of the game, each face-up vanguard card and each
/// conspiracy card in the command zone receives a timestamp.
pub fn begin_game(g: &mut Game) {
    for id in g.command.clone() {
        let o = g.obj(id);
        let gets =
            (o.base.is(CardType::Vanguard) && !o.face_down) || o.base.is(CardType::Conspiracy);
        if gets {
            let ts = g.new_timestamp();
            g.objects[id.0 as usize].timestamp = ts;
            g.dirty = true;
        }
    }
}

/// Turns a face-down card in the command zone (a plane, phenomenon, scheme or
/// conspiracy card) face up. It receives a new timestamp at that time (CR 613.7h,
/// 613.7j). Returns true if it did.
pub fn turn_face_up_in_command(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.zone != Zone::Command || !o.face_down {
        return false;
    }
    let ts = g.new_timestamp();
    let o = &mut g.objects[id.0 as usize];
    o.face_down = false;
    o.timestamp = ts;
    g.dirty = true;
    true
}

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
