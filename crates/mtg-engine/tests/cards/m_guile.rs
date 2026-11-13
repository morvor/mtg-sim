//! Guile (hand-written, `src/cards/guile.rs`): "If a spell or ability you control would
//! counter a spell, instead exile that spell and you may play that card without paying
//! its mana cost." (a replacement of countering, CR 701.6, 614.1a).

use mtg_engine::testing::*;
use mtg_engine::*;

fn setup() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guile");
    t.g.turn.active = P1;
    t.g.turn.priority = Some(P1);
    t.lands(P1, "Mountain", 4);
    let giant = t.hand(P1, "Hill Giant");
    let spell = t.cast(P1, giant).go();
    (t, spell)
}

fn counter_it(t: &mut TestGame, spell: ObjectId) {
    let cs = t.hand(P0, "Counterspell");
    t.lands(P0, "Island", 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, cs).target(spell).go();
    t.resolve();
}

#[test]
fn the_countered_spell_is_exiled_and_may_be_cast_for_free() {
    cr!("701.6a", "614.1a");
    ruling!("Guile", "You have the option to cast it immediately upon its exile");
    let (mut t, spell) = setup();
    t.answer_yes(P0, true);
    counter_it(&mut t, spell);
    // Guile's controller cast it: it's on the stack, then resolves under their control.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(t.obj_now(giant[0]).controller, P0);
    assert!(!t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn exiling_is_mandatory_casting_is_not() {
    cr!("614.1a");
    ruling!("Guile", "Exiling the spell is mandatory. Casting it is not.");
    ruling!("Guile", "A spell exiled this way was never actually countered.");
    let (mut t, spell) = setup();
    t.answer_yes(P0, false);
    counter_it(&mut t, spell);
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn a_spell_that_cant_be_countered_is_unaffected() {
    cr!("701.6a");
    ruling!("Guile", "Guile's ability has no effect on it");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guile");
    t.g.turn.active = P1;
    t.g.turn.priority = Some(P1);
    t.lands(P1, "Forest", 6);
    let tyrant = t.hand(P1, "Carnage Tyrant");
    let spell = t.cast(P1, tyrant).go();
    counter_it(&mut t, spell);
    t.resolve_all();
    let on = t.named_on_battlefield("Carnage Tyrant");
    assert_eq!(on.len(), 1);
    assert_eq!(t.obj_now(on[0]).controller, P1);
}

#[test]
fn an_opponents_counterspell_counters_normally() {
    cr!("701.6a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guile");
    t.lands(P0, "Mountain", 4);
    let giant = t.hand(P0, "Hill Giant");
    let spell = t.cast(P0, giant).go();
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}
