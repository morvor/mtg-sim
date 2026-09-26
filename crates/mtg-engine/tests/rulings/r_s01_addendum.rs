//! Rulings batch S01 — addendum (an ability word, CR 207.2c): "If you cast this spell
//! during your main phase, [effect]."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Sphinx's Insight ("Draw two cards. Addendum — If you cast this spell during your
/// main phase, you gain 2 life.") for `p` in the current step.
fn cast_insight(t: &mut TestGame, p: PlayerId) -> ObjectId {
    give_mana_for(t, p, "Sphinx's Insight");
    let c = t.hand(p, "Sphinx's Insight");
    t.cast(p, c).go()
}

#[test]
fn addendum_applies_only_to_a_spell_cast_during_your_main_phase() {
    cr!("207.2c", "505.1");
    supported("Sphinx's Insight");
    supported("Arrester's Admonition");
    // During your main phase: the bonus applies.
    let mut t = TestGame::new(2);
    cast_insight(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 22);
    // During your combat phase: it doesn't.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    cast_insight(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 20);
    // During an opponent's main phase: it doesn't either ("your main phase").
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Arrester's Admonition");
    // "Return target creature to its owner's hand. Addendum — ... draw a card."
    let c = t.hand(P0, "Arrester's Admonition");
    t.cast(P0, c).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn a_copy_of_an_addendum_spell_wasnt_cast_and_gets_no_bonus() {
    cr!("707.10");
    ruling!(
        "Sphinx's Insight",
        "If an effect copies a spell with an addendum ability while it's on the stack, the copy wasn't cast at all, so you won't get the addendum bonus."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let insight = cast_insight(&mut t, P0);
    t.lands(P0, "Island", 2);
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(insight).go();
    // Twincast, then the copy: two cards, no life.
    t.resolve();
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 20);
    // The original: two more cards and 2 life.
    t.resolve();
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn addendum_applies_as_the_spell_resolves_and_not_if_it_is_countered() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Sphinx's Insight",
        "Addendum abilities of instant spells apply while the spell is resolving, not immediately after casting it. If the spell is countered, you don't get the addendum bonus."
    );
    supported("Counterspell");
    let mut t = TestGame::new(2);
    let insight = cast_insight(&mut t, P0);
    // Nothing happened on casting.
    assert_eq!(t.life(P0), 20);
    t.lands(P1, "Island", 2);
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(insight).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sphinx's Insight"));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P0), 20);
}
