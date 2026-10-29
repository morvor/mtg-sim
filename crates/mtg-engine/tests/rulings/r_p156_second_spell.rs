//! Rulings batch P156 — "whenever you cast your second spell each turn" and "if you've
//! cast two or more spells this turn": every spell cast this turn counts (resolved,
//! countered or still on the stack), including ones cast before the permanent entered and
//! the card itself; the trigger happens once per turn, only from the battlefield, and
//! resolves before the second spell.

use crate::r_s01_common::*;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// How the first spell of the turn has fared by the time the second is cast.
#[derive(Clone, Copy, Debug)]
enum First {
    Resolved,
    Countered,
    OnStack,
}

/// `p` casts their first spell of the turn and leaves it in the state `how`.
fn first_spell(t: &mut TestGame, p: PlayerId, how: First) {
    match how {
        First::Resolved => {
            cast_new(t, p, "Grizzly Bears", &[]);
            t.resolve_all();
        }
        First::Countered => {
            let s = cast_new(t, p, "Grizzly Bears", &[]);
            assert!(t.g.counter(s, None));
            t.g.flush_events();
            t.settle();
        }
        First::OnStack => {
            // An instant, so that another spell can be cast while it waits.
            cast_new(t, p, "Dark Ritual", &[]);
            t.settle();
        }
    }
}

/// Casts an instant (with nothing else to do) and puts any triggers on the stack.
fn cast_instant(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let s = cast_new(t, p, "Dark Ritual", &[]);
    t.settle();
    s
}

/// A measure of everything the "second spell" triggers of this batch do: cards taken from
/// P0's library, counters on P0's permanents, P0's tokens and damage dealt to P1.
fn progress(t: &TestGame) -> i64 {
    let lib = 100 - t.library_size(P0) as i64;
    let counters: i64 =
        t.g.permanents()
            .filter(|o| o.controller == P0)
            .map(|o| o.counters.values().map(|n| *n as i64).sum::<i64>())
            .sum();
    lib + counters + tokens(t, P0).len() as i64 + (20 - t.life(P1)) as i64
}

/// Queues answers for the cards whose trigger targets (any target: P1).
fn queue_targets(t: &mut TestGame, name: &str) {
    if name == "Pyromancer's Assault" {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
}

const TRIGGER: &str = "second spell";

#[test]
fn second_spell_triggers_once_resolve_first_whatever_the_first_spell_did() {
    cr!("603.2", "603.3b", "601.2i", "608.1");
    ruling!(
        "Jori En, Ruin Diver",
        "Jori En’s ability can trigger only once each turn. The ability will resolve before the second spell resolves. It doesn’t matter if the first spell you cast that turn has resolved, was countered, or is still on the stack."
    );
    ruling!(
        "Thunder Drake",
        "Thunder Drake’s ability can trigger only once each turn. The ability will resolve before the second spell resolves. It doesn’t matter if the first spell you cast that turn has resolved, was countered, or is still on the stack."
    );
    ruling!(
        "Pyromancer's Assault",
        "The ability can trigger only once each turn. The ability will resolve before the second spell resolves. It doesn’t matter if the first spell you cast that turn has resolved, was countered, or is still on the stack."
    );
    ruling!(
        "Firja, Judge of Valor",
        "The triggered ability can trigger only once each turn. The ability will resolve before the second spell does. It doesn't matter if the first spell you cast that turn has resolved, was countered, or is still on the stack."
    );
    for name in [
        "Jori En, Ruin Diver",
        "Thunder Drake",
        "Pyromancer's Assault",
        "Firja, Judge of Valor",
    ] {
        supported(name);
        for how in [First::Resolved, First::Countered, First::OnStack] {
            let mut t = TestGame::new(2);
            t.battlefield(P0, name);
            first_spell(&mut t, P0, how);
            assert_eq!(triggers_on_stack(&t, TRIGGER), 0, "{name} {how:?}");
            let before = progress(&t);
            queue_targets(&mut t, name);
            let second = cast_instant(&mut t, P0);
            assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name} {how:?}");
            // The trigger is above the second spell: it resolves first.
            t.resolve();
            assert_eq!(t.zone(second), mtg_engine::object::Zone::Stack, "{name}");
            assert!(progress(&t) > before, "{name} {how:?}: no effect");
            t.resolve_all();
            // A third spell doesn't trigger it again.
            cast_instant(&mut t, P0);
            assert_eq!(triggers_on_stack(&t, TRIGGER), 0, "{name} {how:?}: third");
            t.clear_answers();
        }
    }
}

