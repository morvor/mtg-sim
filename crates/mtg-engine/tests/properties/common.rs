//! Decks and setups shared by the property tests.

use mtg_engine::properties::*;
use mtg_engine::CardDef;
use std::sync::Arc;

pub fn red_green() -> Vec<Arc<CardDef>> {
    deck(
        "9 Mountain, 8 Forest, 3 Grizzly Bears, 3 Lightning Bolt, 2 Hill Giant, 2 Shock, \
         3 Llanowar Elves, 2 Giant Growth, 2 Craw Wurm, 2 Raging Goblin, \
         2 Prodigal Pyromancer, 2 Rampant Growth",
    )
}

pub fn white_blue() -> Vec<Arc<CardDef>> {
    deck(
        "9 Plains, 8 Island, 3 Savannah Lions, 3 Serra Angel, 2 Counterspell, 2 Divination, \
         3 Wind Drake, 2 Pacifism, 2 Air Elemental, 2 Disenchant, 2 Opt, 2 Healing Salve",
    )
}

/// A short game of the given decks.
pub fn setup(decks: Vec<Vec<Arc<CardDef>>>, seed: u64) -> GameSetup {
    let mut s = GameSetup::new(decks, seed);
    s.max_turns = 14;
    s.max_actions = 1500;
    s
}

/// The fixed set: two known decks, and random decks for two, three and four players.
pub fn fixed_set() -> Vec<GameSetup> {
    vec![
        setup(vec![red_green(), white_blue()], 1),
        setup(vec![random_deck(11), random_deck(12)], 2),
        setup(vec![random_deck(21), random_deck(22), random_deck(23)], 3),
        setup(
            vec![
                random_deck(31),
                random_deck(32),
                random_deck(33),
                random_deck(34),
            ],
            4,
        ),
    ]
}
