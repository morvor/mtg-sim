//! The Celestial Toymaker (hand-written, `src/cards/the_celestial_toymaker.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn opponents_lose_2_life_for_each_spell_that_made_piles() {
    cr!("700.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Celestial Toymaker");
    // Fact or Fiction: "Reveal the top five cards of your library. An opponent separates
    // those cards into two piles. ..."
    t.lands(P0, "Island", 8);
    for _ in 0..2 {
        let f = t.hand(P0, "Fact or Fiction");
        t.cast(P0, f).go();
        t.resolve_all();
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn nothing_without_piles() {
    cr!("700.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Celestial Toymaker");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn opponents_lose_2_life_for_an_ability_that_made_a_player_guess() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Celestial Toymaker");
    // Liar's Pendulum: "{2}, {T}: Choose a card name. Target opponent guesses whether a
    // card with that name is in your hand. ..."
    let pendulum = t.battlefield(P0, "Liar's Pendulum");
    t.lands(P0, "Island", 2);
    t.activate(P0, pendulum, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
