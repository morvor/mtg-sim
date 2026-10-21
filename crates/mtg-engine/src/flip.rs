//! Flip cards (CR 710).
//!
//! A flip card's normal characteristics are its top half; the alternative name, text box,
//! type line, power and toughness of its bottom half apply only while it's a flipped
//! permanent on the battlefield (CR 710.1a, 710.1b, 710.2). Its color and mana cost don't
//! change as it flips, and effects that applied to it keep applying (CR 710.1c): flipping
//! doesn't make it a new object. Flipping is one-way; a permanent that leaves the
//! battlefield forgets it was flipped (CR 710.4, 110.5). A merged permanent flips its
//! flip-card components (CR 730.2h, [`crate::merge::flip`]).
//!
//! "Flip [this permanent]" is the custom effect [`crate::merge::FLIP`], compiled in
//! `oracle/patterns/r730_flip.rs`.

use crate::card::Layout;
use crate::events::Event;
use crate::game::Game;
use crate::object::{Characteristics, FaceState, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a permanent flipped (`obj`).
pub const FLIPPED: &str = "flipped";

/// The flip card whose characteristics the permanent `id` has: its own card, or the flip
/// card it's a copy of (a Dimir Doppelganger that became a copy of Jushi Apprentice can
/// flip: CR 110.5c, 707.3).
fn flip_card_of(g: &Game, id: ObjectId) -> Option<std::sync::Arc<crate::card::CardDef>> {
    let o = g.obj(id);
    let c = o
        .chars
        .printed
        .as_ref()
        .map(|p| p.0.clone())
        .or_else(|| o.card.clone())?;
    (c.layout == Layout::Flip && c.faces.len() > 1).then_some(c)
}

/// Whether `id` is a permanent with a flip card's characteristics (its own, or copied)
/// that hasn't flipped yet.
pub fn can_flip(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    g.is_live(id)
        && o.zone == Zone::Battlefield
        && !o.face_down
        && !o.flipped
        && o.face == FaceState::Front
        && flip_card_of(g, id).is_some()
}

/// The copiable values a flipped permanent gets from a copy effect copying `values`
/// (CR 707.3: the copied information as modified by the copy's status): a copy of an
/// unflipped flip card has that card's alternative characteristics (CR 710.2), with the
/// flip card's color and mana cost (CR 710.1c). `None` if `values` aren't a flip card's.
pub fn flipped_copy_values(values: &Characteristics) -> Option<Characteristics> {
    let card = &values.printed.as_ref()?.0;
    if card.layout != Layout::Flip || card.faces.len() < 2 {
        return None;
    }
    let mut c = card.characteristics(FaceState::Flipped);
    c.printed = values.printed.clone();
    Some(c)
}

/// Flips the permanent `id` (CR 710): it keeps being the same object with the flipped
/// status, and has its alternative characteristics from now on. Returns false if it
/// can't flip (not a flip card, or already flipped: CR 710.4). A permanent that's a copy
/// of a flip card only gets the flipped status: the copy effect applies it
/// ([`flipped_copy_values`]), and it keeps it as it copies other objects (CR 110.5c).
pub fn flip(g: &mut Game, id: ObjectId) -> bool {
    if crate::merge::is_merged(g, id) {
        // CR 730.2h: a merged permanent flips its flip-card components.
        if g.obj(id).flipped || !crate::merge::flip(g, id) {
            return false;
        }
    } else {
        if !can_flip(g, id) {
            return false;
        }
        let own = g
            .obj(id)
            .card
            .clone()
            .filter(|c| c.layout == Layout::Flip && c.faces.len() > 1);
        let o = &mut g.objects[id.0 as usize];
        if let Some(card) = own {
            o.face = FaceState::Flipped;
            o.base = card.characteristics(FaceState::Flipped);
        }
        g.dirty = true;
        g.log(|g| format!("{} flips", g.describe(id)));
    }
    g.objects[id.0 as usize].flipped = true;
    g.emit(Event::Custom {
        name: SmolStr::new(FLIPPED),
        player: Some(g.obj(id).controller),
        obj: Some(id),
        amount: 0,
    });
    true
}
