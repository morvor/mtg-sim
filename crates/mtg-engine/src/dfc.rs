//! Double-faced cards: transforming (CR 701.27, 712).

use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// Transforms a double-faced permanent (CR 701.27a). Returns true if it transformed.
/// It stays the same object, so effects that applied to it keep applying (CR 712.18).
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
    as_transforms(g, id);
    // CR 701.27b: a transform event, not turning face up or down.
    g.emit(Event::Transformed { obj: id });
    true
}

/// CR 712.20: an "As [this permanent] transforms into [this face] ..." ability of the face
/// it transformed into is applied while it's transforming, not afterward: before anything
/// can see it transformed.
fn as_transforms(g: &mut Game, id: ObjectId) {
    g.recompute();
    let effects: Vec<(crate::eval::Ctx, Effect)> = g
        .obj(id)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Replacement(ReplacementDef {
                    event: ReplacementEvent::Transforms,
                    action: ReplacementAction::AsEnters(e),
                    ..
                }) => {
                    let mut ctx = crate::eval::Ctx::for_object(g, id);
                    ctx.link = a.link;
                    Some((ctx, (**e).clone()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect();
    for (mut ctx, e) in effects {
        let before = g.effects.len();
        g.exec(&e, &mut ctx);
        crate::layers::as_enters_copiable(g, id, before);
    }
}
