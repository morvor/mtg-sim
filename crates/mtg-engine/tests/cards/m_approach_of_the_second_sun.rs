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
    cr!("104.2b", "401.4");
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
    cast_approach(&mut t);
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
