//! Double-faced cards: transforming (CR 701.27, 712).

use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// Transforms a double-faced permanent (CR 701.27a). Returns true if it transformed.
pub fn transform(g: &mut Game, id: ObjectId) -> bool {
    // CR 701.27c, 701.27d, 712.4c, 712.15a: otherwise nothing happens.
    if !crate::transform_rules::can_transform(g, id) {
        return false;
    }
    let o = g.obj(id);
    let Some(card) = o.card.clone() else {
        return false;
    };
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
    crate::transform_rules::record(g, id, ts);
    // CR 701.27b: a transform event, not turning face up or down.
    g.emit(Event::Transformed { obj: id });
    true
}
