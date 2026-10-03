//! "Join forces — Starting with you, each player may pay any amount of mana." (Alliance of
//! Arms, Minds Aglow, Shared Trauma, Collective Voyage): each player in turn, knowing what
//! the players before paid (CR 101.4b), chooses an amount of mana they can pay and pays it
//! (generic mana, CR 107.4); the amounts are added up for "the total amount of mana paid
//! this way". The ability word has no rules meaning (CR 207.2c); the compiler performs this
//! effect as each player (see `oracle::patterns::iteration_grammar::join_forces`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The effect: the player performing it ("you") may pay any amount of mana; what they pay
/// is added to [`crate::oracle::patterns::iteration_grammar::MANA_PAID`].
pub const PAY_ANY_AMOUNT: &str = "join forces: pay any amount of mana";

pub struct JoinForces;

impl KeywordRules for JoinForces {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PAY_ANY_AMOUNT {
            return false;
        }
        let p = ctx.controller;
        if crate::multiplayer::cant_pay(g, p) {
            return true;
        }
        let max = g.max_mana_available(p) as i64;
        let src = ctx
            .source
            .or(ctx.stack_obj)
            .unwrap_or(crate::types::ObjectId(0));
        let n = match g.ask(
            p,
            Decision::ChooseX {
                source: src,
                min: 0,
                max,
            },
        ) {
            Answer::Number(n) if n >= 0 => n.min(max),
            _ => 0,
        };
        if n == 0 {
            return true;
        }
        let Some(mana) = crate::mana::ManaCost::parse(&format!("{{{n}}}")) else {
            return true;
        };
        if g.pay_cost(p, &Cost::mana(mana), ctx.source, ctx) {
            let var = crate::oracle::patterns::iteration_grammar::MANA_PAID;
            let total = ctx.nums.get(&var).copied().unwrap_or(0);
            ctx.nums.insert(var, total + n);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&JoinForces) }
