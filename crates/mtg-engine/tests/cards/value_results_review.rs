//! Value grammar II, review checks: results and referents of earlier instructions in
//! cards whose compiled abilities the value grammar changed or newly reads ("that
//! creature" after sacrificing a land, "the creature tapped this way" by a cost, "cards
//! exiled this way" in a reflexive ability, "each player discards" then counting the
//! discarded cards).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn spiked_ripsaw_gives_trample_to_the_attacking_creature_not_the_sacrificed_forest() {
    cr!("608.2c", "701.21a");
    assert_supported("Spiked Ripsaw");
    let mut t = TestGame::new(2);
    let ripsaw = t.battlefield(P0, "Spiked Ripsaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    t.g.attach(ripsaw, Entity::Object(bears));
    t.g.recompute();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert!(!t.on_battlefield(forest));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn keldon_battlewagon_gets_the_power_of_the_creature_its_cost_tapped() {
    cr!("602.2", "608.2h");
    assert_supported("Keldon Battlewagon");
    let mut t = TestGame::new(2);
    let kb = t.battlefield(P0, "Keldon Battlewagon");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, kb, 0, &[]).expect("activate");
    t.resolve();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.pt(kb), (3, 3));
}

#[test]
fn specter_of_mortality_reflexive_ability_counts_the_cards_exiled() {
    cr!("603.12", "608.2c");
    assert_supported("Specter of Mortality");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let sp = t.enter(P0, "Specter of Mortality");
    t.resolve_all();
    // Two cards exiled: -2/-2 to each other creature.
    assert_eq!(t.pt(wurm), (4, 2));
    assert_eq!(t.pt(sp), (3, 3));
}

#[test]
fn pako_counts_the_noncreature_cards_exiled_from_each_library() {
    cr!("608.2c", "701.13a");
    assert_supported("Pako, Arcane Retriever");
    let mut t = TestGame::new(2);
    let pako = t.battlefield(P0, "Pako, Arcane Retriever");
    t.library_top(P0, "Island");
    t.library_top(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(pako, Entity::Player(P1))], &[]);
    t.resolve_all();
    // One noncreature card (the Island) of the two exiled.
    assert_eq!(t.counters(t.g.current(pako), "+1/+1"), 1);
    assert!(t.in_exile("Island") && t.in_exile("Grizzly Bears"));
}

#[test]
fn kefka_counts_card_types_among_the_cards_every_player_discarded() {
    cr!("608.2c", "205.2a");
    let c = card("Kefka, Court Mage // Kefka, Ruler of Ruin");
    assert!(c.faces[0].unsupported.is_empty(), "{:?}", c.faces[0].unsupported);
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Island");
    for _ in 0..3 {
        t.library_top(P0, "Swamp");
    }
    t.enter(P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
    t.resolve_all();
    // A creature card and a land card were discarded: two cards drawn.
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 0);
}
