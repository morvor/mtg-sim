//! Shared helpers for the tests of rulings batch P186 (`r_p186_*.rs`): typal Vampire,
//! Warrior, Wizard and Zombie cards, permanents that become creatures, and Phyrexians.

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The number of `NameCard` decisions asked so far.
pub fn names_asked(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::NameCard { .. }))
        .count()
}

/// The number of target choices asked of `p` since decision `from`.
pub fn target_asks(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| {
            *q == p && matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. })
        })
        .count()
}

/// The number of creatures on the battlefield controlled by `p`.
pub fn creature_count(g: &mtg_engine::game::Game, p: PlayerId) -> usize {
    g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Creature))
        .count()
}

/// Whether an object entered the battlefield tapped this turn as the object `new`.
pub fn entered_tapped(t: &TestGame, card_name: &str) -> bool {
    t.g.turn_events.iter().any(|e| match e {
        Event::ZoneChange { new, to, .. } if *to == mtg_engine::object::Zone::Battlefield => {
            t.g.obj(*new).chars.name.as_str() == card_name && t.g.obj(*new).tapped
        }
        _ => false,
    })
}

/// The number of times an object named `card_name` entered the battlefield this turn.
pub fn entries_of(t: &TestGame, card_name: &str) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| match e {
            Event::ZoneChange { new, to, .. } if *to == mtg_engine::object::Zone::Battlefield => {
                t.g.obj(*new).chars.name.as_str() == card_name
            }
            _ => false,
        })
        .count()
}
