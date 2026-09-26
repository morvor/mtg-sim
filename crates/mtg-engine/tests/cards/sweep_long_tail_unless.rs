//! "Unless [a player] pays" with scaled costs (Countervailing Winds, Rethink, Extravagant
//! Spirit), and "becomes the target" triggers that refer to the targeting spell or
//! ability: Frost Titan, Shimmering Glasskite, Bonecrusher Giant.

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

/// P1 casts Lightning Bolt at P0 with `lands` Mountains; P0 answers with Countervailing
/// Winds with `yard` cards in their graveyard. Returns P0's life afterwards.
fn winds(lands: usize, yard: usize, pay: bool) -> i32 {
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", lands);
    for _ in 0..yard {
        t.graveyard(P0, "Grizzly Bears");
    }
    let bolt = t.hand(P1, "Lightning Bolt");
    let winds = t.hand(P0, "Countervailing Winds");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.cast(P0, winds).target(spell).go();
    t.answer_yes(P1, pay);
    t.resolve_all();
    t.life(P0)
}

#[test]
fn countervailing_winds_costs_one_per_card_in_graveyard() {
    cr!("118.12a");
    ruling!(
        "Countervailing Winds",
        "Count the number of cards in your graveyard as Countervailing Winds resolves"
    );
    assert_supported("Countervailing Winds");
    // Two cards: P1 has two untapped Mountains left and pays.
    assert_eq!(winds(3, 2, true), 17);
    // Three cards: P1 can't pay {3} with two Mountains, so Bolt is countered.
    assert_eq!(winds(3, 3, true), 20);
    // P1 may decline to pay.
    assert_eq!(winds(3, 2, false), 20);
    // An empty graveyard: {0}, which P1 can pay.
    assert_eq!(winds(1, 0, true), 17);
}

#[test]
fn rethink_costs_the_spells_mana_value() {
    cr!("118.12a");
    ruling!("Rethink", "The spell’s controller gets the option to pay when this spell resolves.");
    assert_supported("Rethink");
    for (lands, countered) in [(4, false), (3, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P0, "Island", 3);
        t.lands(P1, "Forest", lands);
        let bears = t.hand(P1, "Grizzly Bears");
        let rethink = t.hand(P0, "Rethink");
        let spell = t.cast(P1, bears).go();
        t.cast(P0, rethink).target(spell).go();
        t.answer_yes(P1, true);
        t.resolve_all();
        // Grizzly Bears has mana value 2: P1 needs two more untapped lands.
        assert_eq!(
            t.in_graveyard(P1, "Grizzly Bears"),
            countered,
            "{}",
            t.dump_log()
        );
    }
}

#[test]
fn extravagant_spirit_costs_one_per_card_in_hand() {
    cr!("118.12a");
    assert_supported("Extravagant Spirit");
    for (cards, lands, survives) in [(2, 2, true), (3, 2, false)] {
        let mut t = TestGame::new(2);
        let spirit = t.battlefield(P0, "Extravagant Spirit");
        t.lands(P0, "Island", lands);
        t.set_step(P1, Step::End);
        // Cards in hand at P0's upkeep (the draw step comes after).
        for _ in 0..cards {
            t.hand(P0, "Grizzly Bears");
        }
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.on_battlefield(spirit), survives, "{}", t.dump_log());
    }
}

#[test]
fn frost_titan_taxes_opponents_spells_that_target_it() {
    cr!("603.2", "118.12a");
    ruling!(
        "Frost Titan",
        "affects each spell (including Aura spells), activated ability, and triggered ability"
    );
    assert_supported("Frost Titan");
    for (lands, countered) in [(1, true), (3, false)] {
        let mut t = TestGame::new(2);
        let titan = t.battlefield(P0, "Frost Titan");
        t.lands(P1, "Mountain", lands);
        let shock = t.hand(P1, "Shock");
        t.answer_yes(P1, true);
        t.cast(P1, shock).target(titan).go();
        t.resolve_all();
        assert_eq!(t.obj_now(titan).damage == 0, countered, "{}", t.dump_log());
        assert!(t.in_graveyard(P1, "Shock"));
    }
}

#[test]
fn frost_titan_ignores_its_controllers_spells() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Frost Titan");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(titan).go();
    t.resolve_all();
    assert_eq!(t.obj_now(titan).damage, 2);
}

#[test]
fn shimmering_glasskite_counters_only_the_first_each_turn() {
    cr!("603.2", "701.6a");
    assert_supported("Shimmering Glasskite");
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Shimmering Glasskite");
    t.lands(P1, "Mountain", 2);
    let s1 = t.hand(P1, "Shock");
    let s2 = t.hand(P1, "Shock");
    t.cast(P1, s1).target(kite).go();
    t.resolve_all();
    assert_eq!(t.obj_now(kite).damage, 0, "{}", t.dump_log());
    t.cast(P1, s2).target(kite).go();
    t.resolve_all();
    assert_eq!(t.obj_now(kite).damage, 2, "{}", t.dump_log());
}

#[test]
fn bonecrusher_giant_damages_the_spells_controller() {
    cr!("603.2");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "It resolves even if that spell is countered."
    );
    assert_supported("Bonecrusher Giant // Stomp");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Bonecrusher Giant // Stomp");
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Island", 2);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(giant).go();
    // The trigger goes on the stack above Shock; Shock is countered before it resolves.
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let negate = t.hand(P0, "Negate");
    t.cast(P0, negate).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}
