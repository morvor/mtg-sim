//! Rulings batch S34 — "Whenever you expend N" (CR 700.14) counts all the mana its
//! controller spent to cast spells this turn, including mana spent before the permanent
//! entered — even the mana spent to cast that permanent itself.

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts the real card `name` with lands for its mana cost and resolves it.
fn cast_resolved(t: &mut TestGame, name: &str) -> ObjectId {
    give_mana_for(t, P0, name);
    let card = t.hand(P0, name);
    t.cast(P0, card).go();
    t.resolve_all();
    t.g.current(card)
}

#[test]
fn teapot_slinger_cast_for_four_mana_cant_trigger_that_turn() {
    cr!("700.14", "603.2");
    ruling!(
        "Teapot Slinger",
        "A permanent with an ability that triggers whenever you “expend N” will see mana you spent to cast spells the turn it enters, including mana you spent before it entered."
    );
    supported("Teapot Slinger");
    // "Whenever you expend 4, this creature deals 2 damage to each opponent." Cast for
    // {3}{R} as P0's first spell, P0 expended 4 paying for it, before it entered: casting
    // Grizzly Bears afterward (six mana in all) doesn't trigger it.
    let mut t = TestGame::new(2);
    let slinger = cast_resolved(&mut t, "Teapot Slinger");
    assert!(t.on_battlefield(slinger));
    cast_resolved(&mut t, "Grizzly Bears");
    assert_eq!(t.life(P1), 20);
    // On the battlefield already, it sees the Wandertale Mentor ({R}{G}) cast earlier this
    // turn: the Bears are the fourth mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teapot Slinger");
    cast_resolved(&mut t, "Wandertale Mentor");
    assert_eq!(t.life(P1), 20);
    cast_resolved(&mut t, "Grizzly Bears");
    assert_eq!(t.life(P1), 18);
}

#[test]
fn wandertale_mentor_sees_the_mana_spent_to_cast_it() {
    cr!("700.14", "603.2");
    ruling!(
        "Wandertale Mentor",
        "A permanent with an ability that triggers whenever you “expend N” will see mana you spent to cast spells the turn it enters, including mana you spent before it entered."
    );
    supported("Wandertale Mentor");
    // "Whenever you expend 4, put a +1/+1 counter on this creature." Its own {R}{G} counts:
    // the two mana of Grizzly Bears cast next make four.
    let mut t = TestGame::new(2);
    let mentor = cast_resolved(&mut t, "Wandertale Mentor");
    assert_eq!(t.counters(mentor, "+1/+1"), 0);
    cast_resolved(&mut t, "Grizzly Bears");
    assert_eq!(t.counters(mentor, "+1/+1"), 1);
    // Another spell this turn: P0 expends 4 only once.
    cast_resolved(&mut t, "Grizzly Bears");
    assert_eq!(t.counters(mentor, "+1/+1"), 1);
}

#[test]
fn bakersbane_duo_cast_as_the_fourth_mana_never_triggers_that_turn() {
    cr!("700.14", "603.2");
    ruling!(
        "Bakersbane Duo",
        "A permanent with an ability that triggers whenever you \"expend N\" will see mana you spent to cast spells the turn it enters, including mana you spent before it entered."
    );
    supported("Bakersbane Duo");
    // "Whenever you expend 4, this creature gets +1/+1 until end of turn." Grizzly Bears
    // ({1}{G}) then Bakersbane Duo ({1}{G}): the fourth mana was spent on the Duo, before
    // it entered, so a third spell doesn't trigger it.
    let mut t = TestGame::new(2);
    cast_resolved(&mut t, "Grizzly Bears");
    let duo = cast_resolved(&mut t, "Bakersbane Duo");
    cast_resolved(&mut t, "Grizzly Bears");
    assert_eq!(t.pt(duo), (2, 2));
    // Cast first, the Duo sees its own two mana plus the Bears' two.
    let mut t = TestGame::new(2);
    let duo = cast_resolved(&mut t, "Bakersbane Duo");
    cast_resolved(&mut t, "Grizzly Bears");
    assert_eq!(t.pt(duo), (3, 3));
}
