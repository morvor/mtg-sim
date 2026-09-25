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
        // CR 702.171b: the saddled designation.
        "saddled" => g.obj(id).saddled,
        _ => false,
    }
}

fn cast_info<'a>(g: &'a Game, ctx: &'a Ctx) -> Option<&'a CastInfo> {
    g.cast_info(ctx)
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
        // "the total number of cards in all players' hands"
        "cards_in_all_hands" => g
            .players_in_game()
            .into_iter()
            .map(|p| g.player(p).hand.len() as i64)
            .sum(),
        // "the total life lost by your opponents this turn"
        "life_lost_by_opponents_this_turn" => g
            .history
            .life_lost
            .iter()
            .filter(|(p, _)| g.are_opponents(ctx.controller, **p))
            .map(|(_, n)| *n as i64)
            .sum(),
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
    let you = ctx.controller;
    let h = &g.history;
    // "you've cast another red spell this turn": a spell of that color (as it was on the
    // stack) other than the source.
    if let Some(c) = name.strip_prefix("you_cast_another_spell_this_turn:") {
        let Some(color) = c.chars().next().and_then(Color::from_letter) else {
            return false;
        };
        return g.history.spells_cast.iter().any(|(p, o)| {
            *p == ctx.controller && Some(*o) != ctx.source && g.obj(*o).chars.colors.contains(color)
        });
    }
    match name {
        // "if you attacked this turn" (raid): you declared one or more attackers this turn;
        // only the active player declares attackers (CR 508.1).
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
    // "damage prevented:self": "Whenever damage that would be dealt to this is prevented"
    // (CR 615.13), once per prevention effect applied to simultaneous damage.
    if name == "damage prevented:self" {
        if let Event::DamagePrevented {
            source,
            target: Entity::Object(o),
            amount,
            ..
        } = ev
        {
            if *o == src {
                return vec![EventInfo {
                    object: Some(src),
                    other: Some(*source),
                    amount: *amount as i32,
                    ..Default::default()
                }];
            }
        }
        return vec![];
    }
    vec![]
}

pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) {
    // "named-token:N:Name": create N tokens by name (CR 111.11).
    if let Some(spec) = name.strip_prefix("named-token:") {
        crate::tokens::create_named_tokens(g, spec, ctx);
        return;
    }
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
