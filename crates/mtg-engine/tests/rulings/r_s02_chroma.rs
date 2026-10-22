//! Rulings batch S02 — chroma (an ability word, CR 207.2c): counts of the mana symbols of
//! a color in mana costs, e.g. "the number of red mana symbols in the mana costs of
//! permanents you control" (devotion to that color, CR 700.5).

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

fn goats(t: &TestGame) -> usize {
    with_subtype(t, P0, "Goat").len()
}

#[test]
fn chroma_counts_hybrid_mana_symbols_of_the_color() {
    cr!("207.2c", "700.5", "107.4e");
    ruling!(
        "Outrage Shaman",
        "Chroma abilities count hybrid mana symbols of the appropriate color. For example, a card with mana cost {3}{U/R}{U/R} has two red mana symbols in its mana cost."
    );
    supported("Outrage Shaman");
    let mut t = TestGame::new(2);
    // Boros Reckoner: {R/W}{R/W}{R/W}.
    t.battlefield(P0, "Boros Reckoner");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Outrage Shaman ({3}{R}{R}): "When this creature enters, it deals damage to target
    // creature equal to the number of red mana symbols in the mana costs of permanents you
    // control." 3 hybrid + its own 2.
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.enter(P0, "Outrage Shaman");
    t.resolve_all();
    assert_eq!(t.obj(wurm).damage, 5);
    // Hybrid symbols count for each of their colors: Springjack Shepherd ({3}{W}) with
    // Kitchen Finks ({1}{G/W}{G/W}) makes three Goats.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kitchen Finks");
    t.enter(P0, "Springjack Shepherd");
    t.resolve_all();
    assert_eq!(goats(&t), 3);
}

#[test]
fn chroma_counts_only_mana_costs_not_symbols_in_text_boxes() {
    cr!("207.2c", "700.5", "202.1");
    ruling!(
        "Heartlash Cinder",
        "Chroma abilities check only mana costs, which are found in a card’s upper right corner. They don’t count mana symbols that appear in a card’s text box."
    );
    supported("Heartlash Cinder");
    let mut t = TestGame::new(2);
    // Fire Diamond ({2}): "{T}: Add {R}." A red symbol only in its text box.
    t.battlefield(P0, "Fire Diamond");
    // Heartlash Cinder ({1}{R} 1/1, haste): "When this creature enters, it gets +X/+0 until
    // end of turn, where X is the number of red mana symbols in the mana costs of
    // permanents you control."
    let cinder = t.enter(P0, "Heartlash Cinder");
    t.resolve_all();
    assert_eq!(t.pt(cinder), (2, 1));
}

#[test]
fn primalcrux_counts_only_mana_costs_not_symbols_in_text_boxes() {
    cr!("207.2c", "700.5", "604.3");
    ruling!(
        "Primalcrux",
        "Chroma abilities check only mana costs, which are found in a card's upper right corner. They don't count mana symbols that appear in a card's text box."
    );
    supported("Primalcrux");
    let mut t = TestGame::new(2);
    // Primalcrux ({G}{G}{G}{G}{G}{G}): "This creature's power and toughness are each equal
    // to the number of green mana symbols in the mana costs of permanents you control."
    let crux = t.battlefield(P0, "Primalcrux");
    assert_eq!(t.pt(crux), (6, 6));
    // Llanowar Elves ({G}; "{T}: Add {G}."): one more symbol, not two.
    t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.pt(crux), (7, 7));
    // Forests have no mana cost.
    t.lands(P0, "Forest", 2);
    assert_eq!(t.pt(crux), (7, 7));
    // An opponent's green permanents don't count.
    t.battlefield(P1, "Llanowar Elves");
    assert_eq!(t.pt(crux), (7, 7));
}

#[test]
fn chroma_counts_the_symbols_in_its_own_mana_cost() {
    cr!("207.2c", "700.5");
    ruling!(
        "Springjack Shepherd",
        "The effect counts the mana symbols in this cards mana cost as well."
    );
    supported("Springjack Shepherd");
    let mut t = TestGame::new(2);
    // Springjack Shepherd ({3}{W}): "When this creature enters, create a 0/1 white Goat
    // creature token for each white mana symbol in the mana costs of permanents you
    // control." Just its own {W}.
    t.enter(P0, "Springjack Shepherd");
    t.resolve_all();
    assert_eq!(goats(&t), 1);
    // Goat tokens have no mana cost; a second Shepherd counts both Shepherds.
    t.enter(P0, "Springjack Shepherd");
    t.resolve_all();
    assert_eq!(goats(&t), 3);
}
