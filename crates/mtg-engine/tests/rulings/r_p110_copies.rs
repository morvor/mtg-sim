//! Rulings batch P110 — copying spells and abilities on the stack (CR 707.10, 707.10c):
//! Shiko and Narset's flurry, Fire Lord Azula and Firebender Ascension put their trigger
//! and the copy on the stack above the original (CR 603.3, 405.2), and the copy is made
//! from the original's last known information if it was countered (CR 707.10, 608.2h).
//! Also: a cost can't be paid twice with one Clue (CR 118.3, 602.2b).

use crate::r_p110_common::*;
use crate::r_s01_common::attack_with;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets, targets_of};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Shiko and Narset, Unified -----------------------------------------------------------

/// P0 controls Shiko and Narset and casts two Shocks at P1: the second triggers flurry.
/// Returns the second Shock.
fn shiko_second_spell(t: &mut TestGame) -> ObjectId {
    supported("Shiko and Narset, Unified");
    t.battlefield(P0, "Shiko and Narset, Unified");
    cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let second = cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the flurry trigger is on top");
    second
}

#[test]
fn shikos_copy_resolves_first_even_if_the_original_is_countered() {
    cr!("707.10", "603.3", "608.2h");
    ruling!(
        "Shiko and Narset, Unified",
        "Shiko and Narset, Unified’s last ability and the copy it creates resolve before the spell it’s copying. They resolve even if the original spell is countered before the copy is created."
    );
    // The copy resolves before the original.
    let mut t = TestGame::new(2);
    let second = shiko_second_spell(&mut t);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.g.stack, vec![second], "the original is still waiting");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Countered in response to the trigger: the copy is still created and resolves.
    let mut t = TestGame::new(2);
    let second = shiko_second_spell(&mut t);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(second).go();
    t.resolve();
    assert!(!t.g.stack.contains(&second));
    let hand = t.hand_size(P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "the copy dealt 2");
    assert_eq!(t.hand_size(P0), hand, "a spell was copied: no card drawn");
}

#[test]
fn shikos_copy_may_change_any_number_of_targets() {
    cr!("707.10c", "115.7");
    ruling!(
        "Shiko and Narset, Unified",
        "The copy will have the same targets unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    // Keeping the target.
    let mut t = TestGame::new(2);
    shiko_second_spell(&mut t);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Player(P1)]);
    // Changing it to P1's Grizzly Bears; only legal targets are offered (no enchantment:
    // Shock's target is "any target").
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let enchantment = t.battlefield(P1, "Propaganda");
    shiko_second_spell(&mut t);
    change_copy_targets(&mut t, P0, &[Some(obj(bear))]);
    let from = t.asked().len();
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![obj(bear)]);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().all(|c| !c.contains(&obj(enchantment))));
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.life(P1), 16);
}

// --- Fire Lord Azula ---------------------------------------------------------------------

#[test]
fn azulas_copy_resolves_first_even_if_the_spell_is_countered() {
    cr!("707.10", "603.3", "608.2h");
    ruling!(
        "Fire Lord Azula",
        "Azula's ability and the copy it creates both resolve before the spell that caused the ability to trigger. They resolve even if the spell is countered before the copy is created."
    );
    supported("Fire Lord Azula");
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        let azula = t.battlefield(P0, "Fire Lord Azula");
        attack_with(&mut t, &[(azula, Entity::Player(P1))]);
        // Firebending 2's {R}{R}.
        t.resolve_all();
        assert_eq!(pool_of(&t, P0, ManaType::R), 2);
        let card = t.hand(P0, "Shock");
        let shock = t.cast(P0, card).target(Entity::Player(P1)).go();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if counter {
            t.lands(P1, "Island", 2);
            let cs = t.hand(P1, "Counterspell");
            t.cast(P1, cs).target(shock).go();
            t.resolve();
            assert!(!t.g.stack.contains(&shock));
        }
        keep_copy_targets(&mut t, P0);
        t.resolve();
        assert_eq!(spell_copies(&t).len(), 1, "counter={counter}");
        t.resolve();
        assert_eq!(
            t.life(P1),
            18,
            "the copy resolved first (counter={counter})"
        );
        t.resolve_all();
        assert_eq!(t.life(P1), if counter { 18 } else { 16 });
    }
}

