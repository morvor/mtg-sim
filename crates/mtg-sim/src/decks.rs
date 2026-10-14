//! Deck lists: parsing "4 Lightning Bolt" files and generating random decks for fuzzing.

use mtg_engine::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::sync::{Arc, OnceLock};

/// A deck and its sideboard (cards outside the game, e.g. a companion, CR 103.2b).
#[derive(Clone)]
pub struct DeckList {
    pub main: Vec<Arc<CardDef>>,
    pub sideboard: Vec<Arc<CardDef>>,
}

impl DeckList {
    /// "3 Shock, 12 Mountain, ..." in first-seen order.
    pub fn summary(&self) -> String {
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for c in &self.main {
            match counts.iter_mut().find(|(n, _)| *n == &*c.name) {
                Some(e) => e.1 += 1,
                None => counts.push((&c.name, 1)),
            }
        }
        let mut s: Vec<String> = counts.iter().map(|(n, k)| format!("{k} {n}")).collect();
        if !self.sideboard.is_empty() {
            s.push(format!(
                "sideboard: {}",
                self.sideboard
                    .iter()
                    .map(|c| &*c.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        s.join(", ")
    }
}

pub fn load_deck(path: &str) -> DeckList {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("can't read {path}: {e}"));
    parse_decklist(&text)
}

/// Parses "4 Lightning Bolt" style lines; cards after a "Sideboard" line (or a
/// "Companion" line) are the sideboard. Comments start with `#` or `//`.
pub fn parse_decklist(text: &str) -> DeckList {
    let mut main = Vec::new();
    let mut sideboard = Vec::new();
    let mut in_sideboard = false;
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') || l.starts_with("//") {
            continue;
        }
        if l.eq_ignore_ascii_case("sideboard") || l.eq_ignore_ascii_case("companion") {
            in_sideboard = true;
            continue;
        }
        let out = if in_sideboard {
            &mut sideboard
        } else {
            &mut main
        };
        let (n, name) = match l.split_once(' ') {
            Some((n, rest)) if n.trim_end_matches('x').parse::<usize>().is_ok() => {
                (n.trim_end_matches('x').parse().unwrap(), rest.trim())
            }
            _ => (1, l),
        };
        let c = CardDb::global()
            .get(name)
            .unwrap_or_else(|| panic!("unknown card {name}"));
        for _ in 0..n {
            out.push(c.clone());
        }
    }
    DeckList { main, sideboard }
}

pub fn default_deck() -> DeckList {
    parse_decklist(
        "12 Mountain\n12 Forest\n4 Grizzly Bears\n4 Lightning Bolt\n4 Hill Giant\n4 Shock\n4 Raging Goblin\n4 Gray Ogre\n4 Giant Growth\n4 Llanowar Elves\n4 Craw Wurm",
    )
}

/// Nonland cards that can go in a random deck: real paper cards legal somewhere, with
/// mana value at most 7, and their color identity as a WUBRG bit mask.
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
            let front = front.split(" // ").next().unwrap_or(front);
            if front.is_empty() || front.contains("Land") || c.cmc.unwrap_or(0.0) > 7.0 {
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

/// A random 60-card deck: one or two colors, 24 basic lands split between them, and 12
/// distinct fully supported spells in those colors (or colorless), three copies each.
pub fn random_deck(rng: &mut StdRng) -> DeckList {
    const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let mut colors: Vec<usize> = (0..5).collect();
    colors.shuffle(rng);
    colors.truncate(rng.gen_range(1..=2));
    let mask = colors.iter().fold(0u8, |m, &c| m | 1 << c);
    let candidates: Vec<&str> = pool()
        .iter()
        .filter(|(_, ci)| ci & !mask == 0)
        .map(|(n, _)| n.as_str())
        .collect();
    let mut spells: Vec<Arc<CardDef>> = Vec::new();
    for _ in 0..5000 {
        if spells.len() == 12 || candidates.is_empty() {
            break;
        }
        let name = candidates[rng.gen_range(0..candidates.len())];
        if spells.iter().any(|d| d.name.eq_ignore_ascii_case(name)) {
            continue;
        }
        match CardDb::global().get(name) {
            Some(def) if def.is_fully_supported() => spells.push(def),
            _ => {}
        }
    }
    let db = CardDb::global();
    let mut main: Vec<Arc<CardDef>> = (0..24)
        .map(|i| {
            db.get(BASICS[colors[i % colors.len()]])
                .expect("basic land")
        })
        .collect();
    for s in &spells {
        for _ in 0..3 {
            main.push(s.clone());
        }
    }
    DeckList {
        main,
        sideboard: Vec::new(),
    }
}
