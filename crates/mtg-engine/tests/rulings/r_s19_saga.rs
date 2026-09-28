//! Rulings on Sagas (CR 714): chapter abilities and lore counters, the order and targets
//! of chapter abilities that trigger together, chapter abilities on the stack, the
//! sacrifice after the final chapter, removing lore counters, and Saga creatures.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s19_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::saga;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A Saga `name` for P0 on the battlefield without lore counters, with a creature for
/// each player (targets for chapter abilities).
fn blank_saga(t: &mut TestGame, name: &str) -> ObjectId {
    supported(name);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let s = t.battlefield(P0, name);
    t.g.objects[s.0 as usize].counters.clear();
    s
}

/// P1 casts Stifle ("Counter target activated or triggered ability.") on the top object
/// of the stack.
fn stifle_top(t: &mut TestGame) {
    let top = *t.g.stack.last().expect("something to counter");
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(top).go();
    t.resolve();
}

/// Removing a lore counter doesn't trigger anything; putting it back triggers chapter II
/// again. Returns the chapters that triggered each time.
fn remove_and_readd(name: &str) -> Vec<Vec<u32>> {
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, name);
    let mut seen = Vec::new();
    add_lore(&mut t, s, 1);
    seen.push(chapters_on_stack(&t, s));
    t.resolve_all();
    add_lore(&mut t, s, 1);
    seen.push(chapters_on_stack(&t, s));
    t.resolve_all();
    remove_lore(&mut t, s, 1);
    assert_eq!(lore(&t, s), 1);
    seen.push(chapters_on_stack(&t, s));
    add_lore(&mut t, s, 1);
    seen.push(chapters_on_stack(&t, s));
    t.resolve_all();
    assert!(t.on_battlefield(s));
    seen
}

const READDED: [&[u32]; 4] = [&[1], &[2], &[], &[2]];

#[test]
fn removing_lore_counters_lets_a_chapter_trigger_again() {
    cr!("714.2b", "714.2c");
    ruling!(
        "Summon: Knights of Round",
        "Removing lore counters won't cause a previous chapter ability to trigger. If lore counters are removed from a Saga, the appropriate chapter abilities will trigger again when the Saga receives more lore counters."
    );
    ruling!(
        "Arni Slays the Troll",
        "Removing lore counters won’t cause a previous chapter ability to trigger. If lore counters are removed from a Saga, the appropriate chapter abilities will trigger again when the Saga receives more lore counters."
    );
    ruling!(
        "The Birth of Meletis",
        "Removing lore counters won’t cause a previous chapter ability to trigger. If lore counters are removed from a Saga, the appropriate chapter abilities will trigger again when the Saga receives lore counters."
    );
    ruling!(
        "The First Iroan Games",
        "Removing lore counters won't cause a previous chapter ability to trigger. If lore counters are removed from a Saga, the appropriate chapter abilities will trigger again when the Saga receives lore counters."
    );
    ruling!(
        "Time of Ice",
        "If counters are removed from a Saga, the appropriate chapter abilities will trigger again when the Saga receives lore counters. Removing lore counters won't cause a previous chapter ability to trigger."
    );
    ruling!(
        "History of Benalia",
        "If counters are removed from a Saga, the appropriate chapter abilities will trigger again when the Saga receives lore counters. Removing lore counters won’t cause a previous chapter ability to trigger."
    );
    for name in [
        "Summon: Knights of Round",
        "Arni Slays the Troll",
        "The Birth of Meletis",
        "The First Iroan Games",
        "Time of Ice",
        "History of Benalia",
    ] {
        let seen = remove_and_readd(name);
        assert_eq!(seen, READDED.map(|x| x.to_vec()).to_vec(), "{name}");
    }
    // Summon: Knights of Round ("I, II, III, IV — Create three 2/2 white Knight creature
    // tokens."): chapter II's tokens twice.
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "Summon: Knights of Round");
    add_lore(&mut t, s, 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Knight").len(), 1 + 6);
    remove_lore(&mut t, s, 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Knight").len(), 1 + 6);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Knight").len(), 1 + 9);
}

/// A Saga at two lore counters gets a third: only chapter III triggers.
fn third_counter(name: &str) -> Vec<u32> {
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, name);
    t.graveyard(P1, "Hill Giant");
    add_lore(&mut t, s, 2);
    assert_eq!(chapters_on_stack(&t, s).len(), 2, "{name}");
    t.resolve_all();
    add_lore(&mut t, s, 1);
    chapters_on_stack(&t, s)
}

