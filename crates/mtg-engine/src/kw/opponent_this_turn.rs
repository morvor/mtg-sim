//! Conditions about what an opponent did this turn, as the Zendikar Traps' alternative
//! cost conditions ask (CR 118.9, 601.2b): "If an opponent gained life this turn", "If an
//! opponent drew three or more cards this turn", "If an opponent had two or more creatures
//! enter the battlefield under their control this turn". They look at the turn's history
//! (`TurnHistory`): each permanent is looked at as it entered.
//!
//! The conditions are `Condition::Custom` names built by [`gained_life`], [`drew_cards`]
//! and [`had_enter`] (compiled by `oracle/patterns/opponent_this_turn.rs`).

use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

const PREFIX: &str = "an opponent this turn:";

/// "an opponent gained life this turn".
pub fn gained_life() -> String {
    format!("{PREFIX}gained life")
}

/// "an opponent drew N or more cards this turn".
pub fn drew_cards(n: u32) -> String {
    format!("{PREFIX}drew:{n}")
}

/// "an opponent had N or more [card type] permanents enter the battlefield under their
/// control this turn".
pub fn had_enter(n: u32, t: CardType) -> String {
    format!("{PREFIX}entered:{n}:{t:?}")
}

fn card_type(s: &str) -> Option<CardType> {
    [
        CardType::Artifact,
        CardType::Creature,
        CardType::Enchantment,
        CardType::Land,
        CardType::Planeswalker,
        CardType::Battle,
    ]
    .into_iter()
    .find(|t| format!("{t:?}") == s)
}

/// Whether some opponent of `ctx.controller` did it this turn.
fn holds(g: &Game, rest: &str, ctx: &Ctx) -> Option<bool> {
    let opponents: Vec<PlayerId> = g
        .players
        .iter()
        .filter(|q| g.are_opponents(ctx.controller, q.id))
        .map(|q| q.id)
        .collect();
    let h = &g.history;
    if rest == "gained life" {
        return Some(
            opponents
                .iter()
                .any(|q| h.life_gained.get(q).copied().unwrap_or(0) > 0),
        );
    }
    if let Some(n) = rest.strip_prefix("drew:") {
        let n: u32 = n.parse().ok()?;
        return Some(
            opponents
                .iter()
                .any(|q| h.cards_drawn.get(q).copied().unwrap_or(0) >= n),
        );
    }
    let (n, t) = rest.strip_prefix("entered:")?.split_once(':')?;
    let n: usize = n.parse().ok()?;
    let t = card_type(t)?;
    Some(opponents.iter().any(|q| {
        h.permanents_entered
            .iter()
            .filter(|e| {
                e.controller == *q
                    && match &e.as_entered {
                        Some((chars, _)) => chars.is(t),
                        None => g.try_obj(e.id).is_some_and(|o| o.chars.is(t)),
                    }
            })
            .count()
            >= n
    }))
}

struct OpponentThisTurn;

impl super::KeywordRules for OpponentThisTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        holds(g, name.strip_prefix(PREFIX)?, ctx)
    }
}

inventory::submit! { super::KeywordRegistration(&OpponentThisTurn) }