#[test]
fn second_spell_trigger_resolves_even_if_that_spell_is_countered() {
    cr!("603.2", "603.3b", "701.6a");
    ruling!(
        "Illvoi Operative",
        "Illvoi Operative’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Sunstar Lightsmith",
        "Sunstar Lightsmith’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    for name in ["Illvoi Operative", "Sunstar Lightsmith"] {
        supported(name);
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        first_spell(&mut t, P0, First::Resolved);
        let second = cast_instant(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name}");
        // The second spell is countered while the trigger waits.
        assert!(t.g.counter(second, None));
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.counters(perm, "+1/+1"), 1, "{name}");
    }
}

#[test]
fn second_spell_trigger_needs_the_permanent_on_the_battlefield_as_it_is_cast() {
    cr!("603.2", "603.6a", "601.2i");
    ruling!(
        "Jori En, Ruin Diver",
        "Jori En must be on the battlefield in order for the ability to function. Notably, the ability won’t trigger if Jori En is the second spell you cast in a turn."
    );
    ruling!(
        "Thunder Drake",
        "Thunder Drake must be on the battlefield in order for the ability to function. Notably, the ability won’t trigger if Thunder Drake is the second spell you cast in a turn."
    );
    ruling!(
        "Rammas Echor, Ancient Shield",
        "If Rammas Echor was the second spell you cast in a turn, its first ability won't trigger that turn."
    );
    ruling!(
        "Monk of the Open Hand",
        "If Monk of the Open Hand is the second spell you cast in a turn, spells you cast after that point in the same turn will not cause the Flurry of Blows ability to trigger."
    );
    ruling!(
        "Firja, Judge of Valor",
        "The triggered ability triggers only if the creature that has it is on the battlefield as you cast your second spell. Spells you cast in a turn before that creature entered the battlefield will count. In other words, the ability won't trigger if the creature with the ability is the second spell you cast during a turn or if you have already cast two or more spells by the time that creature enters the battlefield that turn."
    );
    for name in [
        "Jori En, Ruin Diver",
        "Thunder Drake",
        "Rammas Echor, Ancient Shield",
        "Monk of the Open Hand",
        "Firja, Judge of Valor",
        "Illvoi Operative",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        first_spell(&mut t, P0, First::Countered);
        // The permanent is the second spell: it isn't on the battlefield as it's cast.
        cast_new(&mut t, P0, name, &[]);
        t.settle();
        assert_eq!(triggers_on_stack(&t, TRIGGER), 0, "{name}");
        t.resolve_all();
        assert!(!t.named_on_battlefield(name).is_empty(), "{name}");
        let before = progress(&t);
        // The third spell isn't the second: nothing that turn.
        cast_instant(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 0, "{name}: third spell");
        t.resolve_all();
        assert_eq!(progress(&t), before, "{name}");
        // Positive control: on the battlefield before the second spell, it triggers.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        first_spell(&mut t, P0, First::Countered);
        cast_instant(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name}: control");
    }
    // Firja, already two spells cast when it enters: the third doesn't trigger it.
    let mut t = TestGame::new(2);
    first_spell(&mut t, P0, First::Resolved);
    first_spell(&mut t, P0, First::Countered);
    t.battlefield(P0, "Firja, Judge of Valor");
    cast_instant(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, TRIGGER), 0);
}

#[test]
fn spells_cast_before_the_permanent_entered_count() {
    cr!("603.2", "601.2i");
    ruling!(
        "Brimstone Roundup",
        "Spells that were cast before Brimstone Roundup entered the battlefield count. If Brimstone Roundup was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Kraum, Violent Cacophony",
        "Spells that were cast before Kraum entered the battlefield count. If Kraum was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Rammas Echor, Ancient Shield",
        "Spells that were cast before Rammas Echor, Ancient Shield entered the battlefield count. If Rammas Echor was the first spell you cast in a turn, the next spell you cast that turn is your second spell."
    );
    ruling!(
        "Sunstar Lightsmith",
        "Sunstar Lightsmith’s ability will count any spells you’ve cast this turn, which may include Sunstar Lightsmith itself. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack. If Sunstar Lightsmith was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Illvoi Operative",
        "Illvoi Operative’s ability will count any spells you’ve cast this turn, which may include Illvoi Operative itself. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack. If Illvoi Operative was the first spell you cast this turn, the next spell you cast this turn is your second spell. Similarly, if Illvoi Operative was the second spell you cast this turn, its ability won’t trigger and it won’t get a +1/+1 counter."
    );
    ruling!(
        "Monk of the Open Hand",
        "Monk of the Open Hand doesn't need to have been on the battlefield as the first spell was cast. For example, if Monk of the Open Hand is the first spell you cast, and then you cast another spell after the Monk resolved, that spell would cause the Flurry of Blows ability to trigger."
    );
    for name in [
        "Brimstone Roundup",
        "Kraum, Violent Cacophony",
        "Rammas Echor, Ancient Shield",
        "Sunstar Lightsmith",
        "Illvoi Operative",
        "Monk of the Open Hand",
        "Firja, Judge of Valor",
    ] {
        supported(name);
        // The permanent itself is the first spell; the next one is the second.
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, name, &[]);
        t.resolve_all();
        let before = progress(&t);
        cast_instant(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name}: itself first");
        t.resolve_all();
        assert!(progress(&t) > before, "{name}: no effect");
        // A spell cast (and countered) before it was put onto the battlefield counts too.
        let mut t = TestGame::new(2);
        first_spell(&mut t, P0, First::Countered);
        t.battlefield(P0, name);
        cast_instant(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name}: earlier spell");
    }
}

