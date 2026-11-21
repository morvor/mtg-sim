//! Rules for token creation used by `oracle/patterns/token_replacements_2.rs`.
//!
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

/// `Effect::Custom` name: inside "create ... tokens that are tapped and attacking that
/// player or a planeswalker they control" (`Effect::TokensJoinCombat`), the planeswalkers
/// the player the tokens attack controls may be attacked instead (CR 508.4).
pub const OR_THEIR_PLANESWALKERS: &str = "token creation:or a planeswalker they control";

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

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != OR_THEIR_PLANESWALKERS {
            return false;
        }
        let v = crate::tokens::TOKEN_DEFENDER;
        let players: Vec<crate::types::PlayerId> = ctx
            .vars
            .get(&v)
            .map(|d| d.iter().filter_map(|e| e.player()).collect())
            .unwrap_or_default();
        let mut d = ctx.vars.get(&v).cloned().unwrap_or_default();
        for p in players {
            for o in g.permanents_controlled_by(p) {
                if g.obj(o).chars.is(crate::types::CardType::Planeswalker) {
                    d.push(crate::types::Entity::Object(o));
                }
            }
        }
        ctx.set_var(v, d);
        true
    }
}

inventory::submit! { KeywordRegistration(&FirstTokensEachTurn) }
