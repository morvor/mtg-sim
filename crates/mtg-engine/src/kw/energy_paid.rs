//! `Value::Custom` [`ENERGY_PAID_OR_LOST_THIS_TURN`]: "the {E} you've paid or lost this
//! turn" (Blaster Hulk: "~ costs {1} less to cast for each {E} you've paid or lost this
//! turn"; Izzet Generatorium: "Activate only if you've paid or lost four or more {E} this
//! turn"). Paying {E} is losing that many energy counters (CR 107.14), so it's the total
//! of the energy counters removed from the player this turn, however they were removed
//! (this turn's events, as they happened: energy paid while casting the spell being cast
//! has been paid before its total cost is determined only if paid earlier, CR 601.2f).

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::types::{counters, Entity};

/// The `Value::Custom` name for the energy the controller has paid or lost this turn.
pub const ENERGY_PAID_OR_LOST_THIS_TURN: &str = "energy you've paid or lost this turn";

/// How many energy counters `p` has paid or lost this turn.
pub fn energy_paid_or_lost(g: &Game, p: crate::types::PlayerId) -> u32 {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .map(|e| match e {
            Event::CountersRemoved {
                target: Entity::Player(q),
                kind,
                n,
                ..
            } if *q == p && kind.as_str() == counters::ENERGY => *n,
            _ => 0,
        })
        .sum()
}

pub struct EnergyPaid;

impl super::KeywordRules for EnergyPaid {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == ENERGY_PAID_OR_LOST_THIS_TURN)
            .then(|| energy_paid_or_lost(g, ctx.controller) as i64)
    }
}

inventory::submit! { super::KeywordRegistration(&EnergyPaid) }
