//! Double-faced cards: transforming (CR 701.27, 712).

use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// Transforms a double-faced permanent (CR 701.27a). Returns true if it transformed.
pub fn transform(g: &mut Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    if o.zone != Zone::Battlefield || !g.is_live(id) {
        return false;
    }
    let Some(card) = o.card.clone() else {
        return false;
    };
    // CR 701.27c: only transforming double-faced cards can transform (not MDFCs).
    if !matches!(
        card.layout,
        crate::card::Layout::Transform
            | crate::card::Layout::Meld
            | crate::card::Layout::DoubleFacedToken
            | crate::card::Layout::Battle
    ) {
        return false;
    }
    if card.faces.len() < 2 {
        return false;
    }
    let new_face = if o.face == FaceState::Back {
        FaceState::Front
    } else {
        FaceState::Back
    };
    let ts = g.new_timestamp();
    let ob = &mut g.objects[id.0 as usize];
    ob.face = new_face;
    ob.base = card.characteristics(new_face);
    ob.timestamp = ts; // CR 613.7g
    g.dirty = true;
    g.emit(Event::Transformed { obj: id });
    true
}
