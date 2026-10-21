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
//!
//! Events during the untap step (CR 502.3): "Whenever you untap one or more permanents
//! during your untap step" (The Millennium Calendar), "Whenever ~ becomes untapped during
//! your untap step" (The Ninth Doctor). The abilities trigger during the untap step and
//! are put on the stack when a player next receives priority, in the upkeep (CR 502.4).

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
/// `Condition::Custom`: a spell or ability the ability's controller controls caused the
/// trigger event ("a spell or ability you control exiles one or more permanents").
pub const CAUSED_BY_YOURS: &str = "event cause:a spell or ability you control";
/// `Condition::Custom`: a spell or ability controlled by the trigger event's player caused
/// it ("causes its controller to shuffle their library").
pub const CAUSED_BY_EVENT_PLAYERS: &str = "event cause:its controller's";
/// `Condition::Custom`: it's the untap step of the ability's controller's turn
/// ("Whenever you untap one or more permanents during your untap step", CR 502.3).
pub const DURING_YOUR_UNTAP_STEP: &str = "event time:during your untap step";
/// `Filter::Custom`: an object the source's linked abilities noted, as it last existed
/// (the creature put onto the battlefield with ~, as it dies: CR 603.10a, 607.2a).
pub const NOTED_AS_IT_LAST_EXISTED: &str = "linked:noted, as it last existed";
/// `Filter::Custom`: the object (a spell cast or a land played) was a card exiled with
/// the source (CR 607.2a): "Whenever a player plays a card exiled with ~".
pub const WAS_EXILED_WITH_SOURCE: &str = "linked:was a card exiled with ~";
/// `TriggerCond::Custom`: "Whenever the final chapter ability of a Saga you control
/// triggers" (Historian's Boon; CR 714.2c: the chapter ability with the greatest chapter
/// number among the Saga's; CR 603.3b: it triggers as that ability is put on the stack).
pub const FINAL_CHAPTER_TRIGGERS: &str = "saga:final chapter ability of a saga you control triggers";
/// `TriggerCond::Custom`: "When ~ enters attached to a creature" (Infinite Reflection):
/// the source enters the battlefield attached to a creature, the trigger's other object
/// ("that creature", as it was attached then: Infinite Reflection's rulings).
pub const ENTERS_ATTACHED_TO_CREATURE: &str = "attach:~ enters attached to a creature";
/// `Condition::Custom`: the trigger event's player is the player chosen for the source
/// ("When the chosen player loses the game": even once they've left it, CR 800.4a).
pub const EVENT_PLAYER_IS_CHOSEN: &str = "chosen:the event's player is the chosen player";
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
            DURING_YOUR_UNTAP_STEP => Some(
                g.turn.step == crate::turn::Step::Untap && g.turn.active == ctx.controller,
            ),
            EVENT_PLAYER_IS_CHOSEN => Some(
                g.chosen_player_of_source(ctx).is_some()
                    && g.chosen_player_of_source(ctx) == ctx.event.as_ref().and_then(|e| e.player),
            ),
            CAUSED_BY_YOURS => Some(cause.is_some_and(|c| ctl(c) == ctx.controller)),
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

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: crate::types::PlayerId,
        ev: &crate::events::Event,
    ) -> Option<Vec<crate::object::EventInfo>> {
        if name == ENTERS_ATTACHED_TO_CREATURE {
            let crate::events::Event::ZoneChange {
                new,
                to: crate::object::Zone::Battlefield,
                ..
            } = ev
            else {
                return Some(vec![]);
            };
            if *new != _src {
                return Some(vec![]);
            }
            let host = match g.obj(*new).attached_to {
                Some(crate::types::Entity::Object(o))
                    if g.obj(o).is(crate::types::CardType::Creature) =>
                {
                    o
                }
                _ => return Some(vec![]),
            };
            return Some(vec![crate::object::EventInfo {
                object: Some(*new),
                other: Some(host),
                player: Some(ctl),
                ..Default::default()
            }]);
        }
        if name != FINAL_CHAPTER_TRIGGERS {
            return None;
        }
        let crate::events::Event::AbilityTriggeredOnStack { ability, source } = ev else {
            return Some(vec![]);
        };
        let chapters = |t: &crate::ability::TriggerCond| -> Vec<u32> {
            match t {
                crate::ability::TriggerCond::Custom(n) => n
                    .strip_prefix("chapter:")
                    .map(|r| r.split(',').filter_map(|x| x.trim().parse().ok()).collect())
                    .unwrap_or_default(),
                _ => vec![],
            }
        };
        let saga = g.obj(*source);
        if saga.controller != ctl || !saga.chars.has_subtype("Saga") {
            return Some(vec![]);
        }
        let final_chapter = saga
            .chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                crate::ability::AbilityKind::Triggered(t) => chapters(&t.trigger).into_iter().max(),
                _ => None,
            })
            .max();
        let si = g.obj(*ability).stack.as_deref();
        let this_chapters = si.and_then(|si| match &si.kind {
            crate::object::StackKind::Triggered { ability: a, .. } => match &a.kind {
                crate::ability::AbilityKind::Triggered(t) => Some(chapters(&t.trigger)),
                _ => None,
            },
            _ => None,
        });
        let ok = match (final_chapter, this_chapters) {
            (Some(f), Some(v)) => v.contains(&f),
            _ => false,
        };
        Some(if ok {
            vec![crate::object::EventInfo {
                object: Some(*source),
                spell: Some(*ability),
                player: Some(ctl),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        match name {
            THE_CAUSE => Some(g.resolving == Some(id)),
            WAS_EXILED_WITH_SOURCE => Some(
                _ctx.source.zip(g.obj(id).prev).is_some_and(|(s, p)| {
                    g.obj(p).zone == crate::object::Zone::Exile
                        && g.obj(s).linked.values().any(|v| v.contains(&p))
                }),
            ),
            NOTED_AS_IT_LAST_EXISTED => Some(
                crate::linked_notes::noted(g, _ctx).contains(&crate::types::Entity::Object(id)),
            ),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&TriggerEvents) }
