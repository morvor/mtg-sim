//! Designations: the monarch (CR 725), the initiative (CR 726), and prepared
//! permanents (CR 722.3).

use crate::events::Event;
use crate::game::Game;
use crate::object::{ObjKind, Zone};
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

// ---------------------------------------------------------------------------
// Prepared (CR 722.3)
// ---------------------------------------------------------------------------

/// Whether a permanent has a prepare spell (CR 722.2a).
pub fn has_prepare_spell(g: &Game, o: ObjectId) -> bool {
    g.obj(o)
        .card
        .as_ref()
        .is_some_and(|c| c.layout == crate::card::Layout::Prepare && c.faces.len() > 1)
}

/// The permanent becomes prepared (CR 722.3a): if it has a prepare spell and isn't
/// already prepared, it gains the designation and its controller creates a copy of it in
/// exile with only the prepare spell's characteristics (CR 722.3c).
pub fn become_prepared(g: &mut Game, o: ObjectId) {
    if !g.is_live(o)
        || g.obj(o).zone != Zone::Battlefield
        || g.obj(o).prepared.is_some()
        || !has_prepare_spell(g, o)
    {
        return;
    }
    let card = g.obj(o).card.clone().unwrap();
    let controller = g.obj(o).controller;
    let chars = card.faces[1].chars.clone();
    let id = g.create_card_object(card, controller, Zone::Exile);
    {
        let c = &mut g.objects[id.0 as usize];
        c.kind = ObjKind::CardCopy;
        // The copy has only the prepare spell's characteristics; it isn't tied to the
        // card's faces, so they're kept through its zone changes.
        c.card = None;
        c.base = chars.clone();
        c.copiable = chars.clone();
        c.chars = chars;
        c.controller = controller;
        c.base_controller = controller;
    }
    g.exile.push(id);
    g.objects[o.0 as usize].prepared = Some(id);
    g.dirty = true;
}

/// The permanent becomes unprepared (CR 722.3b). Its prepare-spell copy then ceases to
/// exist (CR 722.3c, 704.5e).
pub fn become_unprepared(g: &mut Game, o: ObjectId) {
    if g.objects[o.0 as usize].prepared.take().is_some() {
        g.dirty = true;
    }
}

/// Whether an exiled card copy is the prepare-spell copy of a prepared permanent (the
/// exception to CR 704.5e in CR 722.3c).
pub fn is_prepared_copy(g: &Game, copy: ObjectId) -> bool {
    g.battlefield
        .iter()
        .any(|p| g.obj(*p).prepared == Some(copy))
}

/// A prepare-spell copy left exile (it was cast): the permanent loses the prepared
/// designation (CR 722.3c, 601.2i).
pub fn prepared_copy_left_exile(g: &mut Game, copy: ObjectId) {
    for p in g.battlefield.clone() {
        if g.obj(p).prepared == Some(copy) {
            g.objects[p.0 as usize].prepared = None;
        }
    }
}

/// Prepare-spell copies the player may cast: those of prepared permanents they control
/// (CR 722.3c).
pub fn castable_prepared_copies(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.battlefield
        .iter()
        .filter(|x| g.obj(**x).controller == p)
        .filter_map(|x| g.obj(*x).prepared)
        .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Exile)
        .collect()
}
