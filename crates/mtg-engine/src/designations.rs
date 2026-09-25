//! Player designations: the monarch (CR 725) and the initiative (CR 726).

use crate::events::Event;
use crate::game::Game;
use crate::types::*;

pub fn become_monarch(g: &mut Game, p: PlayerId) {
    if g.monarch != Some(p) {
        g.monarch = Some(p);
        g.emit(Event::BecameMonarch { player: p });
    }
}

pub fn take_initiative(g: &mut Game, p: PlayerId) {
    g.initiative = Some(p);
    g.players[p.idx()].initiative_count += 1;
    g.emit(Event::TookInitiative { player: p });
}
