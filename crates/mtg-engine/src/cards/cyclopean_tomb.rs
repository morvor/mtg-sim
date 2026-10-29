//! Cyclopean Tomb: "When ~ is put into a graveyard from the battlefield, at the beginning
//! of each of your upkeeps for the rest of the game, remove all mire counters from a land
//! that a mire counter was put onto with ~ but that a mire counter has not been removed
//! from with ~." Its first ability ("{2}, {T}: Put a mire counter on target non-Swamp
//! land. ...") is compiled; this one is hand-written: the Tomb keeps track of the lands
//! its abilities put mire counters on, and each upkeep its delayed triggered ability
//! (CR 603.7, lasting for the rest of the game) removes all mire counters from one of the
//! lands it put a counter on and hasn't removed counters from yet (its controller chooses
//! which; it doesn't matter where the counters it removes came from, ruling). A land that
//! has left the battlefield is a new object the Tomb never put a counter on (CR 400.7).

use super::{delayed_for_rest_of_game, map_effect, parse, ManualAbility};
use crate::ability::{Ability, AbilityKind, Body, Effect, PlayerRel, TriggerCond, TriggerStep};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{StackKind, Zone};
use crate::types::{Entity, ObjectId};

const TEXT: &str = "When ~ is put into a graveyard from the battlefield, at the beginning of each of your upkeeps for the rest of the game, remove all mire counters from a land that a mire counter was put onto with ~ but that a mire counter has not been removed from with ~.";
const CREATE: &str = "card:Cyclopean Tomb:at the beginning of each of your upkeeps for the rest of the game";
const REMOVE: &str = "card:Cyclopean Tomb:remove all mire counters from a land it put one onto";
/// Rows `[tomb, land]`: lands a mire counter was put onto with the Tomb and that a mire
/// counter hasn't been removed from with it.
const PUT: &str = "card:Cyclopean Tomb:lands a mire counter was put onto";

inventory::submit! { ManualAbility {
    card: "Cyclopean Tomb",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ is put into a graveyard from the battlefield, you gain 1 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(CREATE.into())))
            .collect()
    },
    reason: "tracks the lands its own ability put mire counters on and later removes them: only this card",
} }

/// Whether `a` is this triggered ability.
fn is_this(a: &Ability) -> bool {
    matches!(&a.kind, AbilityKind::Triggered(t) if matches!(&t.body.effect, Effect::Custom(n) if n == CREATE))
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// Mire counters put on a land by an ability of a Tomb (its first ability resolving).
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::CountersAdded {
            target: Entity::Object(land),
            kind,
            ..
        } = ev
        else {
            return;
        };
        if kind != "mire" {
            return;
        }
        let Some(source) = g.resolving.and_then(|r| match g.try_obj(r)?.stack.as_deref()?.kind {
            StackKind::Activated { source, .. } => Some(source),
            _ => None,
        }) else {
            return;
        };
        if !g.try_obj(source).is_some_and(|o| o.chars.abilities.iter().any(is_this)) {
            return;
        }
        let row = vec![source.0 as i64, land.0 as i64];
        if !g.cards.get(PUT).contains(&row) {
            g.cards.push(PUT, row);
        }
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            CREATE => {
                delayed_for_rest_of_game(
                    g,
                    ctx,
                    TriggerCond::BeginningOf {
                        step: TriggerStep::Upkeep,
                        whose: PlayerRel::You,
                    },
                    Body::simple(vec![], Effect::Custom(REMOVE.into())),
                );
                true
            }
            REMOVE => {
                let Some(tomb) = ctx.source else {
                    return true;
                };
                let lands: Vec<ObjectId> = g
                    .cards
                    .get(PUT)
                    .iter()
                    .filter(|r| r[0] == tomb.0 as i64)
                    .map(|r| ObjectId(r[1] as u32))
                    .filter(|l| g.is_live(*l) && g.obj(*l).zone == Zone::Battlefield)
                    .collect();
                if lands.is_empty() {
                    return true;
                }
                // One that has mire counters, if any does (removing none removes none
                // "with ~").
                let with: Vec<ObjectId> = lands
                    .iter()
                    .copied()
                    .filter(|l| g.obj(*l).counter("mire") > 0)
                    .collect();
                let lands = if with.is_empty() { lands } else { with };
                let land = if lands.len() == 1 {
                    lands[0]
                } else {
                    g.ask_objects(
                        ctx.controller,
                        Some(tomb),
                        "Remove all mire counters from which land?",
                        lands.clone(),
                        1,
                        1,
                    )
                    .first()
                    .copied()
                    .filter(|l| lands.contains(l))
                    .unwrap_or(lands[0])
                };
                let n = g.obj(land).counter("mire");
                if n > 0 {
                    g.remove_counters_by(Entity::Object(land), "mire", n, Some(ctx.controller));
                    let row = vec![tomb.0 as i64, land.0 as i64];
                    g.cards.retain(PUT, |r| *r != row);
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
