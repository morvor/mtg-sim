//! Rules behind `oracle/patterns/durations_4.rs`: "During that turn, power-up abilities
//! can't be activated." (Kang the Conqueror, CR 702.193): a rule effect for the turn that
//! stops power-up abilities from being activated (CR 602.5).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Restriction::Custom` of a rule effect: power-up abilities can't be activated.
pub const POWER_UP_CANT_BE_ACTIVATED: &str = "power-up:abilities can't be activated";

struct Durations4;

impl KeywordRules for Durations4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn activation_allowed(&self, g: &Game, _p: PlayerId, _src: ObjectId, a: &Ability) -> bool {
        if !crate::kw::power_up::is_power_up(a) {
            return true;
        }
        let blocked = |r: &Restriction| {
            matches!(r, Restriction::Custom(n) if n.as_str() == POWER_UP_CANT_BE_ACTIVATED)
        };
        !(g.rule_effects.iter().any(|e| blocked(&e.restriction))
            || g.statics.restrictions.iter().any(|(_, _, r)| blocked(r)))
    }
}

inventory::submit! { KeywordRegistration(&Durations4) }
