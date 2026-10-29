//! CR 703.4e, 703.4g: the turn-based actions of the precombat main phase that come from
//! supplemental card types — setting a scheme in motion (Archenemy) and rolling to visit
//! Attractions.

use crate::r703_common::*;
use mtg_engine::ability::KeywordAction;
use mtg_engine::card::card;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::variants::{self, ROLLED_TO_VISIT, SET_IN_MOTION};
use mtg_engine::*;

fn set_in_motion_event(id: ObjectId) -> impl Fn(&Event) -> bool {
    move |e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name.as_str() == SET_IN_MOTION && *o == id)
    }
}

fn saprolings(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Saproling"))
        .count()
}

#[test]
fn the_archenemy_sets_a_scheme_in_motion_as_their_precombat_main_phase_begins() {
    cr!("703.4e");
    supported("Roots of All Evil");
    supported("Look Skyward and Despair");
    let mut t = archenemy_game();
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![
            (*card("Roots of All Evil")).clone(),
            (*card("Look Skyward and Despair")).clone(),
        ],
    );
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    // Before anyone receives priority, the top scheme was moved off the scheme deck and
    // turned face up.
    let began = event_index(&t, step_began(Step::PrecombatMain)).expect("main phase began");
    let set = event_index(&t, set_in_motion_event(deck[0])).expect("scheme set in motion");
    assert!(set > began);
    assert!(!t.obj(deck[0]).face_down);
    assert_eq!(variants::scheme_deck(&t.g, P0), vec![deck[1]]);
    // "When you set this scheme in motion, create five 1/1 green Saproling creature
    // tokens."
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(saprolings(&t, P0), 5);

    // The other players aren't archenemies: nothing is set in motion on their turns.
    to_step_start(&mut t, P1, Step::PrecombatMain);
    assert!(event_index(&t, step_began(Step::PrecombatMain)).is_some());
    assert!(event_index(
        &t,
        |e| matches!(e, Event::Custom { name, .. } if name.as_str() == SET_IN_MOTION)
    )
    .is_none());
    assert_eq!(variants::scheme_deck(&t.g, P0).len(), 2);
}

fn history_and_booth(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Information Booth");
    let saga = t.battlefield(P0, "History of Benalia");
    let booth = t.battlefield(P0, "Information Booth");
    (saga, booth)
}

#[test]
fn the_active_player_rolls_to_visit_their_attractions_after_lore_counters() {
    cr!("703.4g");
    let mut t = TestGame::new(2);
    let (saga, _booth) = history_and_booth(&mut t);
    // Information Booth has 2 and 6 lit up: "Visit — Draw a card."
    t.g.dice.loaded.push_back(2);
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    let began = event_index(&t, step_began(Step::PrecombatMain)).unwrap();
    let lore = event_index(&t, |e| {
        matches!(e, Event::CountersAdded { target: Entity::Object(o), kind, .. }
            if *o == saga && kind.as_str() == counters::LORE)
    })
    .expect("lore counter");
    let rolled = event_index(
        &t,
        |e| matches!(e, Event::DieRolled { player, sides: 6, .. } if *player == P0),
    )
    .expect("rolled a six-sided die");
    let visited = event_index(
        &t,
        |e| matches!(e, Event::Custom { name, amount: 2, .. } if name.as_str() == ROLLED_TO_VISIT),
    )
    .expect("rolled to visit");
    assert!(began < lore && lore < rolled && rolled <= visited);
    // The visit ability triggered; the Saga's chapter ability too.
    let hand = t.hand_size(P0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);

    // A result that isn't lit up visits nothing.
    t.g.dice.loaded.push_back(3);
    keyword_action(&mut t, P0, KeywordAction::RollAttractions, 1);
    assert!(t.g.pending_triggers.is_empty());
}

#[test]
fn a_player_who_controls_no_attractions_doesnt_roll_to_visit() {
    cr!("703.4g");
    let mut t = TestGame::new(2);
    history_and_booth(&mut t);
    t.set_step(P0, Step::End);
    to_step_start(&mut t, P1, Step::PrecombatMain);
    assert!(event_index(&t, step_began(Step::PrecombatMain)).is_some());
    assert!(event_index(&t, |e| matches!(e, Event::DieRolled { .. })).is_none());
}
