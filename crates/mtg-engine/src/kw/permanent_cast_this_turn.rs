//! `Filter::Custom` "a creature you cast this turn" (Cycle of Life): a permanent that
//! entered this turn as the spell its controller cast this turn (CR 601, 608.3).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

pub const YOU_CAST_THIS_TURN: &str = "permanent you cast this turn";

pub struct PermanentCastThisTurn;

impl KeywordRules for PermanentCastThisTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != YOU_CAST_THIS_TURN {
            return None;
        }
        let o = g.obj(id);
        Some(
            o.zone == Zone::Battlefield
                && o.entered_turn == g.turn.number
                && o.base_controller == ctx.controller
                && o.cast
                    .as_ref()
                    .is_some_and(|c| c.was_cast && c.turn == g.turn.number),
        )
    }
}

inventory::submit! { KeywordRegistration(&PermanentCastThisTurn) }
