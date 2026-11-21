//! Rulings batch S10 — jump-start (CR 702.133): "You may cast this card from your
//! graveyard by discarding a card in addition to paying its other costs. Then exile this
//! card."

use crate::r_s01_common::{supported, watch};
use crate::r_s03_common::respond;
use crate::r_s07_common::is_priority;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const JUMP_START: CastMethod = CastMethod::Keyword(KeywordKind::JumpStart);

/// Casts a spell with jump-start whenever that's among the actions offered.
fn jump_start_when_offered(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::Priority { actions } => actions
            .iter()
            .find(|a| matches!(a, Action::Cast { method, .. } if *method == JUMP_START))
            .map(|a| Answer::Action(a.clone())),
        _ => None,
    }
}

#[test]
fn a_jump_start_card_put_into_the_graveyard_can_be_cast_before_opponents_act() {
    cr!("117.3b", "702.133a");
    ruling!(
        "Radical Idea",
        "If a card with jump-start is put into your graveyard during your turn, you'll be able to cast it right away if it's legal to do so, before an opponent can take any actions."
    );
    supported("Radical Idea");
    // Radical Idea ({1}{U} instant): "Draw a card." P0 casts it from the hand; once it
    // resolves and is in the graveyard, P0 receives priority first and jump-starts it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let idea = t.hand(P0, "Radical Idea");
    t.hand(P0, "Grizzly Bears");
    t.cast(P0, idea).go();
    respond(&mut t, P0, jump_start_when_offered);
    // Whenever P1 receives priority: is Radical Idea in P0's graveyard?
    let p1_saw = watch(&mut t, P1, is_priority, |g| {
        !g.find_in_zone(Zone::Graveyard(P0), "Radical Idea")
            .is_empty()
    });
    let ok = t.g.run_until(10_000, |g| {
        !g.find_in_zone(Zone::Exile, "Radical Idea").is_empty()
    });
    assert!(ok, "Radical Idea wasn't jump-started");
    // P1 never had priority while the card was in the graveyard.
    assert!(!p1_saw.lock().unwrap().iter().any(|x| *x));
    assert!(!p1_saw.lock().unwrap().is_empty());
    // It drew twice, and Grizzly Bears was discarded.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 2);
}
