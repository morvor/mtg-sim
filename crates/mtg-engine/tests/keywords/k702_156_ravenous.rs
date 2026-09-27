//! CR 702.156 Ravenous.

use crate::common_k702_153_167::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts Tervigon ({X}{1}{G}, 0/0 trample, ravenous) with the given X and resolves it.
fn cast_tervigon(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P0, "Forest", 2 + x as usize);
    let card = t.hand(P0, "Tervigon");
    t.cast(P0, card).x(x).go();
    t.resolve();
    named(t, P0, "Tervigon")[0]
}

#[test]
fn ravenous_enters_with_x_counters_and_draws_if_x_is_five_or_more() {
    cr!("702.156", "702.156a");
    assert_supported("Tervigon");
    // X = 3: three counters, no card.
    let mut t = TestGame::new(2);
    let tv = cast_tervigon(&mut t, 3);
    assert_eq!(plus1(&t, tv), 3);
    t.resolve_all();
    assert_eq!(t.pt(tv), (3, 3));
    assert_eq!(t.hand_size(P0), 0);
    // X = 5: five counters and a card.
    let mut t = TestGame::new(2);
    let tv = cast_tervigon(&mut t, 5);
    assert_eq!(plus1(&t, tv), 5);
    t.settle();
    assert_eq!(triggers_named(&t, "Ravenous").len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_ravenous_creature_has_its_counters_as_it_enters() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "Any triggered ability that looks for a creature with a certain power or toughness entering the battlefield will see the counters when it checks to see if it should trigger."
    );
    // Elemental Bond: "Whenever a creature you control with power 3 or greater enters,
    // draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    cast_tervigon(&mut t, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn the_draw_checks_the_chosen_x_not_the_counters_it_entered_with() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "The triggered ability that checks to see if X is 5 or greater refers to the value of X that was chosen as the spell was cast"
    );
    // Hardened Scales adds a counter: X = 4 gives five counters but no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let tv = cast_tervigon(&mut t, 4);
    assert_eq!(plus1(&t, tv), 5);
    t.settle();
    assert!(triggers_named(&t, "Ravenous").is_empty());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn a_ravenous_permanent_that_wasnt_cast_has_x_of_zero() {
    cr!("702.156a");
    ruling!(
        "Tervigon",
        "If another permanent enters the battlefield as a copy of a creature with Ravenous, it will not enter with any counters from the ravenous ability."
    );
    // Put onto the battlefield without being cast: no counters (a 0/0 dies).
    let mut t = TestGame::new(2);
    let tv = t.enter(P0, "Tervigon");
    assert_eq!(plus1(&t, tv), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(tv));
    // A Clone copying a Tervigon cast with X = 3 enters without counters.
    let mut t = TestGame::new(2);
    let tv = cast_tervigon(&mut t, 3);
    t.resolve_all();
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(tv)]);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Tervigon").len(), 1);
    assert!(t.on_battlefield(tv));
}
