//! Rulings batch S31 — processors (Blight Herder, Void Attendant): putting cards opponents
//! own from exile into their owners' graveyards (CR 400.3), choosing face-down exiled cards
//! by pile and at random within it (CR 406.3, 406.4), and processing while a replacement
//! effect exiles cards that would go to a graveyard (CR 614.6, 400.7).

use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::can_activate;
use crate::r_s04_common::{add_mana, graveyard_names};
use crate::r_s05_common::run_from;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Exiles `cards` face down together, by an effect of `source` (one pile, CR 406.4).
fn exile_face_down(t: &mut TestGame, source: ObjectId, cards: &[ObjectId]) {
    let targets: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    run_from(
        t,
        P1,
        Some(source),
        Effect::Exile {
            what: Sel::AllTargets,
            face_down: true,
            link: false,
        },
        &targets,
    );
    for c in cards {
        assert!(t.obj_now(*c).face_down && t.zone(*c) == Zone::Exile);
    }
}

/// The candidates offered by the choices of exiled cards asked of `p` since `from`.
fn exile_choices(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

fn in_graveyard_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    graveyard_names(t, p).iter().filter(|n| *n == name).count()
}

/// P0 activates Void Attendant ("{1}{G}, Put a card an opponent owns from exile into that
/// player's graveyard: Create a 1/1 colorless Eldrazi Scion creature token. ...") and
/// resolves it.
fn process_with_void_attendant(t: &mut TestGame, attendant: ObjectId) {
    add_mana(t, P0, ManaType::G, 2);
    t.activate(P0, attendant, 0, &[])
        .expect("activate Void Attendant");
    t.resolve_all();
}

#[test]
fn void_attendant_chooses_face_down_cards_by_pile_and_at_random() {
    cr!("406.3", "406.4", "602.2b");
    ruling!(
        "Void Attendant",
        "You can't look at face-down cards in exile unless an effect allows you to."
    );
    ruling!(
        "Void Attendant",
        "Face-down cards in exile are grouped using two criteria: what caused them to be exiled face down and when they were exiled face down."
    );
    supported("Void Attendant");
    // Three Islands P1 owns exiled face down together, then two Mountains later by the same
    // source: two piles. P0 can't look at any of them.
    let mut t = TestGame::new(2);
    let source = t.battlefield(P1, "Ornithopter");
    let islands: Vec<ObjectId> = (0..3).map(|_| t.hand(P1, "Island")).collect();
    exile_face_down(&mut t, source, &islands);
    let mountains: Vec<ObjectId> = (0..2).map(|_| t.hand(P1, "Mountain")).collect();
    exile_face_down(&mut t, source, &mountains);
    for c in islands.iter().chain(&mountains) {
        assert!(!facedown::can_look_at(&t.g, P0, t.g.current(*c)));
    }
    // Paying Void Attendant's cost, P0 is offered one card per pile (the first Mountain
    // represents its pile), picks the Mountains' pile, and a random Mountain goes to P1's
    // graveyard.
    let attendant = t.battlefield(P0, "Void Attendant");
    let mountain_pile: Vec<Entity> = mountains
        .iter()
        .map(|c| Entity::Object(t.g.current(*c)))
        .collect();
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![mountain_pile[0]]),
    );
    process_with_void_attendant(&mut t, attendant);
    let offers = exile_choices(&t, P0, from);
    assert!(!offers.is_empty());
    assert_eq!(offers[0].len(), 2, "one card per pile: {:?}", offers[0]);
    assert_eq!(in_graveyard_named(&t, P1, "Mountain"), 1);
    assert_eq!(in_graveyard_named(&t, P1, "Island"), 0);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn blight_herder_may_take_cards_from_different_opponents() {
    cr!("400.3", "603.2");
    ruling!(
        "Blight Herder",
        "If a spell or ability requires that you put more than one exiled card into the graveyard, you may choose cards owned by different opponents. Each card chosen will be put into its owner's graveyard."
    );
    supported("Blight Herder");
    // "When you cast this spell, you may put two cards your opponents own from exile into
    // their owners' graveyards. If you do, create three 1/1 colorless Eldrazi Scion
    // creature tokens."
    let mut t = TestGame::new(3);
    let a = t.exile(P1, "Grizzly Bears");
    let b = t.exile(P2, "Hill Giant");
    let herder = t.hand(P0, "Blight Herder");
    add_mana(&mut t, P0, ManaType::C, 5);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, herder).go();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Graveyard(P1));
    assert_eq!(t.zone(b), Zone::Graveyard(P2));
    assert_eq!(tokens(&t, P0).len(), 3);
}

#[test]
fn processing_under_rest_in_peace_keeps_the_card_in_exile() {
    cr!("614.6", "400.7");
    ruling!(
        "Void Attendant",
        "If a replacement effect will cause cards that would be put into a graveyard from anywhere to be exiled instead (such as the one created by Anafenza, the Foremost), you can still put an exiled card into its opponent's graveyard."
    );
    ruling!(
        "Blight Herder",
        "In this situation, you can't use a single exiled card if required to put more than one exiled card into the graveyard."
    );
    supported("Rest in Peace");
    // Rest in Peace: "If a card or token would be put into a graveyard from anywhere, exile
    // it instead." P1 owns one exiled card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    let card = t.exile(P1, "Grizzly Bears");
    let attendant = t.battlefield(P0, "Void Attendant");
    // Void Attendant's cost can be paid with it: it stays in exile as a new object.
    process_with_void_attendant(&mut t, attendant);
    assert_eq!(tokens(&t, P0).len(), 1);
    let again = t.g.current(card);
    assert_ne!(again, card);
    assert_eq!(t.zone(again), Zone::Exile);
    assert_eq!(t.graveyard_size(P1), 0);
    // A separate activation can use the same card again.
    assert!(
        can_activate(&mut t, P0, attendant) || {
            add_mana(&mut t, P0, ManaType::G, 2);
            can_activate(&mut t, P0, attendant)
        }
    );
    process_with_void_attendant(&mut t, attendant);
    assert_eq!(tokens(&t, P0).len(), 2);
    // Blight Herder needs two cards: the one card can't count twice, so no Scions.
    let herder = t.hand(P0, "Blight Herder");
    add_mana(&mut t, P0, ManaType::C, 5);
    t.answer_yes(P0, true);
    t.cast(P0, herder).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert!(t.in_exile("Grizzly Bears"));
}
