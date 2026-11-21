//! Game of Chaos: "Flip a coin. If you win the flip, you gain 1 life and target opponent
//! loses 1 life, and you decide whether to flip again. If you lose the flip, you lose 1
//! life and that opponent gains 1 life, and that player decides whether to flip again.
//! Double the life stakes with each flip." Doubling the stakes doubles the life gained and
//! lost (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::dice::CoinFlip;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const CHAOS: &str = "card:Game of Chaos:flip coins for doubling life stakes";
const TEXT: &str = "Flip a coin. If you win the flip, you gain 1 life and target opponent loses 1 life, and you decide whether to flip again. If you lose the flip, you lose 1 life and that opponent gains 1 life, and that player decides whether to flip again. Double the life stakes with each flip.";

/// The stakes stop doubling here (life totals are 32-bit numbers).
const MAX_STAKE: u32 = 1 << 30;

inventory::submit! { ManualAbility {
    card: "Game of Chaos",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Target opponent loses 1 life.")
            .iter()
            .map(|a| super::map_effect(a, TEXT, |_| Effect::Custom(CHAOS.into())))
            .collect()
    },
    reason: "repeated coin flips with doubling life stakes, the winner deciding whether to go on: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CHAOS {
            return false;
        }
        let Some(Entity::Player(opp)) = ctx.targets.first().and_then(|v| v.first()).copied()
        else {
            return true;
        };
        let you = ctx.controller;
        let mut stake: u32 = 1;
        loop {
            crate::dice::flip(g, &CoinFlip::new(), ctx);
            let won = ctx.nums.get(&crate::dice::WINS).copied().unwrap_or(0) > 0;
            let decider = if won {
                g.gain_life(you, stake);
                g.lose_life(opp, stake);
                you
            } else {
                g.lose_life(you, stake);
                g.gain_life(opp, stake);
                opp
            };
            if g.result.is_some() || !g.player(you).in_game() || !g.player(opp).in_game() {
                break;
            }
            stake = (stake * 2).min(MAX_STAKE);
            if !g.ask_yes_no(decider, ctx.source, &format!("Flip again for {stake} life?"), false)
            {
                break;
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
