//! Helpers shared by the mtg-api tests.
#![allow(dead_code)]

use mtg_engine::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, OnceLock};

/// Nonland cards for random decks: fully supported, mana value at most 6, with their
/// colour identity as a WUBRG mask.
fn pool() -> &'static [(String, u8)] {
    static POOL: OnceLock<Vec<(String, u8)>> = OnceLock::new();
    POOL.get_or_init(|| {
        let mut v = Vec::new();
        for c in mtg_data::cards().iter() {
            if !c.is_playable_card() || !c.is_legal_somewhere() {
                continue;
            }
            if !c.games.is_empty() && c.games.iter().all(|g| g != "paper") {
                continue;
            }
            let front = c.type_line.as_deref().unwrap_or("");
            if front.is_empty() || front.contains("Land") || c.cmc.unwrap_or(0.0) > 6.0 {
                continue;
            }
            let mask = c.color_identity.iter().fold(0u8, |m, s| {
                m | match s.as_str() {
                    "W" => 1,
                    "U" => 2,
                    "B" => 4,
                    "R" => 8,
                    "G" => 16,
                    _ => 0,
                }
            });
            v.push((c.name.clone(), mask));
        }
        v
    })
}

/// A random 40-card deck: one or two colours, 17 basic lands, 23 spells.
pub fn random_deck(seed: u64) -> Vec<Arc<CardDef>> {
    const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut colors: Vec<usize> = (0..5).collect();
    colors.shuffle(&mut rng);
    colors.truncate(rng.gen_range(1..=2));
    let mask = colors.iter().fold(0u8, |m, &c| m | 1 << c);
    let candidates: Vec<&str> = pool()
        .iter()
        .filter(|(_, ci)| ci & !mask == 0)
        .map(|(n, _)| n.as_str())
        .collect();
    let db = CardDb::global();
    let mut deck: Vec<Arc<CardDef>> = (0..17)
        .map(|i| db.get(BASICS[colors[i % colors.len()]]).expect("basic"))
        .collect();
    let mut spells: Vec<Arc<CardDef>> = Vec::new();
    for _ in 0..2000 {
        if spells.len() == 12 {
            break;
        }
        let name = *candidates.choose(&mut rng).expect("cards");
        if spells.iter().any(|d| d.name.eq_ignore_ascii_case(name)) {
            continue;
        }
        if let Some(d) = db.get(name).filter(|d| d.is_fully_supported()) {
            spells.push(d);
        }
    }
    for i in 0..23 {
        if let Some(s) = spells.get(i % spells.len().max(1)) {
            deck.push(s.clone());
        }
    }
    deck
}

/// A simple two-colour deck of well-known cards.
pub fn simple_deck() -> Vec<Arc<CardDef>> {
    let db = CardDb::global();
    let mut v = Vec::new();
    for (n, name) in [
        (9, "Mountain"),
        (8, "Forest"),
        (3, "Grizzly Bears"),
        (3, "Lightning Bolt"),
        (2, "Hill Giant"),
        (2, "Shock"),
        (3, "Llanowar Elves"),
        (2, "Giant Growth"),
        (2, "Craw Wurm"),
        (2, "Raging Goblin"),
        (2, "Prodigal Pyromancer"),
        (2, "Rampant Growth"),
    ] {
        for _ in 0..n {
            v.push(db.get(name).expect(name));
        }
    }
    v
}
