//! What "unless" applies to: in "[A], then [B] unless [condition]" only [B] depends on the
//! condition (CR 608.2c: the steps happen in the order written), and a character's
//! pronouns and opus's "that spell" in the condition.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// Casts Katara with six Islands and a Grizzly Bears in hand, paying her waterbend cost or
/// not; returns P0's hand and graveyard sizes after her enters trigger resolves.
fn katara(waterbend: bool) -> (usize, usize) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let k = t.hand(P0, "Katara, Seeking Revenge");
    t.hand(P0, "Grizzly Bears");
    let k = t.cast(P0, k).kicked(waterbend).go();
    t.resolve_all();
    assert!(t.on_battlefield(k));
    (t.hand_size(P0), t.graveyard_size(P0))
}

#[test]
fn katara_always_draws_and_discards_unless_her_additional_cost_was_paid() {
    cr!("608.2c", "601.2b", "118.8");
    assert_supported(&["Katara, Seeking Revenge"]);
    // "When Katara enters, draw a card, then discard a card unless her additional cost was
    // paid." Without the waterbend cost: draw, then discard.
    assert_eq!(katara(false), (1, 1), "drew one and discarded one");
    // With waterbend {2} paid: the draw still happens, the discard doesn't.
    assert_eq!(katara(true), (2, 0), "drew one, no discard");
}

#[test]
fn muse_seeker_discards_unless_five_or_more_mana_was_spent() {
    cr!("608.2c", "601.2h");
    assert_supported(&["Muse Seeker"]);
    // "Opus — Whenever you cast an instant or sorcery spell, draw a card. Then discard a
    // card unless five or more mana was spent to cast that spell."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muse Seeker");
    t.hand(P0, "Grizzly Bears");
    // Lightning Bolt (one mana): draw, then discard.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 2, "Lightning Bolt and the discarded card");
    // Blaze with X = 4 (five mana): draw, no discard.
    t.lands(P0, "Mountain", 5);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(4).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn phantasmal_sphere_always_gets_its_counter() {
    cr!("608.2c", "118.12");
    assert_supported(&["Phantasmal Sphere"]);
    // "At the beginning of your upkeep, put a +1/+1 counter on this creature, then
    // sacrifice this creature unless you pay {1} for each +1/+1 counter on it." Paying
    // doesn't skip the counter, and the counter it just got counts.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Phantasmal Sphere");
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(s));
    assert_eq!(t.counters(s, counters::PLUS1), 1);
    // Next upkeep: two counters, {2} to pay; with one land it's sacrificed.
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(s));
}
