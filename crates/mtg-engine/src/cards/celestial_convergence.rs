//! Celestial Convergence: "At the beginning of your upkeep, remove an omen counter from ~.
//! If there are no omen counters on ~, the player with the highest life total wins the
//! game. If two or more players are tied for highest life total, the game is a draw."

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const HIGHEST: &str = "card:Celestial Convergence:the player with the highest life total wins";
const TEXT: &str = "At the beginning of your upkeep, remove an omen counter from ~. If there are no omen counters on ~, the player with the highest life total wins the game. If two or more players are tied for highest life total, the game is a draw.";

/// Replaces "you win the game" with the highest-life-total rule.
fn replace_win(e: Effect) -> Effect {
    match e {
        Effect::WinGame { .. } => Effect::Custom(HIGHEST.into()),
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(replace_win).collect()),
        Effect::If {
            cond,
            then,
            otherwise,
        } => Effect::If {
            cond,
            then: Box::new(replace_win(*then)),
            otherwise: Box::new(replace_win(*otherwise)),
        },
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Celestial Convergence",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of your upkeep, remove an omen counter from ~. If there are no omen counters on ~, you win the game.")
            .iter()
            .map(|a| map_effect(a, TEXT, replace_win))
            .collect()
    },
    reason: "the player with the highest life total wins (a draw on a tie): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != HIGHEST {
            return false;
        }
        let players = g.players_in_game();
        let Some(best) = players.iter().map(|p| g.player(*p).life).max() else {
            return true;
        };
        let top: Vec<_> = players
            .into_iter()
            .filter(|p| g.player(*p).life == best)
            .collect();
        if top.len() == 1 {
            g.players_win(&top);
        } else {
            // CR 104.4a: the game is a draw.
            g.game_is_a_draw(ctx.controller);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