#[test]
fn a_chapter_doesnt_trigger_again_for_a_later_lore_counter() {
    cr!("714.2b");
    ruling!(
        "Binding the Old Gods",
        "A chapter ability doesn't trigger if a lore counter is put on a Saga that already had a number of lore counters greater than or equal to that chapter's number. For example, the third lore counter put on a Saga causes the chapter III ability to trigger, but chapters I and II won't trigger again."
    );
    ruling!(
        "The Eldest Reborn",
        "A chapter ability doesn't trigger if a lore counter is put on a Saga that already had a number of lore counters greater than or equal to that chapter's number. For example, the third lore counter put on a Saga causes the III chapter ability to trigger, but I and II won't trigger again."
    );
    ruling!(
        "Showdown of the Skalds",
        "A chapter ability doesn’t trigger if a lore counter is put on a Saga that already had a number of lore counters greater than or equal to that chapter’s number. For example, the third lore counter put on a Saga causes the chapter III ability to trigger, but chapters I and II won’t trigger again."
    );
    for name in [
        "Binding the Old Gods",
        "The Eldest Reborn",
        "Showdown of the Skalds",
    ] {
        assert_eq!(third_counter(name), vec![3], "{name}");
    }
}

#[test]
fn chapters_that_trigger_together_are_ordered_and_targeted_as_they_go_on_the_stack() {
    cr!("603.3b", "714.2b", "601.2c");
    ruling!(
        "Binding the Old Gods",
        "If multiple chapter abilities trigger at the same time, their controller puts them on the stack in any order. If any of them require targets, those targets are chosen as you put the abilities on the stack, before any of those abilities resolve."
    );
    // Binding the Old Gods: "I — Destroy target nonland permanent an opponent controls.
    // II — Search your library for a Forest card, put it onto the battlefield tapped, then
    // shuffle."
    let mut orders = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let s = blank_saga(&mut t, "Binding the Old Gods");
        let anthem = t.battlefield(P1, "Glorious Anthem");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.answer_targets(P0, &[Entity::Object(anthem)]);
        add_lore(&mut t, s, 2);
        // Both are on the stack, chapter I with its target chosen, neither resolved.
        let on_stack = chapters_on_stack(&t, s);
        assert_eq!(on_stack.len(), 2);
        let chapter_1 = t
            .g
            .stack
            .iter()
            .copied()
            .find(|id| {
                t.g.obj(*id)
                    .stack
                    .as_ref()
                    .and_then(|si| si.event.as_ref())
                    .is_some_and(|e| e.amount == 1)
            })
            .unwrap();
        let si = t.g.obj(chapter_1).stack.as_ref().unwrap();
        assert_eq!(si.chosen[0].targets, vec![vec![Entity::Object(anthem)]]);
        assert!(t.on_battlefield(anthem));
        t.resolve_all();
        assert!(!t.on_battlefield(anthem));
        orders.push(on_stack);
    }
    // The controller chose the order: both orders are possible.
    orders.sort();
    assert_eq!(orders, vec![vec![1, 2], vec![2, 1]]);
}

#[test]
fn a_chapter_ability_on_the_stack_is_independent_of_the_saga() {
    cr!("113.7a", "714.2b");
    ruling!(
        "The Eldest Reborn",
        "Once a chapter ability has triggered, the ability on the stack won't be affected if the Saga gains or loses counters, or if it leaves the battlefield."
    );
    // The Eldest Reborn: "I — Each opponent sacrifices a creature or planeswalker of their
    // choice." The Saga is destroyed with chapter I on the stack.
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "The Eldest Reborn");
    add_lore(&mut t, s, 1);
    assert_eq!(chapters_on_stack(&t, s), vec![1]);
    remove_lore(&mut t, s, 1);
    destroy(&mut t, s);
    assert!(!t.on_battlefield(s));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Gaining counters doesn't change it either (chapter I resolves once, then II).
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "The Eldest Reborn");
    t.hand(P1, "Island");
    add_lore(&mut t, s, 1);
    add_lore(&mut t, s, 1);
    assert_eq!(chapters_on_stack(&t, s), vec![1, 2]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Island"));
}

