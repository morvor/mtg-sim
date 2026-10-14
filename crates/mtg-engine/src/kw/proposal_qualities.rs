//! Qualities of a spell that depend on the choices made as it's proposed (CR 601.2b–c),
//! for effects that let a spell with those qualities be cast as though it had flash:
//! "You may cast this spell as though it had flash if X is 3 or less." (Silver Scrutiny),
//! "... if it targets a permanent you control." (Flash Photography). The player may begin
//! to cast the spell if some choice could give it the quality (CR 601.3b); once the
//! proposal is complete, the spell must have it, or casting it is illegal (CR 601.2e,
//! checked in `casting.rs`). The filters are written `Or(not on the stack, quality)`:
//! optimistic for a card that isn't a spell yet, exact for the proposed spell.
//!
//! Also "the spell's caster is the active player" ([`CASTER_IS_ACTIVE`]: "The first spell
//! each player casts during each of their turns", Weftwalking): its controller on the
//! stack, its owner (who would cast it) elsewhere.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Filter::Custom` prefix, followed by N: the spell's X is N or less (a spell being
/// proposed with X chosen; a spell without X has X = 0, CR 107.3).
pub const X_AT_MOST: &str = "proposal:x is at most ";
/// `Filter::Custom`: the spell's caster (its controller on the stack, else the player whose
/// hand it's in, else its owner) is the active player.
pub const CASTER_IS_ACTIVE: &str = "proposal:its caster is the active player";

/// The player who casts or would cast `id`.
pub fn caster(g: &Game, id: ObjectId) -> PlayerId {
    let o = g.obj(id);
    match o.zone {
        Zone::Stack => o.controller,
        Zone::Hand(p) => p,
        _ => o.owner,
    }
}

pub struct ProposalQualities;

impl KeywordRules for ProposalQualities {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name == CASTER_IS_ACTIVE {
            return Some(caster(g, id) == g.turn.active);
        }
        let n: i32 = name.strip_prefix(X_AT_MOST)?.parse().ok()?;
        let x = g
            .obj(id)
            .stack
            .as_deref()
            .and_then(|s| s.x)
            .unwrap_or(0);
        Some(x <= n)
    }
}

inventory::submit! { KeywordRegistration(&ProposalQualities) }
