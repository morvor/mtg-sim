//! CR 702.19 Trample and trample over planeswalkers. The damage assignment itself is in
//! `combat.rs` (CR 510.1c–d); this module identifies the variants and computes the
//! amounts that count as "lethal" while damage is being assigned.

use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// The text marking the trample over planeswalkers variant of a trample keyword.
pub const OVER_PLANESWALKERS: &str = "trample over planeswalkers";

/// Whether a trample keyword instance is trample over planeswalkers (CR 702.19c).
pub fn is_over_planeswalkers(kw: &Keyword) -> bool {
    kw.kind == KeywordKind::Trample
        && kw
            .text
            .as_deref()
            .is_some_and(|t| t.trim().eq_ignore_ascii_case(OVER_PLANESWALKERS))
}

/// Whether the creature has trample over planeswalkers (CR 702.19c).
pub fn over_planeswalkers(g: &Game, id: ObjectId) -> bool {
    g.obj(id).chars.keywords().any(is_over_planeswalkers)
}

/// Damage already being assigned to `e` by other creatures in this combat damage step.
pub fn pending_to(pending: &[(ObjectId, Entity, u32)], e: Entity) -> u32 {
    pending
        .iter()
        .filter(|(_, r, _)| *r == e)
        .map(|(_, _, n)| *n)
        .sum()
}

/// Lethal damage for `source` to assign to the blocking creature `creature` (CR 702.19b,
/// 702.2c): its toughness minus damage already marked on it and damage other creatures
/// are assigning to it in the same combat damage step; any nonzero amount from a source
/// with deathtouch. Abilities and effects that would change the damage actually dealt
/// aren't considered.
pub fn lethal_damage(
    g: &Game,
    source: ObjectId,
    creature: ObjectId,
    pending: &[(ObjectId, Entity, u32)],
) -> u32 {
    let e = Entity::Object(creature);
    let deathtouch_pending = pending.iter().any(|(s, r, n)| {
        *r == e && *n > 0 && g.obj(*s).has_keyword(KeywordKind::Deathtouch)
    });
    if deathtouch_pending {
        return 0;
    }
    let base = crate::combat::lethal_damage(g, source, creature);
    let o = g.obj(creature);
    let left = (o.toughness() - o.damage as i32 - pending_to(pending, e) as i32).max(0) as u32;
    base.min(left)
}

/// Damage still needed for the attacked planeswalker before excess damage may be assigned
/// to its controller (CR 702.19c): its loyalty minus damage other creatures are assigning
/// to it in the same step. Deathtouch doesn't matter for planeswalkers.
pub fn planeswalker_damage_needed(
    g: &Game,
    pw: ObjectId,
    pending: &[(ObjectId, Entity, u32)],
) -> u32 {
    let loyalty = g.obj(pw).loyalty().max(0) as u32;
    loyalty.saturating_sub(pending_to(pending, Entity::Object(pw)))
}