#[test]
fn a_chapter_ability_on_the_stack_is_independent_of_the_saga_curly_quotes() {
    cr!("113.7a", "714.2b");
    ruling!(
        "Kiora Bests the Sea God",
        "Once a chapter ability has triggered, the ability on the stack won’t be affected if the Saga gains or loses counters, or if it leaves the battlefield."
    );
    // Kiora Bests the Sea God: "I — Create an 8/8 blue Kraken creature token with
    // hexproof." The Saga is bounced with chapter I on the stack.
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "Kiora Bests the Sea God");
    add_lore(&mut t, s, 1);
    assert_eq!(chapters_on_stack(&t, s), vec![1]);
    t.g.move_object(s, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Kraken").len(), 1);
}

#[test]
fn a_chapter_ability_uses_the_stack_and_can_be_countered() {
    cr!("714.2b", "603.3", "701.6b");
    ruling!(
        "Summon: Primal Odin",
        "Chapter abilities are put onto the stack and may be responded to."
    );
    supported("Stifle");
    // Summon: Primal Odin: "I — Gungnir — Destroy target creature an opponent controls."
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "Summon: Primal Odin");
    let theirs = t.named_on_battlefield("Grizzly Bears")[1];
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    add_lore(&mut t, s, 1);
    assert_eq!(chapters_on_stack(&t, s), vec![1]);
    stifle_top(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(theirs));
}

#[test]
fn a_chapter_ability_uses_the_stack_and_can_be_countered_curly_quotes() {
    cr!("714.2b", "603.3", "701.6b");
    ruling!(
        "Kiora Bests the Sea God",
        "Each symbol on the left of a Saga’s text box represents a chapter ability. A chapter ability is a triggered ability that triggers when a lore counter that is put on the Saga causes the number of lore counters on the Saga to become equal to or greater than the ability’s chapter number. Chapter abilities are put onto the stack and may be responded to."
    );
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "Kiora Bests the Sea God");
    // Two counters at once: chapters I and II trigger (the count became at least 1 and 2).
    add_lore(&mut t, s, 2);
    assert_eq!(chapters_on_stack(&t, s).len(), 2);
    // Chapter I ("Create an 8/8 blue Kraken") is countered; chapter II still resolves.
    for _ in 0..2 {
        if chapters_on_stack(&t, s).last() == Some(&1) {
            break;
        }
        t.resolve();
    }
    assert_eq!(chapters_on_stack(&t, s), vec![1]);
    stifle_top(&mut t);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Kraken").is_empty());
}

#[test]
fn the_saga_is_sacrificed_once_its_final_chapter_has_left_the_stack() {
    cr!("714.4", "704.5s");
    ruling!(
        "Binding the Old Gods",
        "Once the number of lore counters on a Saga is greater than or equal to the greatest number among its chapter abilities, the Saga's controller sacrifices it as soon as its chapter ability has left the stack, most likely by resolving or being countered. This state-based action doesn't use the stack."
    );
    ruling!(
        "The Birth of Meletis",
        "Once the number of lore counters on a Saga is greater than or equal to the greatest number among its chapter abilities, the Saga’s controller sacrifices it as soon as its chapter ability has left the stack, most likely by resolving or being countered. This state-based action doesn’t use the stack."
    );
    ruling!(
        "The Eldest Reborn",
        "Once the number of lore counters on a Saga is greater than or equal to the greatest number among its chapter abilities—in the Dominaria set, this is always three—the Saga's controller sacrifices it as soon as its chapter ability has left the stack, most likely by resolving or being countered. This state-based action doesn't use the stack."
    );
    // Resolving: Binding the Old Gods' chapter III ("Creatures you control gain deathtouch
    // until end of turn.").
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "Binding the Old Gods");
    add_lore(&mut t, s, 2);
    t.resolve_all();
    add_lore(&mut t, s, 1);
    assert_eq!(chapters_on_stack(&t, s), vec![3]);
    assert!(t.on_battlefield(s));
    t.resolve();
    assert!(!t.on_battlefield(s));
    assert!(t.in_graveyard(P0, "Binding the Old Gods"));
    assert_eq!(t.stack_len(), 0);
    // Countered: The Birth of Meletis' chapter III ("You gain 2 life.") and The Eldest
    // Reborn's.
    for name in ["The Birth of Meletis", "The Eldest Reborn"] {
        let mut t = TestGame::new(2);
        let s = blank_saga(&mut t, name);
        add_lore(&mut t, s, 2);
        t.resolve_all();
        add_lore(&mut t, s, 1);
        assert_eq!(chapters_on_stack(&t, s), vec![3]);
        stifle_top(&mut t);
        assert!(!t.on_battlefield(s), "{name}");
        assert_eq!(t.stack_len(), 0);
        assert_eq!(t.life(P0), 20);
    }
}

