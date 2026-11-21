//! Shared helpers for the CR 900–905 tests (casual variants).

#![allow(dead_code)]

pub use crate::r107_planechase::{add_planar_deck, face_up_names, force_next_roll, planechase_game, roll};
use crate::r703_common::oracle_card;
use mtg_engine::card::CardDef;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use std::sync::Arc;

/// A custom plane card with abilities compiled from `text`.
pub fn plane(name: &str, text: &str) -> CardDef {
    oracle_card(name, "Plane — Test Realm", "", None, text)
}

/// Puts a custom planar card at the bottom of `owner`'s planar deck (face down in the
/// command zone).
pub fn add_custom_planar(t: &mut TestGame, owner: PlayerId, def: CardDef) -> ObjectId {
    let id = t.custom(owner, def, Zone::Command);
    t.g.objects[id.0 as usize].face_down = true;
    t.g.recompute();
    id
}

/// Rolls the planar die because of an effect (not the special action), showing `face`.
pub fn roll_effect(t: &mut TestGame, p: PlayerId, face: PlanarFace) {
    force_next_roll(t, face);
    planechase::roll_planar_die(&mut t.g, p);
    t.settle();
}

/// How many times a player planeswalked this turn (planeswalk events).
pub fn planeswalks(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name.as_str() == planechase::PLANESWALKED))
        .count()
}

/// Whether chaos ensued this turn.
pub fn chaos_count(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name.as_str() == planechase::CHAOS_ENSUES))
        .count()
}

/// The name of the card an object represents (even face down).
pub fn name_of(t: &TestGame, id: ObjectId) -> String {
    let o = t.g.obj(id);
    o.card
        .as_ref()
        .map_or_else(|| o.chars.name.to_string(), |c| c.name.to_string())
}

/// Names of a list of card definitions' cards, as `Arc`s.
pub fn cards(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| mtg_engine::card::card(n)).collect()
}

/// Ten different plane cards.
pub const TEN_PLANES: [&str; 10] = [
    "Krosa",
    "Goldmeadow",
    "The Fourth Sphere",
    "Panopticon",
    "Tazeem",
    "Naar Isle",
    "Lethe Lake",
    "Llanowar",
    "Stronghold Furnace",
    "The Eon Fog",
];

/// Four different phenomenon cards.
pub const PHENOMENA: [&str; 4] = [
    "Mutual Epiphany",
    "Planewide Disaster",
    "Chaotic Aether",
    "Spatial Merging",
];

/// A card with just a name, for booster packs.
pub fn named(name: &str) -> Arc<CardDef> {
    Arc::new(CardDef::custom(mtg_engine::object::Characteristics {
        name: smol_str::SmolStr::new(name),
        rules_text: Arc::from(""),
        ..Default::default()
    }))
}

/// Booster packs for a draft: player `p` opens pack `r` in round `r + 1`; each pack has
/// `size` cards named "P{p}R{r}C{k}".
pub fn boosters(players: usize, rounds: usize, size: usize) -> Vec<Vec<Vec<Arc<CardDef>>>> {
    (0..players)
        .map(|p| {
            (0..rounds)
                .map(|r| (0..size).map(|k| named(&format!("P{p}R{r}C{k}"))).collect())
                .collect()
        })
        .collect()
}

/// The names of cards.
pub fn names_of(cards: &[Arc<CardDef>]) -> Vec<String> {
    cards.iter().map(|c| c.name.to_string()).collect()
}
