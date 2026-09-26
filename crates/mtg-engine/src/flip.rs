//! Flip cards (CR 710).
//!
//! A flip card's normal characteristics are its top half; the alternative name, text box,
//! type line, power and toughness of its bottom half apply only while it's a flipped
//! permanent on the battlefield (CR 710.1a, 710.1b, 710.2). Its color and mana cost don't
//! change as it flips, and effects that applied to it keep applying (CR 710.1c): flipping
//! doesn't make it a new object. Flipping is one-way; a permanent that leaves the
//! battlefield forgets it was flipped (CR 710.4, 110.5).

use crate::card::Layout;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::{FaceState, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` name: "flip [this permanent]" (the source of the effect).
pub const FLIP_SELF: &str = "flip this permanent";
/// `Event::Custom` name: a permanent flipped (`obj`).
pub const FLIPPED: &str = "flipped";

/// Whether `id` is a permanent represented by a flip card that hasn't flipped yet.
pub fn can_flip(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    g.is_live(id)
        && o.zone == Zone::Battlefield
        && !o.face_down
        && !o.flipped
        && o.face == FaceState::Front
        && o.card
            .as_ref()
            .is_some_and(|c| c.layout == Layout::Flip && c.faces.len() > 1)
}

/// Flips the permanent `id` (CR 710): it keeps being the same object with the flipped
/// status, and has its alternative characteristics from now on. Returns false if it
/// can't flip (not a flip card, or already flipped: CR 710.4).
pub fn flip(g: &mut Game, id: ObjectId) -> bool {
    if !can_flip(g, id) {
        return false;
    }
    let Some(card) = g.obj(id).card.clone() else {
        return false;
    };
    let o = &mut g.objects[id.0 as usize];
    o.flipped = true;
    o.face = FaceState::Flipped;
    o.base = card.characteristics(FaceState::Flipped);
    g.dirty = true;
    g.log(|g| format!("{} flips", g.describe(id)));
    g.emit(Event::Custom {
        name: SmolStr::new(FLIPPED),
        player: Some(g.obj(id).controller),
        obj: Some(id),
        amount: 0,
    });
    true
}

/// "Flip [this permanent]". Returns false if `name` isn't a flip effect.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &Ctx) -> bool {
    if name != FLIP_SELF {
        return false;
    }
    if let Some(src) = ctx.source {
        flip(g, src);
    }
    true
}
