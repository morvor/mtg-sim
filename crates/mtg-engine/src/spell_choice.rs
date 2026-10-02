//! Choosing a card to cast "a spell with certain characteristics" during resolution
//! ("you may cast an instant or sorcery spell with mana value 2 or less from your
//! graveyard" — Sword of Once and Future). The characteristics considered are those of
//! the spell the card would become, not the card's (CR 601.3e): one half of a split card
//! (CR 709.3), an Adventure or Omen (CR 715.3, 720.3), either face of a modal
//! double-faced card. A card is a choice if any face or half it could be cast as has
//! them, and it's then cast as one of those.

use crate::ability::{Filter, Sel, ZoneKind};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::FaceState;
use crate::types::*;

/// The faces or halves `card` could be cast as whose characteristics as a spell match `f`
/// (CR 601.3e). For a card with a single way to be cast, the card itself is checked.
pub fn matching_faces(g: &Game, card: ObjectId, f: &Filter, ctx: &Ctx) -> Vec<FaceState> {
    let faces = crate::casting::castable_faces(g, card);
    if faces.len() < 2 {
        return if g.matches(card, f, ctx) {
            faces
        } else {
            vec![]
        };
    }
    faces
        .into_iter()
        .filter(|face| {
            let chars = g.face_characteristics(card, *face);
            crate::casting::matches_with_chars(g, card, &chars, f, ctx)
        })
        .collect()
}

/// Resolves the selection of cards an effect lets a player cast: for a choice among cards
/// in a zone ("an instant or sorcery spell ... from your graveyard"), the chosen cards
/// with the faces they may be cast as. Any other selection is resolved as usual (its
/// cards may be cast as any face). Returns None if `what` isn't such a choice.
pub fn choose_cards_to_cast(
    g: &mut Game,
    what: &Sel,
    ctx: &mut Ctx,
) -> Option<Vec<(ObjectId, Vec<FaceState>)>> {
    let Sel::Choose {
        chooser,
        filter,
        count,
        up_to,
        store,
    } = what
    else {
        return None;
    };
    let zone = filter.zone().filter(|z| *z != ZoneKind::Battlefield)?;
    let p = g.eval_player(chooser, ctx).unwrap_or(ctx.controller);
    let n = g.eval_value(count, ctx).max(0) as u32;
    let mut faces: Vec<(ObjectId, Vec<FaceState>)> = g
        .objects_in_zone_kind(zone)
        .into_iter()
        .map(|o| (o, matching_faces(g, o, filter, ctx)))
        .filter(|(_, f)| !f.is_empty())
        .collect();
    let mut cands: Vec<ObjectId> = faces.iter().map(|(o, _)| *o).collect();
    // CR 723.4: a controlled player can't be made to choose cards from outside the game.
    crate::player_control::visible_choices(g, p, &mut cands);
    let min = if *up_to { 0 } else { n.min(cands.len() as u32) };
    let picked = crate::zones::choose_objects(g, p, ctx.source, "Choose", cands, min, n);
    if let Some(v) = store {
        ctx.vars
            .insert(*v, picked.iter().map(|o| Entity::Object(*o)).collect());
    }
    faces.retain(|(o, _)| picked.contains(o));
    Some(faces)
}
