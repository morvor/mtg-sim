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

/// The preparation card whose prepare spell the object `o` has: the alternative
/// characteristics are part of its copiable values (CR 722.2a, 722.2b), so a copy of a
/// preparation card's permanent has them too.
fn prepare_card(g: &Game, o: ObjectId) -> Option<std::sync::Arc<crate::card::CardDef>> {
    let ob = g.obj(o);
    let card = match &ob.copiable.printed {
        Some(p) => Some(p.0.clone()),
        // Characteristics not computed (a card in a library): its own card.
        None if matches!(ob.zone, Zone::Library(_) | Zone::Nowhere) => ob.card.clone(),
        None => None,
    }?;
    (card.layout == crate::card::Layout::Prepare && card.faces.len() > 1).then_some(card)
}

/// Whether an object has a prepare spell (CR 722.2a), even if it doesn't use it.
pub fn has_prepare_spell(g: &Game, o: ObjectId) -> bool {
    prepare_card(g, o).is_some()
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
    let Some(card) = prepare_card(g, o) else {
        return;
    };
    let controller = g.obj(o).controller;
    // Only the prepare spell's characteristics, ignoring copy exceptions that apply to
    // the permanent (CR 722.3c).
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

/// A prepared permanent's prepare-spell copy `copy` was cast as the spell `spell`: it's
/// a spell cast as a prepare spell (CR 722.3c, 722.3d).
pub fn prepare_spell_cast(g: &mut Game, copy: ObjectId, spell: ObjectId) {
    if is_prepared_copy(g, copy) {
        g.special.prepare_spells.push(spell);
    }
}

/// A spell was copied (CR 707.10): a copy of a prepare spell is a prepare spell too
/// (CR 722.3d).
pub fn spell_copied(g: &mut Game, spell: ObjectId, copy: ObjectId) {
    if g.special.prepare_spells.contains(&spell) {
        g.special.prepare_spells.push(copy);
    }
}

/// Whether the spell `id` was cast as a prepare spell, or is a copy of such a spell
/// (CR 722.3d).
pub fn is_prepare_spell(g: &Game, id: ObjectId) -> bool {
    g.obj(id).zone == Zone::Stack && g.special.prepare_spells.contains(&id)
}
