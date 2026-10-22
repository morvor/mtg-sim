//! Hidden information stays hidden: a player's requests are the same in games that differ
//! only in what that player can't see (CR 400.2, 401.2, 402.3, 406.3, 708.5). See
//! `mtg_api::hidden` and docs/PROPERTIES.md.

mod common;

use common::*;
use mtg_api::hidden::*;
use mtg_engine::*;
use std::sync::Arc;

fn deck(list: &[(usize, &str)]) -> Vec<Arc<CardDef>> {
    let db = CardDb::global();
    let mut v = Vec::new();
    for (n, name) in list {
        let c = db
            .get(name)
            .unwrap_or_else(|| panic!("unknown card {name}"));
        assert!(c.is_fully_supported(), "{name} isn't fully supported");
        for _ in 0..*n {
            v.push(c.clone());
        }
    }
    v
}

/// Decks full of secrets: morphs, manifests and cloaks, foretold cards, hands looked at
/// and revealed, libraries searched and looked at.
fn secret_decks() -> Vec<Vec<Arc<CardDef>>> {
    vec![
        deck(&[
            (9, "Island"),
            (8, "Plains"),
            (4, "Exalted Angel"),
            (4, "Willbender"),
            (4, "Saw It Coming"),
            (3, "Brainstorm"),
            (3, "Peek"),
            (2, "Telepathy"),
            (3, "Fugitive Codebreaker"),
        ]),
        deck(&[
            (9, "Forest"),
            (8, "Swamp"),
            (4, "Den Protector"),
            (4, "Whisperwood Elemental"),
            (4, "Soul Summons"),
            (3, "Thoughtseize"),
            (3, "Rampant Growth"),
            (2, "Duress"),
            (3, "Hide in Plain Sight"),
        ]),
    ]
}

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        max_turns: 14,
        max_actions: 1500,
        ..Default::default()
    }
}

fn check(decks: Vec<Vec<Arc<CardDef>>>, seed: u64) {
    match check_game(config(seed), decks, seed, 3) {
        Ok(s) => eprintln!(
            "seed {seed}: {} decisions, {} checked, {} objects changed, {} turns",
            s.decisions, s.checked, s.changed, s.turns
        ),
        Err(e) => panic!("seed {seed}: {e}"),
    }
}

#[test]
fn requests_dont_depend_on_hidden_information_with_face_down_cards() {
    cr!("400.2", "401.2", "402.3", "406.3", "708.5");
    for seed in 1..=2 {
        check(secret_decks(), seed);
    }
}

#[test]
fn requests_dont_depend_on_hidden_information_in_random_games() {
    cr!("400.2", "402.3");
    check(vec![random_deck(41), random_deck(42)], 3);
    check(vec![random_deck(43), random_deck(44), random_deck(45)], 4);
}

#[test]
fn object_ids_dont_tell_hidden_cards_apart() {
    cr!("400.2", "401.2", "402.3");
    for seed in 1..=3 {
        check_start(&config(seed), &secret_decks(), seed)
            .unwrap_or_else(|e| panic!("secret decks, seed {seed}: {e}"));
        check_start(
            &config(seed),
            &[
                random_deck(seed),
                random_deck(seed + 10),
                random_deck(seed + 20),
            ],
            seed,
        )
        .unwrap_or_else(|e| panic!("random decks, seed {seed}: {e}"));
    }
}

fn with(base: &[(usize, &str)], extra: &[&str]) -> Vec<Arc<CardDef>> {
    let mut d = deck(base);
    d.extend(
        extra
            .iter()
            .map(|n| CardDb::global().get(n).expect("a card")),
    );
    d
}

const ATTRACTIONS: [&str; 10] = [
    "Push Your Luck",
    "Scavenger Hunt",
    "The Superlatorium",
    "Guess Your Fate",
    "Gallery of Legends",
    "Information Booth",
    "Trivia Contest",
    "Log Flume",
    "Costume Shop",
    "Concession Stand",
];

/// The cards of an Attraction, planar or scheme deck are numbered anew once it's shuffled:
/// the ids they were made with, in the deck's order, would tell which card is where in
/// the face-down deck (found by the property with two Bounce Chambers in an Attraction
/// deck).
#[test]
fn the_ids_of_face_down_decks_in_the_command_zone_dont_tell_their_cards() {
    cr!("400.2", "406.3", "103.3a", "717.2");
    let lands = [(20usize, "Plains"), (20, "Island")];
    let attractions = vec![with(&lands, &ATTRACTIONS), with(&lands, &ATTRACTIONS[..3])];
    for seed in 1..=3 {
        check_start(&config(seed), &attractions, seed).unwrap_or_else(|e| panic!("{e}"));
    }
    let planes = [
        "The Great Aerie",
        "Strixhaven",
        "The Windy City",
        "Shy Town",
        "The Pro Tour",
        "Horizon Boughs",
    ];
    let planechase = GameConfig {
        variant: mtg_engine::game::Variant::Planechase,
        ..config(1)
    };
    let decks = vec![with(&lands, &planes), with(&lands, &planes)];
    for seed in 1..=3 {
        check_start(
            &GameConfig {
                seed,
                ..planechase.clone()
            },
            &decks,
            seed,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }
    let schemes = [
        "What's Yours Is Now Mine",
        "Dark Wings Bring Your Downfall",
        "My Undead Horde Awakens",
        "Nature Demands an Offering",
        "Every Dream a Nightmare",
        "Dance, Pathetic Marionette",
    ];
    let decks = vec![with(&lands, &schemes), with(&lands, &[])];
    for seed in 1..=3 {
        check_start(
            &GameConfig {
                seed,
                ..GameConfig::supervillain_rumble()
            },
            &decks,
            seed,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }
}

/// What the other players see of conspiracies put into the command zone face down (hidden
/// agenda) doesn't depend on which is which.
#[test]
fn the_ids_of_face_down_conspiracies_dont_tell_them_apart() {
    cr!("400.7", "702.106a", "905.4a");
    let lands = [(20usize, "Plains"), (20, "Island")];
    let decks = vec![deck(&lands), deck(&lands)];
    let side: Vec<Arc<CardDef>> = ["Brago's Favor", "Double Stroke", "Muzzio's Preparations"]
        .iter()
        .map(|n| CardDb::global().get(n).expect("a card"))
        .collect();
    for seed in 1..=4 {
        check_start_with_sideboards(
            &GameConfig {
                limited: true,
                ..config(seed)
            },
            &decks,
            &[side.clone(), side.clone()],
            seed,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }
}
