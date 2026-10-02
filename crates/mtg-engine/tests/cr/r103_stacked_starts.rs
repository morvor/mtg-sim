//! CR 103.3a, 103.5, 103.7: shuffled scheme decks, and stacked starts
//! (`GameConfig::top_of_library`), which put named cards on top of the shuffled library
//! or supplementary deck so a simulation can give a player a chosen opening hand or
//! starting plane.

use crate::r100_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

const SCHEMES: [&str; 6] = [
    "What's Yours Is Now Mine",
    "Dark Wings Bring Your Downfall",
    "My Undead Horde Awakens",
    "Nature Demands an Offering",
    "Every Dream a Nightmare",
    "Dance, Pathetic Marionette",
];

#[test]
fn scheme_decks_are_shuffled_before_the_game_begins() {
    cr!("103.3a");
    let mut orders = std::collections::BTreeSet::new();
    for seed in 0..6 {
        let deck: Vec<Arc<CardDef>> = fillers(20)
            .into_iter()
            .chain(SCHEMES.iter().map(|n| card(n)))
            .collect();
        let mut t = pregame(
            GameConfig {
                skip_mulligans: true,
                seed,
                ..GameConfig::supervillain_rumble()
            },
            vec![deck, fillers(20)],
        );
        t.g.start();
        let order = mtg_engine::variants::scheme_deck(&t.g, P0);
        assert_eq!(order.len(), SCHEMES.len());
        orders.insert(order);
    }
    assert!(orders.len() > 1, "the scheme deck was never shuffled");
}

#[test]
fn a_stacked_start_puts_the_named_cards_in_the_opening_hand() {
    cr!("103.5");
    let deck = || -> Vec<Arc<CardDef>> {
        fillers(30)
            .into_iter()
            .chain(["Lightning Bolt", "Shock", "Mountain"].map(card))
            .collect()
    };
    for seed in 0..4 {
        let mut t = pregame(
            GameConfig {
                skip_mulligans: true,
                seed,
                top_of_library: vec![
                    vec![],
                    ["Shock", "Mountain", "Lightning Bolt"]
                        .map(SmolStr::new)
                        .to_vec(),
                ],
                ..GameConfig::default()
            },
            vec![deck(), deck()],
        );
        t.g.start();
        // The opening hand is drawn from the top of the library (CR 103.5).
        for name in ["Shock", "Mountain", "Lightning Bolt"] {
            assert!(t.in_hand(P1, name), "seed {seed}: {name} not in hand");
        }
        assert_eq!(t.hand_size(P1), 7);
        assert_eq!(t.library_size(P1) + t.hand_size(P1), 33);
    }
}

#[test]
fn a_stacked_start_can_name_the_top_card_of_a_planar_deck() {
    cr!("103.7");
    let planes = [
        "The Great Aerie",
        "Strixhaven",
        "The Windy City",
        "Shy Town",
        "The Pro Tour",
        "Horizon Boughs",
    ];
    for (seed, top) in planes.iter().enumerate() {
        let deck: Vec<Arc<CardDef>> = fillers(20)
            .into_iter()
            .chain(planes.iter().map(|n| card(n)))
            .collect();
        let mut t = pregame(
            GameConfig {
                variant: Variant::Planechase,
                starting_player: Some(P0),
                skip_mulligans: true,
                seed: seed as u64,
                top_of_library: vec![vec![SmolStr::new(*top)]],
                ..GameConfig::default()
            },
            vec![deck, fillers(20)],
        );
        t.g.start();
        // The starting player's top planar card becomes the starting plane.
        let face_up = mtg_engine::planechase::face_up_planar_cards(&t.g);
        assert_eq!(face_up.len(), 1);
        assert_eq!(t.g.obj(face_up[0]).chars.name, *top);
        assert_eq!(t.g.obj(face_up[0]).zone, Zone::Command);
        assert_eq!(t.g.obj(face_up[0]).owner, P0);
    }
}
