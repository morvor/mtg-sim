//! Targets that must have a relationship with each other (`src/target_groups.rs`,
//! parsed in `oracle::phrases::parse_target`): "up to three target cards from a single
//! graveyard", "two target creature cards that share a creature type".

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn cards_from_a_single_graveyard_compile() {
    assert_compiles(&[
        "Decompose",
        "Arashin Sunshield",
        "Famished Ghoul",
        "Rag Dealer",
        "Carrion Beetles",
        "Unbury",
        "Nullmage Advocate",
        "Spurnmage Advocate",
    ]);
}

#[test]
fn decompose_exiles_cards_from_one_graveyard_only() {
    cr!("115.1", "601.2c");
    let mut t = TestGame::new(2);
    let spell = t.hand(P0, "Decompose");
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let c = t.graveyard(P1, "Llanowar Elves");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::B, 2);
    // Cards from two graveyards: only those from the first one's graveyard are kept.
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Hill Giant"));

    // Cards from one graveyard are fine.
    let mut t = TestGame::new(2);
    let spell = t.hand(P0, "Decompose");
    let a = t.graveyard(P1, "Grizzly Bears");
    let c = t.graveyard(P1, "Llanowar Elves");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::B, 2);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(c)])
        .go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Llanowar Elves"));
}
