//! CR 701.32: set in motion; CR 701.33: abandon.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{KeywordAction, Sel};
use mtg_engine::card::{card, CardDef};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::variants::{self, ABANDONED, SET_IN_MOTION};
use mtg_engine::*;

const ROOTS: &str = "Roots of All Evil";
const SKYWARD: &str = "Look Skyward and Despair";
const SOIL: &str = "The Very Soil Shall Shake";
const ECHOES: &str = "My Laughter Echoes";

/// A three-player Archenemy game: P0 is the archenemy, facing P1 and P2.
fn archenemy_game() -> TestGame {
    TestGame::with_config(
        3,
        GameConfig {
            variant: Variant::Archenemy,
            teams: Some(vec![0, 1, 1]),
            ..Default::default()
        },
    )
}

fn real(name: &str) -> CardDef {
    (*card(name)).clone()
}

/// Puts scheme cards into `p`'s scheme deck (face down in the command zone), top first.
fn scheme_deck(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = names
        .iter()
        .map(|n| {
            let id = t.custom(p, real(n), Zone::Command);
            t.g.objects[id.0 as usize].face_down = true;
            id
        })
        .collect();
    t.g.recompute();
    ids
}

fn set_in_motion(t: &mut TestGame, p: PlayerId, n: i32) {
    run(t, p, None, ka(KeywordAction::SetInMotion, Sel::None, n), &[]);
    t.resolve_all();
}

fn tokens(t: &TestGame, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

/// The names of the cards in `p`'s scheme deck, top first.
fn deck(t: &TestGame, p: PlayerId) -> Vec<String> {
    variants::scheme_deck(&t.g, p)
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

fn face_up(t: &TestGame) -> Vec<String> {
    variants::face_up_schemes(&t.g)
        .iter()
        .map(|id| t.obj_now(*id).chars.name.to_string())
        .collect()
}

#[test]
fn only_the_archenemy_sets_scheme_cards_in_motion() {
    cr!("701.32a");
    // Not in another game.
    let mut t = TestGame::new(2);
    scheme_deck(&mut t, P0, &[ROOTS]);
    set_in_motion(&mut t, P0, 1);
    assert!(face_up(&t).is_empty());
    // Not by another player of an Archenemy game.
    let mut t = archenemy_game();
    scheme_deck(&mut t, P0, &[ROOTS]);
    scheme_deck(&mut t, P1, &[SKYWARD]);
    set_in_motion(&mut t, P1, 1);
    assert!(face_up(&t).is_empty());
    assert_eq!(tokens(&t, "Dragon"), 0);
    // The archenemy does.
    set_in_motion(&mut t, P0, 1);
    assert_eq!(tokens(&t, "Saproling"), 5);
    // Only a scheme card: a permanent can't be set in motion.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = custom_events(&t, SET_IN_MOTION).len();
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::SetInMotion, Sel::Target(0), 1),
        &[Entity::Object(bears)],
    );
    assert_eq!(custom_events(&t, SET_IN_MOTION).len(), before);
    assert!(t.on_battlefield(bears));
}

#[test]
fn setting_a_scheme_in_motion_moves_it_off_the_deck_face_up() {
    cr!("701.32b");
    let mut t = archenemy_game();
    let ids = scheme_deck(&mut t, P0, &[ROOTS, SKYWARD]);
    set_in_motion(&mut t, P0, 1);
    assert_eq!(custom_events(&t, SET_IN_MOTION), vec![(Some(P0), Some(ids[0]), 0)]);
    // "When you set this scheme in motion, create five 1/1 green Saproling creature
    // tokens." Then, not ongoing, it goes to the bottom of the scheme deck.
    assert_eq!(tokens(&t, "Saproling"), 5);
    assert_eq!(deck(&t, P0), vec![SKYWARD, ROOTS]);
}

