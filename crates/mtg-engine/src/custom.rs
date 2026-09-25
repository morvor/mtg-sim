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

/// How the spell (or, for its abilities, the permanent) was cast: the resolving spell's
/// cast info, or the source permanent's.
fn cast_info<'a>(g: &'a Game, ctx: &'a Ctx) -> Option<&'a CastInfo> {
    match ctx.cast.as_ref() {
        Some(c) if c.was_cast => Some(c),
        _ => ctx
            .source
            .and_then(|s| g.obj(s).cast.as_deref())
            .or(ctx.cast.as_ref()),
    }
}

pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> i64 {
    let _ = (g, ctx);
    // "mana_spent_of:U": amount of mana of one type spent to cast it (adamant).
    if let Some(t) = name.strip_prefix("mana_spent_of:") {
        let Some(t) = t
            .chars()
            .next()
            .and_then(crate::mana::ManaType::from_letter)
        else {
            return 0;
        };
        return cast_info(g, ctx).map_or(0, |c| {
            c.mana_spent.iter().filter(|m| **m == t).count() as i64
        });
    }
    match name {
        // "at least three mana of the same color was spent to cast it" (adamant).
        "max_mana_spent_of_one_color" => cast_info(g, ctx).map_or(0, |c| {
            [
                crate::mana::ManaType::W,
                crate::mana::ManaType::U,
                crate::mana::ManaType::B,
                crate::mana::ManaType::R,
                crate::mana::ManaType::G,
            ]
            .iter()
            .map(|t| c.mana_spent.iter().filter(|m| *m == t).count() as i64)
            .max()
            .unwrap_or(0)
        }),
        // Creatures that died under the controller's control this turn.
        "creatures_you_controlled_died_this_turn" => g
            .history
            .creatures_died
            .iter()
            .filter(|o| g.obj(**o).controller == ctx.controller)
            .count() as i64,
        // Number of spells the controller has cast this turn.
        "spells_you_cast_this_turn" => g
            .history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == ctx.controller)
            .count() as i64,
        _ => 0,
    }
}

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> bool {
    let _ = (g, ctx);
    match name {
        // "you attacked this turn" (raid): you declared one or more attackers this turn
        // (CR 508.1).
        "you_attacked_this_turn" => g
            .history
            .attackers
            .iter()
            .any(|a| g.obj(*a).controller == ctx.controller),
        // "a permanent left the battlefield under your control this turn" (revolt).
        "permanent_left_under_your_control_this_turn" => g
            .history
            .permanents_left
            .iter()
            .any(|o| g.obj(*o).controller == ctx.controller),
        // "you were the starting player" (CR 103.1).
        "you_were_the_starting_player" => g.turn.starting_player == ctx.controller,
        // "an opponent lost life this turn".
        "opponent_lost_life_this_turn" => g
            .history
            .life_lost
            .iter()
            .any(|(p, n)| *n > 0 && g.are_opponents(ctx.controller, *p)),
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
