//! Rulings batch S05 — the processors among the devoid cards: "put a card an opponent owns
//! from exile into that player's graveyard" (as an effect and as a cost).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
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
        P0,
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

/// Cards named `name` in `p`'s graveyard.
fn in_graveyard_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    graveyard_names(t, p).iter().filter(|n| *n == name).count()
}

#[test]
fn face_down_exiled_cards_are_chosen_by_pile_and_at_random_within_it() {
    cr!("406.4", "406.3");
    ruling!(
        "Wasteland Strangler",
        "Face-down cards in exile are grouped using two criteria: what caused them to be exiled face down and when they were exiled face down."
    );
    ruling!(
        "Wasteland Strangler",
        "You can’t look at face-down cards in exile unless an effect allows you to."
    );
    supported("Wasteland Strangler");
    // Three Islands P1 owns exiled face down together, then two Mountains later by the same
    // source: two piles.
    let mut t = TestGame::new(2);
    let source = t.battlefield(P1, "Ornithopter");
    let islands: Vec<ObjectId> = (0..3).map(|_| t.hand(P1, "Island")).collect();
    exile_face_down(&mut t, source, &islands);
    let mountains: Vec<ObjectId> = (0..2).map(|_| t.hand(P1, "Mountain")).collect();
    exile_face_down(&mut t, source, &mountains);
    // P0 can't look at them.
    for c in islands.iter().chain(&mountains) {
        assert!(!facedown::can_look_at(&t.g, P0, t.g.current(*c)));
    }
    // Wasteland Strangler: "When this creature enters, you may put a card an opponent owns
    // from exile into that player's graveyard. If you do, target creature gets -3/-3 until
    // end of turn." P0 is offered one card per pile, and picks the Islands' pile.
    let wurm = t.battlefield(P1, "Craw Wurm");
    let island_pile: Vec<Entity> = islands
        .iter()
        .map(|c| Entity::Object(t.g.current(*c)))
        .collect();
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    // The first Island represents its pile.
    t.answer(
        P0,
        DecisionKind::Entities,
        mtg_engine::decision::Answer::Entities(vec![island_pile[0]]),
    );
    enter(&mut t, P0, "Wasteland Strangler");
    t.resolve_all();
    let offers = exile_choices(&t, P0, from);
    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].len(), 2, "one card per pile: {:?}", offers[0]);
    assert!(offers[0].contains(&island_pile[0]));
    // A random card of the Islands' pile went to P1's graveyard; the Mountains stayed.
    assert_eq!(in_graveyard_named(&t, P1, "Island"), 1);
    assert_eq!(in_graveyard_named(&t, P1, "Mountain"), 0);
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn cards_owned_by_different_opponents_go_to_their_owners_graveyards() {
    cr!("400.3", "406.4");
    ruling!(
        "Ulamog's Nullifier",
        "If a spell or ability requires that you put more than one exiled card into the graveyard, you may choose cards owned by different opponents. Each card chosen will be put into its owner’s graveyard."
    );
    supported("Ulamog's Nullifier");
    // Ulamog's Nullifier (flash): "When this creature enters, you may put two cards your
    // opponents own from exile into their owners' graveyards. If you do, counter target
    // spell."
    let mut t = TestGame::new(3);
    let a = t.exile(P1, "Grizzly Bears");
    let b = t.exile(P2, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(Entity::Player(P0)).go();
    give_mana_for(&mut t, P0, "Ulamog's Nullifier");
    let nullifier = t.hand(P0, "Ulamog's Nullifier");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.cast(P0, nullifier).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn processing_under_rest_in_peace_leaves_the_card_in_exile_as_a_new_object() {
    cr!("614.6", "400.7");
    ruling!(
        "Wasteland Strangler",
        "If a replacement effect will cause cards that would be put into a graveyard from anywhere to be exiled instead (such as the one created by Anafenza, the Foremost), you can still put an exiled card into its opponent’s graveyard. The card becomes a new object and remains in exile. In this situation, you can’t use a single exiled card if required to put more than one exiled card into the graveyard. Conversely, you could use the same card in this situation if two separate spells or abilities each required you to put a single exiled card into its owner’s graveyard."
    );
    supported("Rest in Peace");
    // Rest in Peace: "If a card or token would be put into a graveyard from anywhere, exile
    // it instead." P1 owns one exiled card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    let card = t.exile(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // A Wasteland Strangler processes it: it stays in exile, and the Wurm gets -3/-3.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    enter(&mut t, P0, "Wasteland Strangler");
    t.resolve_all();
    assert_eq!(t.pt(wurm), (3, 1));
    let again = t.g.current(card);
    assert_ne!(again, card);
    assert_eq!(t.zone(again), Zone::Exile);
    assert_eq!(t.graveyard_size(P1), 0);
    // A second Strangler can use the same card: the Wurm gets -3/-3 again and dies.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    enter(&mut t, P0, "Wasteland Strangler");
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    // Ulamog's Nullifier needs two cards: with the one card P1 owns in exile, it can't put
    // it into the graveyard twice, and Shock isn't countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    t.exile(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(Entity::Player(P0)).go();
    give_mana_for(&mut t, P0, "Ulamog's Nullifier");
    let nullifier = t.hand(P0, "Ulamog's Nullifier");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.cast(P0, nullifier).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn a_processor_cost_needs_a_card_an_opponent_owns_in_exile() {
    cr!("118.3", "602.2b");
    supported("Oracle of Dust");
    supported("Processor Assault");
    // Oracle of Dust: "{2}, Put a card an opponent owns from exile into that player's
    // graveyard: Draw a card, then discard a card."
    let mut t = TestGame::new(2);
    let oracle = t.battlefield(P0, "Oracle of Dust");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.exile(P0, "Hill Giant");
    assert!(!can_activate(&mut t, P0, oracle));
    let theirs = t.exile(P1, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, oracle));
    t.activate(P0, oracle, 0, &[]).unwrap();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
    t.resolve_all();
    // Processor Assault: "As an additional cost to cast this spell, put a card an opponent
    // owns from exile into that player's graveyard." (and 5 damage to target creature).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    give_mana_for(&mut t, P0, "Processor Assault");
    let assault = t.hand(P0, "Processor Assault");
    assert!(!can_cast(&mut t, P0, assault, mtg_engine::object::CastMethod::Normal));
    t.exile(P1, "Grizzly Bears");
    t.cast(P0, assault).target(wurm).go();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}
