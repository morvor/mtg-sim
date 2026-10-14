//! Rulings batch P032 — spells copied by triggered abilities that trigger on casting:
//! the trigger and its copies resolve first and don't depend on the original spell still
//! being on the stack (CR 707.10, 603.3, 707.12); conditions are checked as the trigger
//! resolves (CR 608.2); "first ... each turn" counts the whole turn (CR 603.2).

use crate::r_p032_common::*;
use crate::r_p146_common::counter;
use crate::r_s01_common::supported;
use crate::r_s25_common::{cast_new, change_copy_targets, spell_copies};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn knights(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Knight"))
        .count()
}

/// P0 casts Banish into Fable from hand at P1's first Bears (of three); the cast
/// trigger is on the stack. Returns the spell and the three Bears.
fn banish(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    supported("Banish into Fable");
    t.set_step(P0, Step::PrecombatMain);
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    let spell = cast_new(t, P0, "Banish into Fable", &[obj(bears[0])]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    (spell, bears)
}

#[test]
fn banish_into_fable_copies_resolve_even_if_it_is_countered() {
    cr!("603.3", "707.10", "608.2n");
    ruling!(
        "Banish into Fable",
        "The triggered ability that copies Banish into Fable resolves and creates copies of Banish into Fable even if Banish into Fable is countered. The ability and the copies resolve before Banish into Fable itself resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    let (spell, bears) = banish(&mut t);
    counter(&mut t, spell);
    change_copy_targets(&mut t, P0, &[Some(obj(bears[1]))]);
    change_copy_targets(&mut t, P0, &[Some(obj(bears[2]))]);
    t.resolve_all();
    assert!(t.on_battlefield(bears[0]));
    assert!(!t.on_battlefield(bears[1]));
    assert!(!t.on_battlefield(bears[2]));
    assert_eq!(knights(&t), 2);

    // Not countered: both copies resolve before the original.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    let (spell, bears) = banish(&mut t);
    change_copy_targets(&mut t, P0, &[Some(obj(bears[1]))]);
    change_copy_targets(&mut t, P0, &[Some(obj(bears[2]))]);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    assert_eq!(t.g.stack[0], spell);
    t.resolve();
    t.resolve();
    assert_eq!(knights(&t), 2);
    assert!(t.on_battlefield(bears[0]));
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve();
    assert!(!t.on_battlefield(bears[0]));
    assert_eq!(knights(&t), 3);
}

#[test]
fn banish_into_fable_checks_artifacts_and_enchantments_on_resolution() {
    cr!("603.4", "608.2c");
    ruling!(
        "Banish into Fable",
        "Whether you control an artifact and/or an enchantment is checked only as the triggered ability resolves."
    );
    // Nothing at the cast; an artifact and an enchantment arrive before the trigger
    // resolves: two copies.
    let mut t = TestGame::new(2);
    let (_spell, _bears) = banish(&mut t);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    // Both at the cast, but the artifact is gone by then: one copy.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    let (_spell, _bears) = banish(&mut t);
    crate::r_s02_common::destroy(&mut t, thopter);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
}

#[test]
fn bygone_marvels_is_copied_twice_even_if_countered() {
    cr!("603.3", "707.10", "702.169a");
    ruling!(
        "Bygone Marvels",
        "The triggered ability will resolve before Bygone Marvels does. If Bygone Marvels is countered in response to its own triggered ability, you will still copy Bygone Marvels twice."
    );
    supported("Bygone Marvels");
    // Descend 8 — eight permanent cards in the graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cards: Vec<ObjectId> = (0..8).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    let spell = cast_new(&mut t, P0, "Bygone Marvels", &[obj(cards[0])]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter(&mut t, spell);
    change_copy_targets(&mut t, P0, &[Some(obj(cards[1]))]);
    change_copy_targets(&mut t, P0, &[Some(obj(cards[2]))]);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve_all();
    assert_eq!(t.zone(cards[1]), Zone::Hand(P0));
    assert_eq!(t.zone(cards[2]), Zone::Hand(P0));
    assert_eq!(t.zone(cards[0]), Zone::Graveyard(P0));
}

const PAIR: &str = "Curious Pair // Treats to Share";

fn foods(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Food"))
        .count()
}

/// P0 casts Treats to Share ("Create a Food token.", no targets) with Chancellor of
/// Tales on the battlefield, answering its "may copy" yes; the trigger is on the stack.
fn treats_with_chancellor(t: &mut TestGame) -> ObjectId {
    supported("Chancellor of Tales");
    supported(PAIR);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Chancellor of Tales");
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, PAIR);
    t.answer_yes(P0, true);
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    spell
}

#[test]
fn chancellor_of_tales_copies_an_adventure_without_targets() {
    cr!("707.10", "715.3");
    ruling!(
        "Chancellor of Tales",
        "Chancellor of Tales can copy any Adventure spell, not just one with targets."
    );
    let mut t = TestGame::new(2);
    let card = treats_with_chancellor(&mut t);
    t.resolve_all();
    assert_eq!(foods(&t), 2);
    // The card itself went on an adventure (exile); the copy simply ceased to exist.
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn chancellor_of_tales_copy_resolves_even_if_the_adventure_is_countered() {
    cr!("603.3", "707.10", "715.4");
    ruling!(
        "Chancellor of Tales",
        "Chancellor of Tales's ability and the copy it creates both resolve before the Adventure spell. They resolve even if the Adventure spell is countered before the copy is created."
    );
    let mut t = TestGame::new(2);
    let spell = treats_with_chancellor(&mut t);
    counter(&mut t, spell);
    t.resolve_all();
    assert_eq!(foods(&t), 1);
    // Countered, it went to the graveyard rather than on an adventure.
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    // Not countered: the copy resolves first.
    let mut t = TestGame::new(2);
    let spell = treats_with_chancellor(&mut t);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve();
    assert_eq!(foods(&t), 1);
    assert_eq!(t.g.stack, vec![spell]);
}

#[test]
fn owlin_spiralmancer_doesnt_trigger_on_a_second_x_spell() {
    cr!("603.2", "603.2h");
    ruling!(
        "Owlin Spiralmancer",
        "If you cast your first spell with {X} in its mana cost during a turn before Owlin Spiralmancer is on the battlefield, casting another spell with {X} in its mana cost later in the turn won't cause its last ability to trigger."
    );
    supported("Owlin Spiralmancer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).x(0).go();
    t.resolve_all();
    t.battlefield(P0, "Owlin Spiralmancer");
    t.lands(P0, "Wastes", 2);
    let two = t.hand(P0, "Endless One");
    t.answer_yes(P0, true);
    t.cast(P0, two).x(2).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "Owlin Spiralmancer triggered");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Endless One").len(), 1);
    // On a later turn, the first {X} spell does trigger it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Owlin Spiralmancer");
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Endless One");
    t.answer_yes(P0, true);
    t.cast(P0, card).x(2).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn labyrinth_guardian_is_sacrificed_even_if_the_spell_is_countered() {
    cr!("603.3", "608.2b", "701.21a");
    ruling!(
        "Labyrinth Guardian",
        "You'll sacrifice Labyrinth Guardian even if you counter the spell that targets it. If that spell has no other targets, it'll be countered when it tries to resolve."
    );
    supported("Labyrinth Guardian");
    // P0 Shocks P1's Labyrinth Guardian; P1 counters the Shock.
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P1, "Labyrinth Guardian");
    let shock = cast_new(&mut t, P0, "Shock", &[obj(guardian)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter(&mut t, shock);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Labyrinth Guardian"));
    // Not countered: the trigger resolves first, and the Shock has no legal target left.
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P1, "Labyrinth Guardian");
    let life = t.life(P1);
    cast_new(&mut t, P0, "Shock", &[obj(guardian)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Labyrinth Guardian"));
    assert_eq!(t.life(P1), life);
    assert!(t.in_graveyard(P0, "Shock"));
}
