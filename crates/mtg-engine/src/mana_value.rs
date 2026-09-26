//! Mana value (CR 202.3): the total amount of mana in an object's mana cost, with the
//! special cases for objects with no mana cost (CR 202.3a): the back face of a nonmodal
//! double-faced card (CR 202.3b, 712.8c, 712.8e) and melded permanents (CR 202.3c,
//! 712.8g). X is 0 except on the stack (CR 202.3e); hybrid and Phyrexian symbols are
//! counted by [`crate::mana::ManaSymbol::mana_value`] (CR 202.3f, 202.3g); split cards
//! get their combined or half's mana cost from their characteristics (CR 202.3d).

use crate::card::Layout;
use crate::game::{Affected, Game, Layer1};
use crate::object::{Characteristics, FaceState, GameObject, ObjKind, Zone};
use crate::types::ObjectId;

/// The value of X for mana value purposes: the number chosen for it while the object is
/// on the stack, 0 anywhere else (CR 202.3e).
fn x_for(o: &GameObject) -> u32 {
    if o.zone == Zone::Stack {
        o.stack.as_ref().and_then(|s| s.x).unwrap_or(0).max(0) as u32
    } else {
        0
    }
}

/// Whether a copy effect (CR 707) currently sets the object's copiable values: the object
/// is a copy of something else rather than its own card.
pub fn is_copying(g: &Game, id: ObjectId) -> bool {
    g.effects.iter().any(|e| {
        matches!(e.layer1, Some(Layer1::Copy { .. }))
            && matches!(&e.affected, Affected::Objects(v) if v.contains(&id))
    })
}

/// Mana value of the object `id` with characteristics `c` (its current characteristics,
/// or those it would have as a spell, CR 601.3e).
pub fn mana_value_with(g: &Game, id: ObjectId, c: &Characteristics) -> u32 {
    let o = g.obj(id);
    let x = x_for(o);
    if let Some(mc) = &c.mana_cost {
        return mc.mana_value_with_x(x);
    }
    // CR 202.3a: no mana cost means 0, except for the back face of a nonmodal
    // double-faced card and a melded permanent. A copy of either has mana value 0 (CR
    // 202.3b, 202.3c), even if the object representing the copy is itself such a card.
    if o.kind != ObjKind::Card || o.face_down || is_copying(g, id) {
        return 0;
    }
    match o.face {
        // CR 202.3b, 712.8c, 712.8e: the mana cost of its front face.
        FaceState::Back => match &o.card {
            Some(card)
                if matches!(card.layout, Layout::Transform | Layout::Battle)
                    && card.faces.len() > 1 =>
            {
                card.front()
                    .chars
                    .mana_cost
                    .as_ref()
                    .map_or(0, |m| m.mana_value_with_x(x))
            }
            _ => 0,
        },
        // CR 202.3c, 712.8g: the combined mana cost of the front faces of each card
        // that represents it.
        FaceState::Melded => o
            .merged_with
            .iter()
            .map(|part| {
                let p = g.obj(*part);
                match &p.card {
                    Some(card) if p.kind == ObjKind::Card => card
                        .front()
                        .chars
                        .mana_cost
                        .as_ref()
                        .map_or(0, |m| m.mana_value()),
                    _ => 0,
                }
            })
            .sum(),
        _ => 0,
    }
}
