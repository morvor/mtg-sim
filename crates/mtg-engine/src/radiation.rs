//! Rad counters (CR 728): life lost "from radiation" (CR 728.1a) is life lost as a result
//! of the inherent triggered ability associated with rad counters (see
//! `counter_rules::rad_trigger`). Effects can change it ("You gain life rather than lose
//! life from radiation").

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` name: the player of the triggering event loses 1 life from radiation.
pub const LOSE_LIFE_FROM_RADIATION: &str = "radiation: lose 1 life";
/// `StaticEffect::Custom` name: "You gain life rather than lose life from radiation."
pub const GAIN_RATHER_THAN_LOSE: &str = "radiation: gain life rather than lose life";

/// The instruction "that player loses 1 life" of the rad counter ability (CR 728.1).
pub fn lose_life_from_radiation() -> Effect {
    Effect::Custom(SmolStr::new(LOSE_LIFE_FROM_RADIATION))
}

/// Whether `p` gains life rather than loses life from radiation.
pub fn gains_instead(g: &Game, p: PlayerId) -> bool {
    g.statics
        .customs
        .iter()
        .any(|(_, c, n)| *c == p && n.as_str() == GAIN_RATHER_THAN_LOSE)
}

/// `p` loses `n` life from radiation (CR 728.1a).
pub fn radiation_life_loss(g: &mut Game, p: PlayerId, n: u32) {
    if g.dirty {
        g.recompute();
    }
    if gains_instead(g, p) {
        g.gain_life(p, n);
    } else {
        g.lose_life(p, n);
    }
}

struct Radiation;

impl KeywordRules for Radiation {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != LOSE_LIFE_FROM_RADIATION {
            return false;
        }
        let p = ctx
            .event
            .as_ref()
            .and_then(|e| e.player)
            .unwrap_or(ctx.controller);
        radiation_life_loss(g, p, 1);
        true
    }
}

inventory::submit! { KeywordRegistration(&Radiation) }
