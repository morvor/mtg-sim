//! Susan Foreman (hand-written, `src/cards/susan_foreman.rs`).

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::planechase;
use mtg_engine::testing::*;
use mtg_engine::*;

fn game() -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Planechase,
            ..Default::default()
        },
    );
    let deck: Vec<ObjectId> = ["Goldmeadow", "The Fourth Sphere", "Panopticon"]
        .iter()
        .map(|n| {
            let id = t.command(P0, n);
            t.g.objects[id.0 as usize].face_down = true;
            id
        })
        .collect();
    t.g.recompute();
    (t, deck)
}

#[test]
fn the_top_card_can_go_to_the_bottom_before_planeswalking() {
    cr!("701.31b", "614.1a");
    let (mut t, deck) = game();
    t.battlefield(P0, "Susan Foreman");
    t.answer_choose(P0, &[Entity::Object(deck[0])]);
    let to = planechase::planeswalk(&mut t.g, P0).unwrap();
    assert_eq!(t.obj(to).chars.name.as_str(), "The Fourth Sphere");
    // (Face down, so compared by object.)
    assert_eq!(planechase::planar_deck(&t.g, P0), vec![deck[2], deck[0]]);
}

#[test]
fn or_the_second_card_to_keep_the_top_one() {
    cr!("701.31b");
    let (mut t, deck) = game();
    t.battlefield(P0, "Susan Foreman");
    t.answer_choose(P0, &[Entity::Object(deck[1])]);
    let to = planechase::planeswalk(&mut t.g, P0).unwrap();
    assert_eq!(t.obj(to).chars.name.as_str(), "Goldmeadow");
    assert_eq!(planechase::planar_deck(&t.g, P0), vec![deck[2], deck[1]]);
}
