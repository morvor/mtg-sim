//! Discard, then draw that many: "discard up to two cards, then draw that many cards"
//! (Kinetic Augur), "Discard all the cards in your hand, then draw that many cards."
//! (Decaying Time Loop), "discard any number of cards, then draw that many cards plus one"
//! (Colossus of the Blood Age). The player chooses what to discard (CR 701.9b); "that many"
//! is how many were discarded.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
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
fn kinetic_augur_discards_up_to_two_and_draws_that_many() {
    cr!("701.9a", "701.9b");
    ruling!(
        "Kinetic Augur",
        "you choose whether to discard zero, one, or two cards, discard them, and draw that many"
    );
    assert_supported("Kinetic Augur");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let augur = t.hand(P0, "Kinetic Augur");
    let bolt = t.hand(P0, "Lightning Bolt");
    let forest = t.hand(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, augur).go();
    t.resolve();
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"), "{}", t.dump_log());
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    // One discarded, one drawn.
    assert_eq!(t.library_size(P0), lib - 1);
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn kinetic_augur_discarding_nothing_draws_nothing() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let augur = t.hand(P0, "Kinetic Augur");
    t.hand(P0, "Lightning Bolt");
    t.cast(P0, augur).go();
    t.resolve();
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib);
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn decaying_time_loop_discards_hand_and_draws_that_many() {
    cr!("701.9a");
    assert_supported("Decaying Time Loop");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let lt = t.hand(P0, "Decaying Time Loop");
    t.hand(P0, "Lightning Bolt");
    t.hand(P0, "Forest");
    t.hand(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.cast(P0, lt).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), lib - 3);
}

#[test]
fn colossus_of_the_blood_age_draws_that_many_plus_one() {
    cr!("701.9a", "701.9b");
    assert_supported("Colossus of the Blood Age");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Colossus of the Blood Age");
    let bolt = t.hand(P0, "Lightning Bolt");
    let forest = t.hand(P0, "Forest");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    // Kill it: its dies trigger lets P0 discard any number of cards.
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(colossus).go();
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(forest)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Colossus of the Blood Age"));
    assert!(t.in_graveyard(P0, "Forest"), "{}", t.dump_log());
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // Two discarded, three drawn.
    assert_eq!(t.library_size(P0), lib - 3);
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn colossus_of_the_blood_age_may_discard_nothing_and_draw_one() {
    cr!("107.1c", "608.2d");
    ruling!(
        "Colossus of the Blood Age",
        "You may choose to discard no cards and just draw a card."
    );
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Colossus of the Blood Age");
    t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(colossus).go();
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Colossus of the Blood Age"));
    assert!(t.in_hand(P0, "Lightning Bolt"), "{}", t.dump_log());
    assert_eq!(t.library_size(P0), lib - 1);
    assert_eq!(t.hand_size(P0), 2);
}
