//! Rulings batch S32 — basic snow lands in limited decks (CR 100.2b): unlike other basic
//! lands, they can't be added beyond the card pool.

use crate::r_s01_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_limited, DeckProblem};
use mtg_engine::*;
use std::sync::Arc;

fn cards(list: &[(&str, usize)]) -> Vec<Arc<CardDef>> {
    list.iter()
        .flat_map(|(n, k)| std::iter::repeat_n(card(n), *k))
        .collect()
}

#[test]
fn basic_snow_lands_come_only_from_the_limited_card_pool() {
    cr!("100.2b");
    ruling!(
        "Snow-Covered Island",
        "In a Limited event (usually Booster Draft or Sealed Deck), you can't add basic snow lands to your card pool as you would other basic lands. You can play with basic snow lands only if you open them in your sealed deck or draft them."
    );
    supported("Snow-Covered Island");
    let pool = cards(&[("Grizzly Bears", 23), ("Snow-Covered Island", 2)]);
    // Any number of Islands.
    let ok = cards(&[("Grizzly Bears", 23), ("Island", 17)]);
    assert!(check_limited(&ok, &pool).is_empty());
    // The two opened Snow-Covered Islands, and no more.
    let ok = cards(&[
        ("Grizzly Bears", 23),
        ("Snow-Covered Island", 2),
        ("Island", 15),
    ]);
    assert!(check_limited(&ok, &pool).is_empty());
    let too_many = cards(&[("Grizzly Bears", 23), ("Snow-Covered Island", 17)]);
    assert_eq!(
        check_limited(&too_many, &pool),
        vec![DeckProblem::NotInPool {
            name: "Snow-Covered Island".into(),
            have: 17,
            available: 2,
        }]
    );
}
