//! CR 702.57 Forecast. A forecast ability is an activated ability that can be activated
//! only from a player's hand, written "Forecast — [Activated ability]" (CR 702.57a). It
//! may be activated only during its owner's upkeep step and only once each turn; its
//! controller reveals the card as it's activated and plays with it revealed in their hand
//! until it leaves the hand or a step or phase that isn't an upkeep step begins
//! (CR 702.57b).
//!
//! The oracle compiler turns "Forecast — [cost], Reveal ~ from your hand: [effect]" into
//! an activated ability of the hand whose text starts with "Forecast", with the timing
//! restrictions above and [`REVEAL`] as part of its cost (see
//! `oracle/patterns/k702_052_066.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::turn::Step;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` (part of the cost): reveal the card with the forecast ability.
pub const REVEAL: &str = "forecast:reveal this card from your hand";

/// `Event::Custom` name: a player revealed a card.
pub const REVEALED: &str = "revealed";

/// Builds a forecast ability from its cost (without the reveal) and effect.
pub fn forecast_ability(mut cost: Cost, body: Body) -> ActivatedAbility {
    cost.parts
        .push(CostPart::Effect(Box::new(Effect::Custom(SmolStr::new(REVEAL)))));
    let mut act = ActivatedAbility::new(cost, body);
    act.zone = FunctionZone::Hand;
    act.timing = ActivationTiming::YourUpkeep;
    act.max_per_turn = Some(1);
    act
}

/// Whether `a` is a forecast ability.
pub fn is_forecast(a: &AbilityDef) -> bool {
    matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with("Forecast")
}

/// Whether `card` is revealed because a forecast ability of it was activated (CR 702.57b):
/// it's still in its owner's hand (the same object) during the upkeep step in which the
/// ability was activated.
pub fn is_revealed(g: &Game, card: ObjectId) -> bool {
    let o = g.obj(card);
    g.is_live(card)
        && matches!(o.zone, Zone::Hand(_))
        && g.turn.step == Step::Upkeep
        && o.chars.abilities.iter().any(|a| {
            is_forecast(a) && o.activations_this_turn.get(&a.uid).copied().unwrap_or(0) > 0
        })
}

pub struct Forecast;

impl KeywordRules for Forecast {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Forecast]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REVEAL {
            return false;
        }
        if let Some(card) = ctx.source {
            let p = ctx.controller;
            g.log(|g| format!("{p} reveals {}", g.describe(card)));
            g.emit(Event::Custom {
                name: REVEALED.into(),
                player: Some(p),
                obj: Some(card),
                amount: 0,
            });
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Forecast) }
