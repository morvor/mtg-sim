//! Approach of the Second Sun (hand-written, `src/cards/approach_of_the_second_sun.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn cast_approach(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Plains", 7);
    let a = t.hand(P0, "Approach of the Second Sun");
    t.cast(P0, a).go();
    t.resolve();
    a
}

#[test]
fn first_one_goes_seventh_from_the_top_and_gains_seven() {
    cr!("104.2b");
    ruling!("Approach of the Second Sun", "place Approach of the Second Sun just under them");
    let mut t = TestGame::new(2);
    cast_approach(&mut t);
    assert_eq!(t.life(P0), 27);
    assert!(t.g.result.is_none());
    let lib = &t.g.players[0].library;
    let seventh = lib[lib.len() - 7];
    assert_eq!(t.g.obj(seventh).chars.name.as_str(), "Approach of the Second Sun");
}

#[test]
fn second_one_cast_from_hand_wins() {
    cr!("104.2b");
    ruling!("Approach of the Second Sun", "it checks only whether the first one was cast, not whether the first one resolved");
    let mut t = TestGame::new(2);
    // The first one is countered.
    t.lands(P0, "Plains", 7);
    let a = t.hand(P0, "Approach of the Second Sun");
    t.cast(P0, a).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    let first = t.g.stack.last().copied().unwrap();
    t.cast(P1, cs).target(first).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Approach of the Second Sun"));
    assert_eq!(t.life(P0), 20);
    cast_approach(&mut t);
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn an_opponents_approach_doesnt_count() {
    cr!("104.2b");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Plains", 7);
    let a = t.hand(P1, "Approach of the Second Sun");
    t.cast(P1, a).go();
    t.resolve();
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    cast_approach(&mut t);
    assert!(t.g.result.is_none());
    assert_eq!(t.life(P0), 27);
}

#[test]
fn the_second_one_must_be_cast_from_hand() {
    cr!("104.2b");
    ruling!("Approach of the Second Sun", "The second Approach of the Second Sun that you cast must be cast from your hand");
    let mut t = TestGame::new(2);
    cast_approach(&mut t);
    // A second one cast from exile doesn't win: it's put into the library again.
    t.lands(P0, "Plains", 7);
    let a = t.exile(P0, "Approach of the Second Sun");
    mtg_engine::casting::grant_play_permission(&mut t.g, P0, vec![a], ability::Duration::Permanent, false, None);
    t.cast(P0, a).go();
    t.resolve();
    assert!(t.g.result.is_none());
    assert_eq!(t.life(P0), 34);
}

#[test]
fn the_first_one_may_have_been_cast_from_anywhere() {
    cr!("104.2b");
    ruling!("Approach of the Second Sun", "but first may have been cast from anywhere");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 7);
    let a = t.exile(P0, "Approach of the Second Sun");
    mtg_engine::casting::grant_play_permission(&mut t.g, P0, vec![a], ability::Duration::Permanent, false, None);
    t.cast(P0, a).go();
    t.resolve();
    assert_eq!(t.life(P0), 27);
    cast_approach(&mut t);
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn with_fewer_than_seven_cards_it_goes_on_the_bottom() {
    cr!("401.7");
    ruling!("Approach of the Second Sun", "If you have fewer than six cards in your library");
    let mut t = TestGame::new(2);
    t.g.players[0].library.truncate(3);
    cast_approach(&mut t);
    let lib = &t.g.players[0].library;
    assert_eq!(lib.len(), 4);
    // The bottom card is the first in the list.
    assert_eq!(t.g.obj(lib[0]).chars.name.as_str(), "Approach of the Second Sun");
}
