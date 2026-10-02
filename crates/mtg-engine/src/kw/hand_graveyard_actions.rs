//! "That many" after a card action performed by each of several players ("Each player
//! discards all the cards in their hand, then draws that many cards"): each player's
//! later instruction uses the number of cards *that player* acted on. The grammar
//! (`oracle/patterns/hand_graveyard_grammar.rs`) records the number per iterated player
//! with [`RECORD_THAT_MANY`], and reads it back with the value [`THAT_MANY`].

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The number of cards the most recent card action affected (any player).
pub const THAT_MANY_VAR: Var = vars::USER + 6102;
/// Per-player numbers: `THAT_MANY_BY_PLAYER + player index`.
const THAT_MANY_BY_PLAYER: Var = vars::USER + 6120;

/// `Effect::Custom`: copies [`THAT_MANY_VAR`] into the iterated player's slot (when the
/// action is performed by each of several players in turn).
pub const RECORD_THAT_MANY: &str = "record that many for the iterated player";
/// `Value::Custom`: the iterated player's number if they performed the action, otherwise
/// [`THAT_MANY_VAR`].
pub const THAT_MANY: &str = "that many";

pub struct HandGraveyardActions;

impl KeywordRules for HandGraveyardActions {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, _g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RECORD_THAT_MANY {
            return false;
        }
        if let Some(p) = ctx.iter_player {
            let n = ctx.nums.get(&THAT_MANY_VAR).copied().unwrap_or(0);
            ctx.nums.insert(THAT_MANY_BY_PLAYER + p.0 as Var, n);
        }
        true
    }

    fn custom_value(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != THAT_MANY {
            return None;
        }
        let per = ctx
            .iter_player
            .and_then(|p| ctx.nums.get(&(THAT_MANY_BY_PLAYER + p.0 as Var)).copied());
        Some(per.unwrap_or_else(|| ctx.nums.get(&THAT_MANY_VAR).copied().unwrap_or(0)))
    }
}

inventory::submit! { KeywordRegistration(&HandGraveyardActions) }
