//! "creatures that blocked or were blocked by it this turn", "creatures that were blocked
//! by that creature this turn" (Venomous Breath, Glyph of Doom): who blocked whom this
//! turn, from the turn's block declarations and blocks added by effects (CR 509.1,
//! 509.3). The creature named is the one a delayed triggered ability remembers
//! ([`crate::oracle::patterns::delayed_grammar::REFERENT`]); a creature that left the
//! battlefield since is a new object that blocked nothing (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: an attacking creature one of the remembered creatures blocked this
/// turn.
pub const BLOCKED_BY_REFERENT: &str = "blocked by the remembered creature this turn";
/// `Filter::Custom`: a creature that blocked, or was blocked by, one of the remembered
/// creatures this turn.
pub const BLOCKED_OR_BLOCKED_BY_REFERENT: &str =
    "blocked or blocked by the remembered creature this turn";

/// The (blocker, attacker) pairs of this turn.
fn blocks_this_turn(g: &Game) -> Vec<(ObjectId, ObjectId)> {
    let mut out = Vec::new();
    for e in &g.turn_events {
        match e {
            Event::BlockersDeclared { blocks } => out.extend(blocks.iter().copied()),
            Event::BlockAdded {
                blocker, attacker, ..
            } => out.push((*blocker, *attacker)),
            _ => {}
        }
    }
    out
}

pub struct BlockedThisTurn;

impl KeywordRules for BlockedThisTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let either = match name {
            BLOCKED_BY_REFERENT => false,
            BLOCKED_OR_BLOCKED_BY_REFERENT => true,
            _ => return None,
        };
        let remembered: Vec<ObjectId> = ctx
            .vars
            .get(&crate::oracle::patterns::delayed_grammar::REFERENT)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        Some(blocks_this_turn(g).iter().any(|(blocker, attacker)| {
            (*attacker == id && remembered.contains(blocker))
                || (either && *blocker == id && remembered.contains(attacker))
        }))
    }
}

inventory::submit! { KeywordRegistration(&BlockedThisTurn) }
