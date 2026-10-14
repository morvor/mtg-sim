//! Pack tactics (an ability word): "Whenever this creature attacks, if you attacked with
//! creatures with total power 6 or greater this combat, ...". "You attacked with" refers
//! to the declaration of attackers (CR 508.1): the total is the power the creatures you
//! declared as attackers had as they were declared (`CombatState::declared_attack_power`).
//! Changing their power afterwards, or removing them from combat, doesn't change it, and
//! creatures put onto the battlefield attacking weren't declared, so they don't count. The
//! intervening "if" (CR 603.4) reads the same total as the ability triggers and resolves.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Value::Custom`: the total power of the creatures the ability's controller declared as
/// attackers this combat, as they were declared.
pub const ATTACKED_WITH_TOTAL_POWER: &str = "pack tactics:total power you attacked with";

pub struct PackTactics;

impl KeywordRules for PackTactics {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != ATTACKED_WITH_TOTAL_POWER {
            return None;
        }
        let Some(c) = g.combat.as_ref() else {
            return Some(0);
        };
        Some(
            c.declared_attack_power
                .iter()
                .filter(|(id, _)| g.obj(*id).controller == ctx.controller)
                .map(|(_, p)| *p as i64)
                .sum(),
        )
    }
}

inventory::submit! { KeywordRegistration(&PackTactics) }
