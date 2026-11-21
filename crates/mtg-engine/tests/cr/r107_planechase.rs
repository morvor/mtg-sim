//! CR 107.11–107.12: the Planeswalker symbol and the chaos symbol on the planar die
//! (Planechase, CR 901).

use super::r105_util::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::planechase::{self, PlanarFace, PLANAR_DIE_ACTION};
use mtg_engine::testing::*;
use mtg_engine::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub fn planechase_game(n: usize, single_deck: bool) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            variant: Variant::Planechase,
            single_planar_deck: single_deck,
            ..Default::default()
        },
    )
}

/// Puts plane cards into `p`'s planar deck (face down in the command zone), top first.
pub fn add_planar_deck(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = names
        .iter()
        .map(|n| {
            let id = t.command(p, n);
            t.g.objects[id.0 as usize].face_down = true;
            id
        })
        .collect();
    t.g.recompute();
    ids
}

/// Makes the next roll of the planar die show `face`.
pub fn force_next_roll(t: &mut TestGame, face: PlanarFace) {
    for seed in 0u64.. {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let r: u32 = rng.gen_range(1..=6);
        if planechase::face_for(r) == face {
            t.g.rng = ChaCha8Rng::seed_from_u64(seed);
            return;
        }
    }
}

/// Takes the special action of rolling the planar die.
pub fn roll(t: &mut TestGame, p: PlayerId, face: PlanarFace) {
    force_next_roll(t, face);
    t.g.turn.priority = Some(p);
    t.g.perform_action(
        p,
        Action::Special(SpecialAction::Other {
            name: PLANAR_DIE_ACTION.into(),
            obj: None,
        }),
    )
    .unwrap();
    t.settle();
}

pub fn face_up_names(t: &TestGame) -> Vec<String> {
    planechase::face_up_planar_cards(&t.g)
        .iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect()
}

#[test]
fn rolling_the_planeswalker_symbol_triggers_the_planeswalking_ability() {
    cr!("107.11");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere", "Panopticon"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    roll(&mut t, P0, PlanarFace::Planeswalker);
    // The planeswalking ability is on the stack, controlled by the player who rolled.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.obj(t.g.stack[0]).controller, P0);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    t.resolve();
    // P0 planeswalked: Goldmeadow went to the bottom of the planar deck face down, and
    // the next plane was turned face up.
    assert_eq!(face_up_names(&t), vec!["The Fourth Sphere"]);
    let deck = planechase::planar_deck(&t.g, P0);
    assert_eq!(deck.len(), 2);
    assert_eq!(t.obj(*deck.last().unwrap()).base.name.as_str(), "Goldmeadow");
    // No chaos ensued.
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn rolling_the_chaos_symbol_makes_chaos_ensue() {
    cr!("107.12");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere"]);
    planechase::set_starting_plane(&mut t.g);
    // Goldmeadow: "Whenever chaos ensues, create a 0/1 white Goat creature token."
    roll(&mut t, P0, PlanarFace::Chaos);
    t.resolve_all();
    let goats = tokens_of(&t, P0);
    assert_eq!(goats.len(), 1);
    assert!(t.obj(goats[0]).chars.has_subtype("Goat"));
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    // A blank face does nothing (the second roll this turn costs {1}).
    t.lands(P0, "Plains", 1);
    roll(&mut t, P0, PlanarFace::Blank);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
}

#[test]
fn an_ability_can_refer_to_rolling_the_chaos_symbol() {
    cr!("107.12");
    let mut t = planechase_game(2, false);
    let plane = card_from_text(
        "Test Plane",
        "",
        "Plane — Test",
        None,
        "Whenever you roll {CHAOS}, you gain 3 life.",
    );
    let id = t.custom(P0, plane, mtg_engine::object::Zone::Command);
    t.g.objects[id.0 as usize].face_down = true;
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Test Plane"]);
    roll(&mut t, P0, PlanarFace::Blank);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    t.lands(P0, "Plains", 1);
    roll(&mut t, P0, PlanarFace::Chaos);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}
