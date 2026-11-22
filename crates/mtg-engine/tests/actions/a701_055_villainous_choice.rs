//! CR 701.55: face a villainous choice.

use crate::a701_028_071_common::*;
use mtg_engine::kwa::villainous::FACED;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Dalek Emperor: "At the beginning of combat on your turn, each opponent faces a
/// villainous choice — That player sacrifices a creature of their choice, or you create a
/// 3/3 black Dalek artifact creature token with menace."
fn emperor_combat(t: &mut TestGame) {
    supported("The Dalek Emperor");
    t.battlefield(P0, "The Dalek Emperor");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
}

fn daleks(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Dalek"))
        .count()
}

#[test]
fn the_player_chooses_an_option_and_its_actions_are_performed() {
    cr!("701.55a");
    ruling!(
        "The Dalek Emperor",
        "When a player faces a villainous choice, they first choose one of the two options, then all actions in the chosen option are performed."
    );
    // The first option: that player sacrifices a creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    option(&mut t, P1, 0);
    emperor_combat(&mut t);
    assert!(!t.on_battlefield(bears));
    assert_eq!(daleks(&t), 0);
    // The second: you create a Dalek.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    option(&mut t, P1, 1);
    emperor_combat(&mut t);
    assert!(t.on_battlefield(bears));
    assert_eq!(daleks(&t), 1);
    let chooser: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| matches!(d, Decision::ChooseOption { .. }).then_some(p))
        .collect();
    assert_eq!(chooser, vec![P1]);
}

#[test]
fn an_impossible_option_can_be_chosen() {
    cr!("701.55b");
    ruling!(
        "The Dalek Emperor",
        "a player who controls no creatures can still choose that option."
    );
    let mut t = TestGame::new(2);
    option(&mut t, P1, 0);
    emperor_combat(&mut t);
    assert_eq!(daleks(&t), 0);
    assert_eq!(custom_events(&t, FACED), vec![(Some(P1), None, 0)]);
}

#[test]
fn a_replacement_effect_can_make_a_player_face_the_choice_again() {
    cr!("701.55c");
    supported("The Valeyard");
    // "If an opponent would face a villainous choice, they face that choice an additional
    // time."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Valeyard");
    let bears = t.battlefield(P1, "Grizzly Bears");
    option(&mut t, P1, 1);
    option(&mut t, P1, 0);
    emperor_combat(&mut t);
    // Once each way, one after the other.
    assert_eq!(daleks(&t), 1);
    assert!(!t.on_battlefield(bears));
    assert_eq!(
        custom_events(&t, FACED),
        vec![(Some(P1), None, 1), (Some(P1), None, 0)]
    );
    // It doesn't apply to the Valeyard's controller's own choices.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "The Valeyard");
    emperor_combat(&mut t);
    assert_eq!(custom_events(&t, FACED).len(), 1);
}

#[test]
fn several_players_face_the_choice_one_at_a_time_in_apnap_order() {
    cr!("701.55d");
    ruling!(
        "The Dalek Emperor",
        "the first player in turn order makes their choice and the action for that choice is performed before the next player makes their choice."
    );
    let mut t = TestGame::new(3);
    // P1 (next in turn order) chooses first and that option is performed before P2
    // chooses.
    option(&mut t, P1, 1);
    option(&mut t, P2, 1);
    emperor_combat(&mut t);
    let ev = custom_events(&t, FACED);
    assert_eq!(ev, vec![(Some(P1), None, 1), (Some(P2), None, 1)]);
    assert_eq!(daleks(&t), 2);
    // The first Dalek existed when P2 chose: the tokens entered between the choices.
    let asked: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| matches!(d, Decision::ChooseOption { .. }).then_some(p))
        .collect();
    assert_eq!(asked, vec![P1, P2]);
    let created: Vec<usize> = t
        .turn_events
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            matches!(e, mtg_engine::events::Event::TokenCreated { .. }).then_some(i)
        })
        .collect();
    let faced: Vec<usize> = t
        .turn_events
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            matches!(e, mtg_engine::events::Event::Custom { name, .. } if name == FACED)
                .then_some(i)
        })
        .collect();
    assert!(created[0] < faced[1]);
}
