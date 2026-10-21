//! "The first time you would create one or more tokens each turn, ..." (Moonlit
//! Meditation, Mirrormind Crown, Esix, Fractal Bloom): a token-creation replacement
//! effect that applies only to the first token-creation event of that player this turn
//! (CR 614.1a). `Condition::Custom(FIRST_TOKENS)`, evaluated with the creation event as
//! the trigger event: no token has been created under that player's control this turn.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::token_replacements_2::FIRST_TOKENS;

struct FirstTokensEachTurn;

impl KeywordRules for FirstTokensEachTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != FIRST_TOKENS {
            return None;
        }
        let p = ctx
            .event
            .as_ref()
            .and_then(|e| e.player)
            .unwrap_or(ctx.controller);
        Some(g.history.tokens_created.get(&p).copied().unwrap_or(0) == 0)
    }
}

inventory::submit! { KeywordRegistration(&FirstTokensEachTurn) }