#[test]
fn a_saga_gets_lore_counters_as_it_enters_and_as_the_first_main_phase_begins() {
    cr!("714.3a", "714.3c", "505.4");
    ruling!(
        "Summon: Ixion",
        "As a Saga enters, its controller puts a lore counter on it. As your first main phase begins (immediately after your draw step), you put another lore counter on each Saga you control. Putting a lore counter on a Saga in either of these ways doesn't use the stack."
    );
    ruling!(
        "The First Iroan Games",
        "As a Saga enters the battlefield, its controller puts a lore counter on it. As your precombat main phase begins (immediately after your draw step), you put another lore counter on each Saga you control. Putting a lore counter on a Saga in either of these ways doesn't use the stack."
    );
    for (name, chapter_1_target) in [("Summon: Ixion", true), ("The First Iroan Games", false)] {
        supported(name);
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P0, "Grizzly Bears");
        if chapter_1_target {
            t.answer_targets(P0, &[Entity::Object(theirs)]);
        }
        let card = in_hand_with_mana(&mut t, P0, name);
        let spell = t.cast(P0, card).go();
        t.resolve();
        let s = t.g.current(spell);
        // It entered with a lore counter; only chapter I is on the stack.
        assert_eq!(lore(&t, s), 1, "{name}");
        assert_eq!(chapters_on_stack(&t, s), vec![1], "{name}");
        t.resolve_all();
        // The next turn: the counter is put on as the first main phase begins; chapter II
        // triggers, and nothing else was put on the stack for it.
        t.set_step(P0, Step::Draw);
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(lore(&t, s), 2, "{name}");
        t.settle();
        assert_eq!(chapters_on_stack(&t, s), vec![2], "{name}");
        assert_eq!(t.stack_len(), 1, "{name}");
    }
}

#[test]
fn a_saga_without_chapter_abilities_isnt_sacrificed_or_given_lore_counters() {
    cr!("714.2d", "714.3c", "714.4");
    ruling!(
        "Summon: Titan",
        "a Saga that somehow loses all of its chapter abilities will not be sacrificed as a state-based action. It will also not gain a lore counter at the beginning of each of its controller's first main phases."
    );
    supported("Frogify");
    // Frogify: "Enchanted creature loses all abilities and is a blue Frog creature with
    // base power and toughness 1/1."
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, "Summon: Titan");
    t.g.objects[s.0 as usize]
        .counters
        .insert(counters::LORE.into(), 3);
    let frogify = t.battlefield(P0, "Frogify");
    t.g.attach(frogify, Entity::Object(s));
    t.g.recompute();
    assert_eq!(saga::final_chapter(t.obj(s)), None);
    t.settle();
    assert!(t.on_battlefield(s));
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert!(t.on_battlefield(s));
    assert_eq!(lore(&t, s), 3);
    assert_eq!(t.stack_len(), 0);
    // Without Frogify, it has its chapter abilities again: sacrificed.
    destroy(&mut t, frogify);
    t.settle();
    assert!(!t.on_battlefield(s));
}

#[test]
fn a_saga_creatures_other_abilities_apply_at_any_lore_count() {
    cr!("714.1a");
    ruling!(
        "Summon: Titan",
        "Any abilities in the latter section aren't chapter abilities and apply no matter how many lore counters are on the creature."
    );
    // Summon: Titan: reach and trample, below its chapters; Summon: Ixion: first strike.
    let mut t = TestGame::new(2);
    let titan = blank_saga(&mut t, "Summon: Titan");
    let ixion = t.battlefield(P0, "Summon: Ixion");
    t.g.objects[ixion.0 as usize].counters.clear();
    for n in 0..3u32 {
        t.g.recompute();
        let c = &t.obj(titan).chars;
        assert!(c.has_keyword(KeywordKind::Reach) && c.has_keyword(KeywordKind::Trample));
        assert!(t.obj(ixion).chars.has_keyword(KeywordKind::FirstStrike));
        assert_eq!(saga::final_chapter(t.obj(titan)), Some(3));
        for id in [titan, ixion] {
            t.g.objects[id.0 as usize]
                .counters
                .insert(counters::LORE.into(), n + 1);
        }
    }
    // They aren't chapter abilities: no chapter number counts them.
    let mut chapters = saga::chapter_numbers(t.obj(titan));
    chapters.sort();
    assert_eq!(chapters, vec![1, 2, 3]);
}