// --- Firebender Ascension ----------------------------------------------------------------

#[test]
fn firebender_ascensions_copy_resolves_before_the_original_trigger() {
    cr!("603.2", "603.3", "707.10", "702.189a");
    ruling!(
        "Firebender Ascension",
        "Firebender Ascension's ability will always go on the stack on top of the triggered ability that caused it to trigger. The copy it creates will be created on the stack on top of the original triggered ability and the copy will resolve before that ability."
    );
    supported("Firebender Ascension");
    supported("Zuko, Exiled Prince");
    // Three quest counters already; Zuko (firebending 3) attacks.
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Firebender Ascension");
    t.g.objects[asc.0 as usize]
        .counters
        .insert("quest".into(), 3);
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    attack_with(&mut t, &[(zuko, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(abilities_from(&t, asc), vec![top], "the Ascension's on top");
    let firebending = abilities_from(&t, zuko);
    assert_eq!(firebending.len(), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(asc, "quest"), 4);
    // The copy is on top of the original firebending trigger.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], firebending[0]);
    t.resolve();
    assert_eq!(pool_of(&t, P0, ManaType::R), 3);
    assert_eq!(t.g.stack, firebending);
    t.resolve();
    assert_eq!(pool_of(&t, P0, ManaType::R), 6);
}

#[test]
fn firebender_ascension_counts_only_triggered_abilities() {
    cr!("113.3c", "603.1", "603.2");
    ruling!(
        "Firebender Ascension",
        "Triggered abilities use the words \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\""
    );
    // An attacking Zuko: its firebending trigger counts; activating its "{3}: Exile the
    // top card of your library" while attacking doesn't; a creature attacking with no
    // attack trigger (Grizzly Bears) doesn't.
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Firebender Ascension");
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    let bear = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(zuko, Entity::Player(P1)), (bear, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.counters(asc, "quest"), 1);
    t.library_top(P0, "Grizzly Bears");
    act(&mut t, P0, zuko, "Exile the top card", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(asc, "quest"), 1);
}

// --- Lonis, Cryptozoologist --------------------------------------------------------------

#[test]
fn one_clue_cant_pay_for_two_abilities() {
    cr!("118.3", "602.2b", "701.21a");
    ruling!(
        "Lonis, Cryptozoologist",
        "You can't sacrifice a Clue to activate its own ability and also to activate the ability of Lonis, Cryptozoologist."
    );
    supported("Lonis, Cryptozoologist");
    // P0's one Clue pays for its own ability: Lonis can't sacrifice it as well.
    let mut t = TestGame::new(2);
    let lonis = t.battlefield(P0, "Lonis, Cryptozoologist");
    let clue = create_token(&mut t, P0, "Clue");
    mana(&mut t, P0, ManaType::C, 2);
    act(&mut t, P0, clue, "Draw", &[]).unwrap();
    assert!(!t.is_live(clue));
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    assert!(act(
        &mut t,
        P0,
        lonis,
        "Sacrifice X Clues",
        &[Entity::Player(P1)]
    )
    .is_err());
    t.resolve_all();
    // Sacrificed for Lonis, the Clue's own ability is gone.
    let mut t = TestGame::new(2);
    let lonis = t.battlefield(P0, "Lonis, Cryptozoologist");
    let clue = create_token(&mut t, P0, "Clue");
    t.library_top(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[obj(clue)]);
    act(
        &mut t,
        P0,
        lonis,
        "Sacrifice X Clues",
        &[Entity::Player(P1)],
    )
    .unwrap();
    assert!(!t.is_live(clue));
    mana(&mut t, P0, ManaType::C, 2);
    assert!(!can_activate(&mut t, P0, clue));
}
