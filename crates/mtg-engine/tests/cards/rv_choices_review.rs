//! Review checks for the choice grammar: wordings read faithfully or not at all.

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn random_cards_from_outside_the_game_are_not_read_as_permanents() {
    // "a creature card with mana value X chosen at random", "a copy of a Liliana
    // planeswalker chosen at random": cards outside the game, not the battlefield.
    for name in [
        "Momir Vig, Simic Visionary Avatar",
        "The Disciple of Vess",
    ] {
        assert!(
            !card(name).unsupported_text().is_empty(),
            "{name} compiles"
        );
    }
}

#[test]
fn aether_gust_a_permanent_goes_on_top_or_bottom_of_its_owners_library() {
    cr!("115.1", "608.2d");
    let def = card("Aether Gust");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Aether Gust");
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    let now = t.g.current(bears);
    assert_eq!(t.zone(now), Zone::Library(P1), "{}", t.dump_log());
    assert_eq!(t.g.player(P1).library[0], now, "the owner chose the bottom");
}

#[test]
fn dubious_challenge_the_exiled_cards_are_not_the_cards_looked_at() {
    // "exile up to two creature cards from among them, then shuffle. Target opponent may
    // choose one of the exiled cards ... Put the rest onto the battlefield under your
    // control.": read as choosing among (and putting onto the battlefield) the cards
    // looked at, which were shuffled away. Not read until "the exiled cards" is.
    assert!(!card("Dubious Challenge").unsupported_text().is_empty());
}