#[test]
fn two_or_more_spells_this_turn_counts_every_spell_including_itself() {
    cr!("601.2i", "604.1", "614.1c", "603.4", "602.5b");
    ruling!(
        "Brightspear Zealot",
        "Brightspear Zealot’s last ability will count any spells you’ve cast this turn, which may include Brightspear Zealot itself. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    ruling!(
        "Effortless Master",
        "Effortless Master’s last ability will count any spells you’ve cast this turn, which includes Effortless Master itself if you cast it. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    ruling!(
        "Loan Shark",
        "Loan Shark’s triggered ability will count any spells you’ve cast this turn, which may include Loan Shark itself. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    ruling!(
        "Xerex Strobe-Knight",
        "Xerex Strobe-Knight will count any spells you’ve cast this turn. It doesn’t matter if those spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    for name in [
        "Brightspear Zealot",
        "Effortless Master",
        "Loan Shark",
        "Xerex Strobe-Knight",
    ] {
        supported(name);
    }
    // Brightspear Zealot: cast after a countered spell, it counts itself.
    for how in [First::Resolved, First::Countered] {
        let mut t = TestGame::new(2);
        first_spell(&mut t, P0, how);
        cast_new(&mut t, P0, "Brightspear Zealot", &[]);
        t.resolve_all();
        let z = t.named_on_battlefield("Brightspear Zealot")[0];
        assert_eq!(t.pt(z), (4, 4), "{how:?}");
    }
    // Cast first, then a spell still on the stack: it counts both.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Brightspear Zealot", &[]);
    t.resolve_all();
    let z = t.named_on_battlefield("Brightspear Zealot")[0];
    assert_eq!(t.pt(z), (2, 4));
    cast_instant(&mut t, P0);
    t.g.recompute();
    assert_eq!(t.pt(z), (4, 4));
    // Effortless Master: the countered spell and itself.
    let mut t = TestGame::new(2);
    first_spell(&mut t, P0, First::Countered);
    cast_new(&mut t, P0, "Effortless Master", &[]);
    t.resolve_all();
    let m = t.named_on_battlefield("Effortless Master")[0];
    assert_eq!(t.counters(m, "+1/+1"), 2);
    // Control: the only spell this turn.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Effortless Master", &[]);
    t.resolve_all();
    let m = t.named_on_battlefield("Effortless Master")[0];
    assert_eq!(t.counters(m, "+1/+1"), 0);
    // Loan Shark: a spell still on the stack and itself... (the Ritual resolves after).
    for (how, draws) in [(Some(First::Resolved), 1), (Some(First::Countered), 1), (None, 0)] {
        let mut t = TestGame::new(2);
        if let Some(how) = how {
            first_spell(&mut t, P0, how);
        }
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Loan Shark", &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{how:?}");
    }
    // Xerex Strobe-Knight: one countered spell and one still on the stack.
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, "Xerex Strobe-Knight");
    assert!(!crate::r_s27_common::can_activate_containing(
        &mut t, P0, x, "Knight"
    ));
    first_spell(&mut t, P0, First::Countered);
    assert!(!crate::r_s27_common::can_activate_containing(
        &mut t, P0, x, "Knight"
    ));
    cast_instant(&mut t, P0);
    assert!(crate::r_s27_common::can_activate_containing(
        &mut t, P0, x, "Knight"
    ));
}

#[test]
fn slick_sequence_checks_for_another_spell_as_it_resolves() {
    cr!("608.2c", "601.2i");
    ruling!(
        "Slick Sequence",
        "Slick Sequence checks only as it resolves whether you’ve cast another spell that turn. The other spell may be one that was countered or didn’t resolve for some reason, or it may be one you cast in response and is still on the stack."
    );
    supported("Slick Sequence");
    // Alone: no card.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Slick Sequence", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
    // Another spell cast in response (after Slick Sequence was cast), then countered.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Slick Sequence", &[Entity::Player(P1)]);
    let r = cast_instant(&mut t, P0);
    assert!(t.g.counter(r, None));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Another spell still on the stack (below it) as it resolves.
    let mut t = TestGame::new(2);
    let r = cast_instant(&mut t, P0);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Slick Sequence", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.zone(r), mtg_engine::object::Zone::Stack);
    assert_eq!(t.hand_size(P0), hand + 1);
}
