//! "[object] that was dealt noncombat damage this turn" (Grisly Sigil: "If it was dealt
//! noncombat damage this turn, ..."): damage dealt other than combat damage (CR 510.2,
//! 120.2). Each object dealt noncombat damage is recorded as the damage is dealt
//! (`TurnHistory::objects_dealt_noncombat_damage`); a permanent that changed zones since is
//! a new object (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: an object dealt noncombat damage this turn.
pub const DEALT_NONCOMBAT_DAMAGE_THIS_TURN: &str = "turn:dealt noncombat damage this turn";

pub struct NoncombatDamage;

impl KeywordRules for NoncombatDamage {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn after_damage(&self, g: &mut Game, _source: ObjectId, target: Entity, amount: u32, combat: bool) {
        if let (Entity::Object(o), false) = (target, combat) {
            if amount > 0 {
                g.history.objects_dealt_noncombat_damage.insert(o);
            }
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name != DEALT_NONCOMBAT_DAMAGE_THIS_TURN {
            return None;
        }
        Some(g.history.objects_dealt_noncombat_damage.contains(&id))
    }
}

inventory::submit! { KeywordRegistration(&NoncombatDamage) }
