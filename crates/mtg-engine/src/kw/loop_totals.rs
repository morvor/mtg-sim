//! "Starting with you, each player may pay any amount of life. Repeat this process until
//! no one pays life." (Plague of Vermin): the player performing it chooses an amount and
//! pays it (CR 119.4: no more than their life total — in Two-Headed Giant, their team's,
//! adjusted between the players' payments, CR 810.9; any amount can be 0, CR 119.4b). Each
//! payment is a separate life payment; the amounts are added to the player's own running
//! total ([`crate::tallies::LIFE_PAID`]), and a payment above 0 notes that someone paid in
//! this pass of the process ([`crate::tallies::SOMEONE_DID`]).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The custom effect.
pub const PAY_ANY_LIFE_TALLIED: &str = "loop totals: pay any amount of life";

pub struct LoopTotals;

impl KeywordRules for LoopTotals {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PAY_ANY_LIFE_TALLIED {
            return false;
        }
        let p = ctx.controller;
        let max = if g.cant_lose_life(p) {
            0
        } else {
            g.player(p).life.max(0) as i64
        };
        let n = g
            .ask_number(p, ctx.source, "Pay any amount of life", 0, max)
            .clamp(0, max);
        let paid = n > 0 && g.pay_life(p, n as u32);
        ctx.prev_happened = paid;
        if paid {
            ctx.add_tally(crate::tallies::LIFE_PAID, p, n);
            ctx.nums.insert(crate::tallies::SOMEONE_DID, 1);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&LoopTotals) }
