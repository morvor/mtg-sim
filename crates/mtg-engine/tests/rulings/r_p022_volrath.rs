//! Rulings batch P022 — Volrath, the Shapestealer ("{1}: Until your next turn, Volrath
//! becomes a copy of target creature with a counter on it, except it's 7/5 and it has this
//! ability"): exceptions that set power and toughness (CR 707.9b, 707.9d), face-down and
//! copying creatures (CR 707.2, 707.3, 708.2), the legend rule (CR 704.5j), summoning
//! sickness (CR 302.6), "activate only once each turn" (CR 602.5b), +1/+1 and -1/-1
//! counters (CR 704.5q), effects that already applied (CR 613.1) and copies of Volrath
//! (CR 707.9b).

use crate::r_p022_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const VOLRATH: &str = "Volrath, the Shapestealer";

/// P0's Volrath on the battlefield (not summoning sick), with mana for `n` activations.
fn volrath(t: &mut TestGame, n: usize) -> ObjectId {
    supported(VOLRATH);
    let v = t.battlefield(P0, VOLRATH);
    t.lands(P0, "Wastes", n);
    v
}

/// Activates Volrath's copy ability targeting `what` and resolves it.
fn volrath_copies(t: &mut TestGame, v: ObjectId, what: ObjectId) {
    let i = activated_index(t, v, "becomes a copy");
    t.activate(P0, v, i, &[obj(what)]).expect("Volrath's ability");
    t.resolve_all();
}

#[test]
fn volrath_doesnt_copy_a_characteristic_defining_power_ability() {
    cr!("707.9d", "604.3");
    ruling!("Volrath, the Shapestealer", "If Volrath becomes a copy of a creature whose power and toughness are defined by an ability (usually printed as */* or similar), Volrath doesn't copy the ability that defines its power and toughness. Volrath remains a 7/5 creature.");
    supported("Nightmare");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    // Nightmare: flying; its P/T are the number of Swamps its controller controls (five).
    t.lands(P0, "Swamp", 5);
    let nightmare = t.battlefield(P0, "Nightmare");
    minus1(&mut t, nightmare);
    assert_eq!(t.pt(nightmare), (4, 4));
    volrath_copies(&mut t, v, nightmare);
    assert_eq!(name_of(&t, v), "Nightmare");
    assert!(has_kw(&t, v, KeywordKind::Flying));
    assert_eq!(t.pt(v), (7, 5));
    assert_eq!(abilities_with(&t, v, "Swamps"), 0);
}

#[test]
fn volrath_copying_a_face_down_creature_is_a_face_up_7_5() {
    cr!("707.2", "708.2", "707.9b");
    ruling!("Volrath, the Shapestealer", "If Volrath becomes a copy of a face-down creature, Volrath becomes a 7/5 creature with no color, cost, or creature type. Volrath is still face up.");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    let fd = crate::r_s12_common::morph(&mut t, P0, "Den Protector");
    minus1(&mut t, fd);
    volrath_copies(&mut t, v, fd);
    let o = t.obj_now(v);
    assert!(!o.face_down);
    assert!(o.chars.name.is_empty(), "{:?}", o.chars.name);
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.subtypes.is_empty());
    assert!(o
        .chars
        .mana_cost
        .as_ref()
        .is_none_or(|m| m.mana_value() == 0));
    assert_eq!(t.pt(v), (7, 5));
    // It has "this ability" (CR 707.9a).
    assert_eq!(abilities_with(&t, v, "becomes a copy"), 1);
}

#[test]
fn volrath_copying_a_legendary_creature_you_control_meets_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!("Volrath, the Shapestealer", "If Volrath becomes a copy of a legendary creature you control, you'll put one of them into your graveyard.");
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    minus1(&mut t, isamaru);
    t.answer_choose(P0, &[obj(isamaru)]);
    volrath_copies(&mut t, v, isamaru);
    assert!(t.on_battlefield(isamaru));
    assert!(!t.on_battlefield(v));
    assert!(t.in_graveyard(P0, VOLRATH));
}

#[test]
fn volrath_copying_a_clone_copies_what_it_copies() {
    cr!("707.3", "707.9b");
    ruling!("Volrath, the Shapestealer", "If Volrath copies a creature that's copying something else, it will become whatever the target is copying.");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    let clone = cloned_angel(&mut t, P1);
    minus1(&mut t, clone);
    volrath_copies(&mut t, v, clone);
    assert_angel(&t, v, false);
    assert_eq!(t.pt(v), (7, 5));
}

