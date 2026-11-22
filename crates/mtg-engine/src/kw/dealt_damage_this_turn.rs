//! Conditions about the damage an ability's source dealt this turn (CR 120): "if this
//! creature dealt damage to an opponent this turn" (Dunerider Outlaw, Whirling Dervish),
//! "if this creature dealt damage to a player this turn".
//!
//! * The source is the object itself: a permanent that changed zones is a new object that
//!   hasn't dealt that damage (CR 400.7).
//! * Damage that was dealt stays dealt: a player who was dealt damage by it and has left
//!   the game since still counts (the condition is about what happened).
//! * "An opponent" is judged as the condition is checked, relative to the ability's
//!   controller: any other player not on their team (CR 102.2, 102.3), including one who
//!   has left the game.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Condition::Custom`: the ability's source dealt damage to an opponent this turn.
pub const DEALT_DAMAGE_TO_OPPONENT: &str = "turn:this dealt damage to an opponent";
/// `Condition::Custom`: the ability's source dealt damage to a player this turn.
pub const DEALT_DAMAGE_TO_PLAYER: &str = "turn:this dealt damage to a player";

/// Whether `source` dealt damage this turn to a player matching `to`.
fn dealt_damage_to(g: &Game, source: ObjectId, to: impl Fn(PlayerId) -> bool) -> bool {
    g.turn_events.iter().any(|e| {
        matches!(e, Event::Damage { source: s, target: Entity::Player(q), amount, .. }
            if *s == source && *amount > 0 && to(*q))
    })
}

pub struct DealtDamageThisTurn;

impl KeywordRules for DealtDamageThisTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let me = ctx.controller;
        let to_opponent = match name {
            DEALT_DAMAGE_TO_OPPONENT => true,
            DEALT_DAMAGE_TO_PLAYER => false,
            _ => return None,
        };
        let Some(source) = ctx.source else {
            return Some(false);
        };
        Some(dealt_damage_to(g, source, |q| {
            // Whether or not they're still in the game.
            !to_opponent || g.are_opponents(me, q)
        }))
    }
}

inventory::submit! { KeywordRegistration(&DealtDamageThisTurn) }
