//! "As this creature enters, pay any amount of life. The amount you pay can't be more
//! than [value]." (Nameless Race, Minion of the Wastes, Phyrexian Processor): as the
//! permanent enters (CR 614.12a), its controller chooses an amount and pays that much
//! life (CR 119.4: no more than their life total, unless it's 0); the amount is the
//! number "the life paid as it entered" refers to (CR 607.2g), kept on the permanent.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The custom effect: choose an amount, pay it, and note it on the entering object.
pub const PAY_ANY_LIFE: &str = "pay any amount of life";
/// The most that may be paid, if the text sets one (stored before [`PAY_ANY_LIFE`]).
pub const CAP: Var = vars::USER + 9450;

pub struct PayAnyAmountOfLife;

impl KeywordRules for PayAnyAmountOfLife {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PAY_ANY_LIFE {
            return false;
        }
        let p = ctx.controller;
        let mut max = if g.cant_lose_life(p) {
            0
        } else {
            g.player(p).life.max(0) as i64
        };
        if let Some(cap) = ctx.nums.get(&CAP) {
            max = max.min((*cap).max(0));
        }
        let n = g
            .ask_number(p, ctx.source, "Pay any amount of life", 0, max)
            .clamp(0, max);
        let paid = if g.pay_life(p, n as u32) { n } else { 0 };
        if let Some(src) = ctx.source {
            g.objects[src.0 as usize].choices.number = Some(paid as i32);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&PayAnyAmountOfLife) }
