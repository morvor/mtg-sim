//! Shared helpers for rulings batch P002 (`r_p002_*.rs`): effects that turn a creature's
//! power or toughness into life, damage or counters (Scryfall Tagger "aikido"), prevention
//! and redirection, alternate win conditions, "sacrifice a creature or pay" additional
//! costs and the amount of mana spent to cast a spell. (The helpers of batch P003 are used
//! too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn pick(d: &Decision, needle: &str) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.contains(needle))
            .map(Answer::Index),
        _ => None,
    }
}

/// `p` answers every replacement-order choice (CR 616.1) with the option whose text
/// contains `needle` (the scripted answers otherwise).
pub fn prefer(t: &mut TestGame, p: PlayerId, needle: &'static str) {
    crate::r_s03_common::respond(t, p, PICKERS.iter().find(|(n, _)| *n == needle).unwrap().1);
}

type Responder = fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>;

/// The replacement-order preferences the tests of this batch use.
const PICKERS: &[(&str, Responder)] = &[
    ("Alhammarret", |_, d| pick(d, "Alhammarret")),
    ("Tainted Remedy", |_, d| pick(d, "Tainted Remedy")),
    ("New Way Forward", |_, d| pick(d, "New Way Forward")),
    ("Healing Salve", |_, d| pick(d, "Healing Salve")),
    ("Inkshield", |_, d| pick(d, "Inkshield")),
    ("controlled by P1", |_, d| pick(d, "controlled by P1")),
    ("controlled by P0", |_, d| pick(d, "controlled by P0")),
];

/// The replacement-order choices asked since decision `from`: (chooser, options).
pub fn replacement_choices(t: &TestGame, from: usize) -> Vec<(PlayerId, Vec<String>)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } => Some((*p, options.clone())),
            _ => None,
        })
        .collect()
}

/// The objects named `name` in exile.
pub fn exiled(t: &TestGame, name: &str) -> usize {
    t.g.find_in_zone(Zone::Exile, name).len()
}

/// Bolts the target (a Lightning Bolt cast by `p` and resolved).
pub fn bolt(t: &mut TestGame, p: PlayerId, target: Entity) {
    crate::r_p003_common::cast_resolve(t, p, "Lightning Bolt", "R", &[target]);
}
