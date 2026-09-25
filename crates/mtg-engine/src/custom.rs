//! Named custom predicates, values, conditions, triggers, and effects — escape hatches
//! for behavior that doesn't fit the ability language. Referenced from the AST by name
//! (e.g. `Filter::Custom("...")`, `TriggerCond::Custom("chapter:1")`).

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> bool {
    let _ = (g, id, ctx);
    match name {
        _ => false,
    }
}

pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> i64 {
    let _ = (g, ctx);
    match name {
        _ => 0,
    }
}

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> bool {
    let _ = (g, ctx);
    let you = ctx.controller;
    let h = &g.history;
    match name {
        // "if you attacked this turn": only the active player declares attackers (CR 508.1).
        "you_attacked_this_turn" => g.turn.active == you && !h.attackers.is_empty(),
        // "if a permanent you controlled left the battlefield this turn" (last known
        // information of the permanents that left).
        "permanent_you_controlled_left_this_turn" => h
            .permanents_left
            .iter()
            .any(|o| g.obj(*o).controller == you),
        "creature_died_under_your_control_this_turn" => {
            h.creatures_died.iter().any(|o| g.obj(*o).controller == you)
        }
        "you_descended_this_turn" => h.descended.get(&you).is_some_and(|n| *n > 0),
        "card_left_your_graveyard_this_turn" => {
            h.cards_left_graveyard.get(&you).is_some_and(|n| *n > 0)
        }
        "you_cast_noncreature_spell_this_turn" => h
            .spells_cast
            .iter()
            .any(|(p, s)| *p == you && !g.obj(*s).chars.card_types.contains(CardType::Creature)),
        // Werewolves (the previous turn's history).
        "no_spells_cast_last_turn" => g.last_turn_history.spells_cast.is_empty(),
        "a_player_cast_two_spells_last_turn" => {
            let spells = &g.last_turn_history.spells_cast;
            spells
                .iter()
                .any(|(p, _)| spells.iter().filter(|(q, _)| q == p).count() >= 2)
        }
        "you_lost_life_last_turn" => g
            .last_turn_history
            .life_lost
            .get(&you)
            .is_some_and(|n| *n > 0),
        _ => false,
    }
}

/// Custom triggers. `chapter:N[,M]` implements Saga chapter abilities (CR 714.2c):
/// triggers when lore counters are put on the source, bringing the count from below N
/// to N or more.
pub fn custom_trigger(
    g: &Game,
    name: &str,
    src: ObjectId,
    ctl: PlayerId,
    ev: &Event,
) -> Vec<EventInfo> {
    let _ = ctl;
    if let Some(ns) = name.strip_prefix("chapter:") {
        if let Event::CountersAdded {
            target: Entity::Object(o),
            kind,
            n,
        } = ev
        {
            if *o == src && kind.as_str() == counters::LORE {
                let after = g.obj(src).counter(counters::LORE);
                let before = after.saturating_sub(*n);
                let mut out = Vec::new();
                for part in ns.split(',') {
                    if let Ok(k) = part.trim().parse::<u32>() {
                        if before < k && after >= k {
                            out.push(EventInfo {
                                object: Some(src),
                                amount: k as i32,
                                ..Default::default()
                            });
                        }
                    }
                }
                return out;
            }
        }
        return vec![];
    }
    vec![]
}

pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) {
    let _ = (g, ctx);
    match name {
        _ => {}
    }
}

/// "can't have more than N [kind] counters on it" (CR 704.5r).
pub fn counter_limits(g: &Game, id: ObjectId) -> Vec<(CounterKind, u32)> {
    let _ = (g, id);
    vec![]
}
