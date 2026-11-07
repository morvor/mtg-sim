//! CR 701.31: planeswalk.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{KeywordAction, Sel};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::planechase::{self, PlanarFace, PLANAR_DIE_ACTION, PLANESWALKED};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn planechase_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Planechase,
            ..Default::default()
        },
    )
}

/// Puts plane cards into `p`'s planar deck (face down in the command zone), top first.
fn planar_deck(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
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

/// A custom plane card at the bottom of `p`'s planar deck.
fn custom_plane(t: &mut TestGame, p: PlayerId, name: &str, text: &str) -> ObjectId {
    let id = t.custom(
        p,
        text_card(name, "Plane — Test Realm", "", None, text),
        Zone::Command,
    );
    t.g.objects[id.0 as usize].face_down = true;
    t.g.recompute();
    id
}

fn face_up(t: &TestGame) -> Vec<String> {
    planechase::face_up_planar_cards(&t.g)
        .iter()
        .map(|id| t.obj_now(*id).chars.name.to_string())
        .collect()
}

fn deck_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    // Face down, they have no name: the cards' names.
    planechase::planar_deck(&t.g, p)
        .iter()
        .map(|id| {
            t.obj_now(*id)
                .card
                .as_ref()
                .map(|c| c.front().chars.name.to_string())
                .unwrap_or_default()
        })
        .collect()
}

fn planeswalk(t: &mut TestGame, p: PlayerId) {
    run(t, p, None, ka(KeywordAction::Planeswalk, Sel::None, 1), &[]);
    t.resolve_all();
}

/// Rolls the planar die as a special action, making it show `face`.
fn roll(t: &mut TestGame, p: PlayerId, face: PlanarFace) {
    for seed in 0u64.. {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let r: u32 = rng.gen_range(1..=6);
        if planechase::face_for(r) == face {
            t.g.rng = ChaCha8Rng::seed_from_u64(seed);
            break;
        }
    }
    t.g.turn.priority = Some(p);
    t.g.perform_action(
        p,
        Action::Special(SpecialAction::Other {
            name: PLANAR_DIE_ACTION.into(),
            obj: None,
        }),
    )
    .expect("roll");
    t.settle();
}

#[test]
fn only_the_planar_controller_of_a_planechase_game_may_planeswalk() {
    cr!("701.31a");
    ruling!(
        "Start the TARDIS",
        "You may planeswalk only in games using the Planechase variant. If you are instructed to planeswalk in any other game, nothing happens."
    );
    supported("Start the TARDIS");
    // "Surveil 2, then draw a card. You may planeswalk."
    let mut t = TestGame::new(2);
    planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Start the TARDIS");
    t.answer_yes(P0, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(face_up(&t).is_empty());
    assert_eq!(deck_names(&t, P0), vec!["Goldmeadow", "Krosa"]);
    // In a Planechase game, a player other than the planar controller can't.
    let mut t = planechase_game();
    planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
    planar_deck(&mut t, P1, &["Panopticon"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    planeswalk(&mut t, P1);
    assert_eq!(face_up(&t), vec!["Goldmeadow"]);
    planeswalk(&mut t, P0);
    assert_eq!(face_up(&t), vec!["Krosa"]);
}

#[test]
fn face_up_planes_go_to_the_bottom_of_their_owners_planar_decks() {
    cr!("701.31b");
    let mut t = planechase_game();
    planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
    planar_deck(&mut t, P1, &["The Fourth Sphere", "Panopticon"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up(&t), vec!["Goldmeadow"]);
    // On P1's turn, P1 (the planar controller) planeswalks: P0's Goldmeadow goes to the
    // bottom of P0's planar deck face down, and P1 turns the top card of their own planar
    // deck face up.
    t.set_step(P1, Step::PrecombatMain);
    t.g.recompute();
    planeswalk(&mut t, P1);
    assert_eq!(face_up(&t), vec!["The Fourth Sphere"]);
    assert_eq!(deck_names(&t, P0), vec!["Krosa", "Goldmeadow"]);
    assert_eq!(deck_names(&t, P1), vec!["Panopticon"]);
    let sphere = planechase::face_up_planar_cards(&t.g)[0];
    assert_eq!(t.obj_now(sphere).owner, P1);
    assert_eq!(t.obj_now(sphere).controller, P1);
}

#[test]
fn ways_to_planeswalk() {
    cr!("701.31c");
    ruling!(
        "Murasa",
        "A plane card is treated as if its text box included \"When you roll {PW}, put this card on the bottom of its owner's planar deck face down, then move the top card of your planar deck off that planar deck and turn it face up.\""
    );
    // The planeswalking ability.
    let mut t = planechase_game();
    planar_deck(&mut t, P0, &["Goldmeadow", "Krosa", "Littjara"]);
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Planeswalker);
    t.resolve_all();
    assert_eq!(face_up(&t), vec!["Krosa"]);
    // A phenomenon's triggered ability leaving the stack. Mutual Epiphany: "When you
    // encounter Mutual Epiphany, each player draws four cards."
    let mut t = planechase_game();
    planar_deck(&mut t, P0, &["Goldmeadow", "Mutual Epiphany", "Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    planeswalk(&mut t, P0);
    assert_eq!(face_up(&t), vec!["Krosa"]);
    let planeswalks = custom_events(&t, PLANESWALKED).len();
    assert_eq!(planeswalks, 2);
    // An instruction: "You may planeswalk."
    let mut t = planechase_game();
    planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Start the TARDIS");
    t.answer_yes(P0, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(face_up(&t), vec!["Krosa"]);
}

#[test]
fn planeswalking_to_and_away_from_a_plane() {
    cr!("701.31d");
    supported("Littjara");
    // Littjara: "When you planeswalk to Littjara and at the beginning of your upkeep,
    // create a 2/2 blue Shapeshifter creature token with changeling."
    let mut t = planechase_game();
    planar_deck(&mut t, P0, &["Goldmeadow", "Littjara"]);
    let away = custom_plane(
        &mut t,
        P0,
        "Test Plane",
        "When you planeswalk away from this plane, you gain 3 life.",
    );
    planechase::set_starting_plane(&mut t.g);
    planeswalk(&mut t, P0);
    assert_eq!(face_up(&t), vec!["Littjara"]);
    let shapeshifters = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Shapeshifter"))
            .count()
    };
    assert_eq!(shapeshifters(&t), 1);
    // Planeswalking away from Littjara doesn't trigger it again; planeswalking to the
    // test plane then away from it triggers its ability, although it's face down by then.
    planeswalk(&mut t, P0);
    assert_eq!(face_up(&t), vec!["Test Plane"]);
    assert_eq!(shapeshifters(&t), 1);
    assert_eq!(t.life(P0), 20);
    planeswalk(&mut t, P0);
    assert_eq!(face_up(&t), vec!["Goldmeadow"]);
    assert_eq!(t.life(P0), 23);
    assert!(t.obj_now(t.g.current(away)).face_down);
}
