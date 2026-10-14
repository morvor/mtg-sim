//! Game-rules behaviors used by the oracle patterns `oracle/patterns/game_rules_*.rs`:
//! energy payments (CR 107.14), and other named custom behaviors.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

const PAY_ENERGY: &str = "game rules:pay energy:";

/// `Effect::Custom` name: "Pay {E}{E}." as an instruction: the controller pays `n` energy
/// if they can (CR 107.14: they can't pay more than they have); whether they did is
/// recorded for "If you can't, ...".
pub fn pay_energy(n: i32) -> String {
    format!("{PAY_ENERGY}{n}")
}

pub fn is_pay_energy(name: &str) -> bool {
    name.starts_with(PAY_ENERGY)
}

/// The amount of a mandatory energy payment.
pub fn pay_energy_amount(name: &str) -> Option<i32> {
    name.strip_prefix(PAY_ENERGY)?.parse().ok()
}

pub struct GameRules;

impl KeywordRules for GameRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(n) = pay_energy_amount(name) {
            let p = ctx.controller;
            let cost = Cost::free().with(CostPart::PayEnergy(Value::c(n)));
            let paid = g.player(p).counter(counters::ENERGY) >= n.max(0) as u32
                && g.pay_cost(p, &cost, ctx.source, ctx);
            ctx.prev_happened = paid;
            return true;
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&GameRules) }
