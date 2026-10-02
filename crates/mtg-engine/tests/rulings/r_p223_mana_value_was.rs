//! Rulings batch P223 — "If its mana value was N or less, [effect]" (Perilous Voyage and
//! the other cards worded that way): the mana value of the object the spell acted on, as
//! it last existed (CR 608.2h).

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s17_common::become_copy;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn giants(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.library_top(p, "Hill Giant");
    }
}

/// A Grizzly Bears of P0's with a +1/+1 counter, for proliferate to find.
fn countered_bears(t: &mut TestGame) -> ObjectId {
    let b = t.battlefield(P0, "Grizzly Bears");
    crate::r_s13_common::add(t, b, counters::PLUS1, 1);
    b
}

fn asked_prompt(t: &TestGame, from: usize, prefix: &str) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.starts_with(prefix)))
        .count()
}

/// P0 casts `name` at P1's `target`; returns the game and the decision index before it
/// resolved.
fn cast_at(t: &mut TestGame, name: &str, target: ObjectId) -> usize {
    supported(name);
    let spell = in_hand_with_mana(t, P0, name);
    t.cast(P0, spell).target(target).go();
    let from = t.asked().len();
    t.resolve_all();
    from
}

#[test]
fn perilous_voyage_uses_the_mana_value_the_permanent_had_on_the_battlefield() {
    cr!("608.2h", "707.2");
    ruling!("Perilous Voyage", "Use the permanent's mana value as it existed on the battlefield to determine whether you scry.");
    // Mana value 2: scry 2. Mana value 4: no scry.
    for (name, scry) in [("Grizzly Bears", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        let x = t.battlefield(P1, name);
        let from = cast_at(&mut t, "Perilous Voyage", x);
        assert!(t.in_hand(P1, name));
        assert_eq!(scry_sizes(&t, P0, from), if scry { vec![2] } else { vec![] });
    }
    // A Hill Giant that's a copy of Grizzly Bears has mana value 2 on the battlefield
    // (4 once it's back in its owner's hand): scry 2.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    become_copy(&mut t, giant, bears);
    let from = cast_at(&mut t, "Perilous Voyage", giant);
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
    // A token (mana value 0) ceases to exist in the hand; it was mana value 0.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let tok = create_token(&mut t, P1, "Soldier");
    let from = cast_at(&mut t, "Perilous Voyage", tok);
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
}

#[test]
fn perilous_voyage_with_an_illegal_target_doesnt_scry() {
    cr!("608.2b");
    ruling!("Perilous Voyage", "If the target permanent is an illegal target by the time Perilous Voyage resolves, the entire spell doesn't resolve. You won't scry.");
    supported("Perilous Voyage");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pv = in_hand_with_mana(&mut t, P0, "Perilous Voyage");
    t.cast(P0, pv).target(bears).go();
    destroy(&mut t, bears);
    let from = t.asked().len();
    t.resolve_all();
    assert!(scries_since(&t, from).is_empty());
}

#[test]
fn other_if_its_mana_value_was_spells() {
    cr!("608.2c", "608.2h");
    // Fading Hope: "Return target creature to its owner's hand. If its mana value was 3 or
    // less, scry 1."
    for (name, yes) in [("Grizzly Bears", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        let x = t.battlefield(P1, name);
        let from = cast_at(&mut t, "Fading Hope", x);
        assert!(t.in_hand(P1, name));
        assert_eq!(scry_sizes(&t, P0, from).len(), usize::from(yes), "{name}");
    }
    // Extinguish the Light: "Destroy target creature or planeswalker. If its mana value
    // was 3 or less, you gain 3 life."
    for (name, yes) in [("Grizzly Bears", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, name);
        cast_at(&mut t, "Extinguish the Light", x);
        assert!(t.in_graveyard(P1, name));
        assert_eq!(t.life(P0), if yes { 23 } else { 20 }, "{name}");
    }
    // Tainted Treats: "Destroy target artifact or creature. If its mana value was 4 or
    // less, create a Food token."
    for (name, yes) in [("Hill Giant", true), ("Serra Angel", false)] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, name);
        cast_at(&mut t, "Tainted Treats", x);
        assert!(t.in_graveyard(P1, name));
        assert_eq!(with_subtype(&t, P0, "Food").len(), usize::from(yes), "{name}");
    }
    // Raze to the Ground: "Destroy target artifact. If its mana value was 1 or less, draw
    // a card."
    for (name, yes) in [("Ornithopter", true), ("Mind Stone", false)] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, name);
        let hand = t.hand_size(P0);
        cast_at(&mut t, "Raze to the Ground", x);
        assert!(t.in_graveyard(P1, name));
        assert_eq!(t.hand_size(P0), hand + usize::from(yes), "{name}");
    }
    // Seedship Impact: "Destroy target artifact or enchantment. If its mana value was 2
    // or less, create a Lander token."
    for (name, yes) in [("Mind Stone", true), ("Glorious Anthem", false)] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, name);
        cast_at(&mut t, "Seedship Impact", x);
        assert!(t.in_graveyard(P1, name));
        assert_eq!(with_subtype(&t, P0, "Lander").len(), usize::from(yes), "{name}");
    }
    // Carnivorous Canopy: "Destroy target artifact, enchantment, or creature with flying.
    // If that permanent's mana value was 3 or less, proliferate."
    for (name, yes) in [("Wind Drake", true), ("Serra Angel", false)] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, name);
        let b = countered_bears(&mut t);
        t.answer_choose(P0, &[Entity::Object(b)]);
        let from = cast_at(&mut t, "Carnivorous Canopy", x);
        assert_eq!(t.counters(b, counters::PLUS1), 1 + u32::from(yes));
        assert!(t.in_graveyard(P1, name));
        assert_eq!(asked_prompt(&t, from, "Proliferate"), usize::from(yes), "{name}");
    }
}

#[test]
fn counter_target_spell_if_that_spells_mana_value_was() {
    cr!("608.2c", "608.2h", "701.6a");
    // Reject Imperfection: "Counter target spell. If that spell's mana value was 3 or
    // less, proliferate." Sound the Trumpets: "... 2 or less, recruit."
    for (counter, prefix) in [
        ("Reject Imperfection", "Proliferate"),
        ("Sound the Trumpets", "Recruit"),
    ] {
        supported(counter);
        for (name, yes) in [("Grizzly Bears", true), ("Hill Giant", false)] {
            let mut t = TestGame::new(2);
            countered_bears(&mut t);
            t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
            let card = in_hand_with_mana(&mut t, P1, name);
            let spell = t.cast(P1, card).go();
            let c = in_hand_with_mana(&mut t, P0, counter);
            t.cast(P0, c).target(spell).go();
            let from = t.asked().len();
            t.resolve_all();
            assert!(t.in_graveyard(P1, name), "{counter}: {name} countered");
            assert_eq!(
                asked_prompt(&t, from, prefix),
                usize::from(yes),
                "{counter}: {name}"
            );
        }
    }
}