#[test]
fn volrath_copying_the_turn_it_entered_can_attack_only_with_haste() {
    cr!("302.6", "702.10b", "707.2");
    ruling!("Volrath, the Shapestealer", "If Volrath copies a creature the same turn Volrath enters the battlefield, you can't attack with it or use any of its {T} abilities (if it gains any) unless the copied creature has haste.");
    supported("Raging Goblin");
    supported("Llanowar Elves");
    // A copy of Llanowar Elves ("{T}: Add {G}") can't attack or tap for mana.
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    t.g.objects[v.0 as usize].summoning_sick = true;
    let elves2 = t.battlefield(P1, "Llanowar Elves");
    plus1(&mut t, elves2, 1);
    volrath_copies(&mut t, v, elves2);
    assert_eq!(name_of(&t, v), "Llanowar Elves");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, v));
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let i = activated_index(&t, v, "Add {G}");
    assert!(t.activate(P0, v, i, &[]).is_err());
    // A copy of Raging Goblin (haste) can attack.
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    t.g.objects[v.0 as usize].summoning_sick = true;
    let goblin = t.battlefield(P1, "Raging Goblin");
    plus1(&mut t, goblin, 1);
    volrath_copies(&mut t, v, goblin);
    assert_eq!(name_of(&t, v), "Raging Goblin");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, v));
}

#[test]
fn volrath_can_activate_a_once_each_turn_ability_again_after_copying_again() {
    cr!("707.2");
    ruling!("Volrath, the Shapestealer", "If Volrath copies a creature with an ability that can be activated only once each turn (such as Chainer, Nightmare Adept), you can activate that ability once. If Volrath then becomes a copy of that same creature, you can activate that ability another time, and so on.");
    supported("Frilled Sandwalla");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 2);
    t.lands(P0, "Forest", 6);
    let sandwalla = t.battlefield(P1, "Frilled Sandwalla");
    plus1(&mut t, sandwalla, 1);
    volrath_copies(&mut t, v, sandwalla);
    assert_eq!(name_of(&t, v), "Frilled Sandwalla");
    let pump = |t: &mut TestGame| {
        let i = activated_index(t, v, "+2/+2");
        let r = t.activate(P0, v, i, &[]);
        t.resolve_all();
        r.is_ok()
    };
    assert!(pump(&mut t));
    assert_eq!(t.pt(v), (9, 7));
    assert!(!pump(&mut t), "only once each turn");
    // Volrath becomes a copy of the Sandwalla again: a new ability, activated once more.
    volrath_copies(&mut t, v, sandwalla);
    assert!(pump(&mut t));
    assert_eq!(t.pt(v), (11, 9));
    assert!(!pump(&mut t));
}

#[test]
fn volrath_counters_annihilate() {
    cr!("704.5q");
    ruling!("Volrath, the Shapestealer", "If a creature has +1/+1 counters and -1/-1 counters on it, state-based actions remove the same number of each so that it has only one of those kinds of counters on it. This may cause the creature to no longer have any counters on it.");
    let mut t = TestGame::new(2);
    volrath(&mut t, 0);
    // Volrath's beginning-of-combat trigger puts a -1/-1 counter on P1's Hill Giant with
    // two +1/+1 counters, then on P1's Grizzly Bears with one.
    let giant = t.battlefield(P1, "Hill Giant");
    plus1(&mut t, giant, 2);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, counters::MINUS1), 0);
    assert_eq!(t.pt(giant), (4, 4));
    let bears = t.battlefield(P1, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    minus1(&mut t, bears);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    assert!(t.obj_now(bears).counters.values().all(|n| *n == 0));
}

#[test]
fn volrath_keeps_effects_that_applied_before_it_became_a_copy() {
    cr!("613.1a", "613.4c", "707.2");
    ruling!("Volrath, the Shapestealer", "If an effect begins to apply to Volrath before it becomes a copy of another creature, that effect will continue to apply.");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    giant_growth(&mut t, P0, v);
    assert_eq!(t.pt(v), (10, 8));
    let bears = t.battlefield(P1, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    volrath_copies(&mut t, v, bears);
    assert_eq!(name_of(&t, v), "Grizzly Bears");
    assert_eq!(t.pt(v), (10, 8));
}

#[test]
fn a_copy_of_volrath_has_its_ability_and_is_7_5() {
    cr!("707.9b", "707.3");
    ruling!("Volrath, the Shapestealer", "If another object becomes a copy of Volrath, it has the ability to become a copy of something else and it's 7/5.");
    let mut t = TestGame::new(2);
    let v = volrath(&mut t, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    volrath_copies(&mut t, v, bears);
    // P1's Clone enters as a copy of Volrath: a 7/5 Grizzly Bears with Volrath's ability.
    let clone = clone_of(&mut t, P1, v);
    assert_eq!(name_of(&t, clone), "Grizzly Bears");
    assert_eq!(t.pt(clone), (7, 5));
    assert_eq!(abilities_with(&t, clone, "becomes a copy"), 1);
}
