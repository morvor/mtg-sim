//! Conditional "instead" alternatives to the previous instruction (CR 608.2c).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn cut_propulsion_twice_that_much_damage_to_itself_if_it_has_flying() {
    cr!("608.2c", "120.3");
    supported("Cut Propulsion");
    // A 2/2 with flying deals 4 damage to itself; a 1/4 without flying deals 1.
    let mut t = TestGame::new(2);
    let flyer = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Cut Propulsion");
    t.cast(P0, spell).target(flyer).go();
    t.resolve();
    assert!(!t.on_battlefield(flyer));
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Horned Turtle");
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Cut Propulsion");
    t.cast(P0, spell).target(giant).go();
    t.resolve();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 1);
}

#[test]
fn gideon_s_triumph_two_of_those_creatures_with_a_gideon() {
    cr!("608.2c", "701.21a");
    supported("Gideon's Triumph");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let stay = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Gideon, Ally of Zendikar");
    t.lands(P0, "Plains", 2);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P0)), (b, Entity::Player(P0))], &[]);
    let spell = t.hand(P0, "Gideon's Triumph");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(stay));
}

#[test]
fn see_the_truth_all_three_cards_if_cast_from_elsewhere() {
    cr!("608.2c", "601.2");
    supported("See the Truth");
    supported("Accumulate Wisdom");
    // Cast from the hand: one card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "See the Truth");
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
}
