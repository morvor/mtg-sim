//! CR 701.27 Transform.
//!
//! * Only permanents represented by double-faced cards or double-faced tokens can
//!   transform (CR 701.27a, 701.27c, 712.9) — modal double-faced cards included
//!   (CR 712.3) — but not meld cards (CR 712.4c) or face-down permanents (CR 712.15a). If
//!   the face it would transform into is an instant or sorcery face, nothing happens
//!   (CR 701.27d).
//! * Transforming isn't turning face up or face down (CR 701.27b): it's a separate event
//!   (`Event::Transformed`).
//! * "Whenever [a permanent] transforms into [a quality]" triggers if it has that quality
//!   immediately after it transforms (CR 701.27e): the trigger's filter is checked against
//!   the permanent after the event.
//! * An activated or triggered ability of a permanent that tries to transform it does so
//!   only if it hasn't transformed since the ability was put onto the stack; for a delayed
//!   triggered ability, since that ability was created (CR 701.27f).
//! * A "transformed permanent" is a double-faced permanent with its back face up, and never
//!   a melded or merged permanent (CR 701.27g): the custom filter [`TRANSFORMED`]. (Older
//!   rulings also exclude modal double-faced permanents, from when those couldn't
//!   transform; the current rule doesn't.)

use crate::ability::*;
use crate::card::Layout;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::*;
use crate::types::*;
use std::collections::BTreeMap;

/// When each permanent last transformed.
#[derive(Clone, Debug, Default)]
pub struct TransformState {
    pub last: BTreeMap<ObjectId, Timestamp>,
}

/// `Ctx::nums` variable of a delayed triggered ability: the timestamp counter when it was
/// created (CR 701.27f).
pub const DELAYED_CREATED: Var = vars::USER + 730;

/// `Filter::Custom`: "transformed permanent" (CR 701.27g).
pub const TRANSFORMED: &str = "transformed";

/// A double-faced card or token that isn't a meld card (CR 712.9).
fn transforming_layout(l: Layout) -> bool {
    l.is_double_faced() && l != Layout::Meld
}

/// Whether the permanent `id` can transform now (CR 701.27a, 701.27c, 701.27d).
pub fn can_transform(g: &Game, id: ObjectId) -> bool {
    can_transform_by(g, id, false)
}

/// [`can_transform`], for a transformation caused by the permanent's own daybound or
/// nightbound ability when `day_night` is set: a permanent with daybound or nightbound
/// can't transform any other way (CR 702.145b, 702.145e).
pub fn can_transform_by(g: &Game, id: ObjectId, day_night: bool) -> bool {
    let o = g.obj(id);
    if o.zone != Zone::Battlefield || !g.is_live(id) || o.face_down {
        return false;
    }
    if !day_night && crate::kw::daybound::transforms_only_by_day_night(g, id) {
        return false;
    }
    // "Can't transform" (and so can't convert, CR 701.28f).
    if crate::kwa::convert::cant_transform(g, id) {
        return false;
    }
    let Some(card) = o.card.as_ref() else {
        return false;
    };
    if !transforming_layout(card.layout) || card.faces.len() < 2 {
        return false;
    }
    let new_face = match o.face {
        FaceState::Front => 1,
        FaceState::Back => 0,
        _ => return false,
    };
    // CR 701.27d: not into an instant or sorcery face.
    let types = &card.faces[new_face].chars.card_types;
    !types.contains(CardType::Instant) && !types.contains(CardType::Sorcery)
}

/// Whether `id` is represented by a double-faced card or a double-faced token (a meld
/// card and a melded permanent included, CR 712.1), as opposed to a single-faced object
/// that may be a copy of one (CR 712.9).
pub fn is_double_faced_permanent(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    matches!(o.kind, ObjKind::Card | ObjKind::Token)
        && o.card.as_ref().is_some_and(|c| c.layout.is_double_faced())
}

/// Whether the card `id` can be put onto the battlefield transformed (with its back face
/// up): a double-faced card or token that can transform (not a meld card, CR 712.4c), or
/// a copy of one cast or copied as a spell (CR 712.11a, 712.13a). A card that isn't
/// double-faced stays in its current zone (CR 712.14a).
pub fn can_enter_transformed(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    matches!(
        o.kind,
        ObjKind::Card | ObjKind::Token | ObjKind::CardCopy | ObjKind::SpellCopy
    ) && o
        .card
        .as_ref()
        .is_some_and(|c| transforming_layout(c.layout) && c.faces.len() >= 2)
}

/// Records that `id` transformed, with the new timestamp it got.
pub fn record(g: &mut Game, id: ObjectId, ts: Timestamp) {
    g.transforms.last.insert(id, ts);
}

/// CR 701.27f: whether the resolving ability (`ctx`) may transform `id`.
pub fn ability_may_transform(g: &Game, id: ObjectId, ctx: &Ctx) -> bool {
    if ctx.source != Some(id) {
        return true;
    }
    let Some(so) = ctx.stack_obj else {
        return true;
    };
    let s = g.obj(so);
    let Some(si) = s.stack.as_deref() else {
        return true;
    };
    if !matches!(
        si.kind,
        StackKind::Activated { source, .. } | StackKind::Triggered { source, .. } if source == id
    ) {
        return true;
    }
    let since = match ctx.nums.get(&DELAYED_CREATED) {
        Some(t) => *t as Timestamp,
        None => s.timestamp,
    };
    !g.transforms.last.get(&id).is_some_and(|t| *t >= since)
}

/// The context saved with a delayed triggered ability as it's created (CR 701.27f).
pub fn delayed_ctx(g: &Game, ctx: &Ctx) -> Ctx {
    let mut c = ctx.clone();
    c.nums.insert(DELAYED_CREATED, g.next_timestamp as i64);
    c
}

/// A double-faced permanent with its back face up; never a melded or merged permanent
/// (CR 701.27g).
pub fn is_transformed(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.zone == Zone::Battlefield
        && o.face == FaceState::Back
        && o.merged_with.is_empty()
        && o.card
            .as_ref()
            .is_some_and(|c| transforming_layout(c.layout) && c.faces.len() >= 2)
}

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
    (name == TRANSFORMED).then(|| is_transformed(g, id))
}