#[test]
fn a_scheme_is_set_in_motion_even_if_its_already_face_up() {
    cr!("701.32b");
    supported(ECHOES);
    // My Laughter Echoes: "Whenever you set a non-ongoing scheme in motion, you may abandon
    // this scheme. If you do, set that scheme in motion again."
    let mut t = archenemy_game();
    scheme_deck(&mut t, P0, &[ECHOES, ROOTS, SKYWARD]);
    set_in_motion(&mut t, P0, 1);
    assert_eq!(face_up(&t), vec![ECHOES]);
    t.answer_yes(P0, true);
    set_in_motion(&mut t, P0, 1);
    // Roots of All Evil was set in motion twice: the second time it was already face up
    // and off the deck, and it still counts.
    assert_eq!(tokens(&t, "Saproling"), 10);
    let roots_events = custom_events(&t, SET_IN_MOTION)
        .into_iter()
        .filter(|(_, o, _)| {
            o.is_some_and(|o| {
                t.obj_now(o)
                    .card
                    .as_ref()
                    .is_some_and(|c| c.front().chars.name == ROOTS)
            })
        })
        .count();
    assert_eq!(roots_events, 2);
    // My Laughter Echoes was abandoned; Roots of All Evil returned to the deck.
    assert!(face_up(&t).is_empty());
    assert_eq!(deck(&t, P0), vec![SKYWARD, ECHOES, ROOTS]);
    assert_eq!(custom_events(&t, ABANDONED).len(), 1);
}

#[test]
fn several_schemes_are_set_in_motion_one_at_a_time() {
    cr!("701.32c");
    let mut t = archenemy_game();
    let ids = scheme_deck(&mut t, P0, &[ROOTS, SKYWARD, SOIL]);
    set_in_motion(&mut t, P0, 2);
    assert_eq!(
        custom_events(&t, SET_IN_MOTION),
        vec![(Some(P0), Some(ids[0]), 0), (Some(P0), Some(ids[1]), 0)]
    );
    assert_eq!(tokens(&t, "Saproling"), 5);
    assert_eq!(tokens(&t, "Dragon"), 1);
    assert_eq!(deck(&t, P0)[0], SOIL);
}

#[test]
fn only_a_face_up_ongoing_scheme_can_be_abandoned() {
    cr!("701.33a");
    let abandon = |t: &mut TestGame, id: ObjectId| {
        run(
            t,
            P0,
            None,
            ka(KeywordAction::Abandon, Sel::Target(0), 1),
            &[Entity::Object(id)],
        );
    };
    let mut t = archenemy_game();
    let ids = scheme_deck(&mut t, P0, &[ROOTS, SOIL]);
    // A face-down scheme in the scheme deck.
    abandon(&mut t, ids[1]);
    assert!(custom_events(&t, ABANDONED).is_empty());
    // A face-up non-ongoing scheme (its trigger is waiting, so it's still face up).
    run(&mut t, P0, None, ka(KeywordAction::SetInMotion, Sel::None, 1), &[]);
    assert_eq!(face_up(&t), vec![ROOTS]);
    abandon(&mut t, ids[0]);
    assert!(custom_events(&t, ABANDONED).is_empty());
    assert_eq!(face_up(&t), vec![ROOTS]);
    t.resolve_all();
    // A face-up ongoing scheme.
    set_in_motion(&mut t, P0, 1);
    assert_eq!(face_up(&t), vec![SOIL]);
    abandon(&mut t, ids[1]);
    assert_eq!(custom_events(&t, ABANDONED).len(), 1);
    assert!(face_up(&t).is_empty());
    // Not in another game.
    let mut t = TestGame::new(2);
    let ids = scheme_deck(&mut t, P0, &[SOIL]);
    variants::turn_face_up_in_command(&mut t.g, ids[0]);
    t.g.recompute();
    abandon(&mut t, ids[0]);
    assert!(custom_events(&t, ABANDONED).is_empty());
}

#[test]
fn abandoning_turns_the_scheme_face_down_on_the_bottom_of_the_scheme_deck() {
    cr!("701.33b");
    supported(SOIL);
    // "Creatures you control get +2/+2 and have trample. When a creature you control
    // dies, abandon this scheme."
    let mut t = archenemy_game();
    scheme_deck(&mut t, P0, &[SOIL, ROOTS, SKYWARD]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    set_in_motion(&mut t, P0, 1);
    assert_eq!(t.pt(giant), (5, 5));
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(face_up(&t).is_empty());
    assert_eq!(deck(&t, P0), vec![ROOTS, SKYWARD, SOIL]);
    assert_eq!(t.pt(giant), (3, 3));
}
