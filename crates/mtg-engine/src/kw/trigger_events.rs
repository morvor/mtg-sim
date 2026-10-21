//! Rules behind the trigger events of `oracle/patterns/trigger_grammar_4.rs`.
//!
//! Events caused by a spell or ability (CR 603.2, 608.2): "Whenever a spell or ability
//! causes a player to shuffle their library" (Psychogenic Probe), "causes its controller
//! to shuffle their library" (Widespread Panic), "Whenever a spell or ability an opponent
//! controls causes a land to be put into your graveyard from the battlefield" (Sacred
//! Ground), "Whenever a white instant or sorcery spell causes you to gain life" (Firesong
//! and Sunspeaker). The cause of an event is the spell or ability whose resolution
//! performed it ([`Game::resolving`]): an event a game rule, a cost or a turn-based action
//! performs (a mulligan's shuffle, a creature dying to lethal damage) isn't caused by a
//! spell or ability, and a triggered ability of a spell card (Renewed Faith's cycling
//! trigger) isn't that spell.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Condition::Custom`: a spell or ability caused the trigger event.
pub const CAUSED_BY_SPELL_OR_ABILITY: &str = "event cause:a spell or ability";
/// `Condition::Custom`: a spell or ability an opponent of the ability's controller controls
/// caused the trigger event.
pub const CAUSED_BY_OPPONENTS: &str = "event cause:a spell or ability an opponent controls";
/// `Condition::Custom`: a spell or ability controlled by the trigger event's player caused
/// it ("causes its controller to shuffle their library").
pub const CAUSED_BY_EVENT_PLAYERS: &str = "event cause:its controller's";
/// `Filter::Custom`: the spell or ability that caused the trigger event.
pub const THE_CAUSE: &str = "event cause:the spell or ability";

struct TriggerEvents;

impl KeywordRules for TriggerEvents {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let cause = g.resolving;
        let ctl = |id: ObjectId| g.obj(id).controller;
        match name {
            CAUSED_BY_SPELL_OR_ABILITY => Some(cause.is_some()),
            CAUSED_BY_OPPONENTS => {
                Some(cause.is_some_and(|c| g.are_opponents(ctx.controller, ctl(c))))
            }
            CAUSED_BY_EVENT_PLAYERS => Some(cause.is_some_and(|c| {
                ctx.event
                    .as_ref()
                    .and_then(|e| e.player)
                    .is_some_and(|p| p == ctl(c))
            })),
            _ => None,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == THE_CAUSE).then(|| g.resolving == Some(id))
    }
}

inventory::submit! { KeywordRegistration(&TriggerEvents) }
