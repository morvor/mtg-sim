//! "that dealt damage to you this turn" (Reciprocate, Spear of Heliod, Retaliate): an
//! object that, as the same object (CR 400.7), dealt damage to the ability's controller
//! this turn. Damage dealt to players is recorded as it's dealt
//! (`TurnHistory::damage_to_players_by_source`); the qualifier is parsed in
//! `oracle/patterns/filters_relational.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: dealt damage to the controller of the ability this turn.
pub const DEALT_DAMAGE_TO_YOU_THIS_TURN: &str = "dealt damage to you this turn";

pub struct DealtDamageToYou;

impl KeywordRules for DealtDamageToYou {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn after_damage(&self, g: &mut Game, source: ObjectId, target: Entity, amount: u32, _: bool) {
        if let Entity::Player(p) = target {
            let record = (source, p);
            if amount > 0 && !g.history.damage_to_players_by_source.contains(&record) {
                g.history.damage_to_players_by_source.push(record);
            }
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == DEALT_DAMAGE_TO_YOU_THIS_TURN).then(|| {
            g.history
                .damage_to_players_by_source
                .contains(&(id, ctx.controller))
        })
    }
}

inventory::submit! { KeywordRegistration(&DealtDamageToYou) }
