//! CR 702.177 Exhaust.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Whether `p` could activate the ability `uid` of `src` now (mana abilities included).
fn can_activate(t: &mut TestGame, p: PlayerId, src: ObjectId, uid: u64) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    let a = t
        .obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| a.uid == uid)
        .cloned()
        .unwrap();
    let AbilityKind::Activated(act) = &a.kind else {
        panic!("not an activated ability");
    };
    t.g.can_activate(p, src, &a, act)
}

#[test]
fn an_exhaust_ability_can_be_activated_only_once() {
    cr!("702.177", "702.177a");
    assert_supported("Pacesetter Paragon");
    let mut t = TestGame::new(2);
    // Pacesetter Paragon: "Exhaust — {2}{R}: Put a +1/+1 counter on this creature. It
    // gains double strike until end of turn."
    let paragon = t.battlefield(P0, "Pacesetter Paragon");
    let exhaust = ability_uid(&mut t, paragon, "Exhaust");
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(activatable(&mut t, P0, paragon, exhaust));
    activate_uid(&mut t, P0, paragon, exhaust).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(paragon, "+1/+1"), 1);
    assert!(t.obj(paragon).chars.has_keyword(KeywordKind::DoubleStrike));
    // Never again: not this turn, and not on later turns.
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(!activatable(&mut t, P0, paragon, exhaust));
    assert!(activate_uid(&mut t, P0, paragon, exhaust).is_err());
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(!activatable(&mut t, P0, paragon, exhaust));
    assert_eq!(t.counters(paragon, "+1/+1"), 1);
}

#[test]
fn exhaust_abilities_can_be_activated_any_time_you_could_activate_an_ability() {
    cr!("702.177a");
    ruling!(
        "Pacesetter Paragon",
        "Exhaust abilities can be activated any time you could normally activate an ability."
    );
    let mut t = TestGame::new(2);
    let paragon = t.battlefield(P0, "Pacesetter Paragon");
    let exhaust = ability_uid(&mut t, paragon, "Exhaust");
    // During the opponent's turn, with a spell on the stack.
    t.set_step(P1, Step::Upkeep);
    add_mana(&mut t, P0, ManaType::R, 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, bolt).target(paragon).go();
    assert!(activatable(&mut t, P0, paragon, exhaust));
    activate_uid(&mut t, P0, paragon, exhaust).unwrap();
    // The ability resolves first: a 3/3 survives the Bolt.
    t.resolve_all();
    assert!(t.on_battlefield(paragon));
    assert_eq!(t.counters(paragon, "+1/+1"), 1);
}

#[test]
fn each_exhaust_ability_of_a_permanent_is_activated_once_separately() {
    cr!("702.177a");
    assert_supported("Loot, the Pathfinder");
    let mut t = TestGame::new(2);
    // Loot: "Exhaust — {G}, {T}: Add three mana of any one color." "Exhaust — {U}, {T}:
    // Draw three cards." "Exhaust — {R}, {T}: Loot deals 3 damage to any target."
    let loot = t.battlefield(P0, "Loot, the Pathfinder");
    let draw = ability_uid(&mut t, loot, "Exhaust — {U}");
    let mana = ability_uid(&mut t, loot, "Exhaust — {G}");
    let hand = t.hand_size(P0);
    add_mana(&mut t, P0, ManaType::U, 1);
    activate_uid(&mut t, P0, loot, draw).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Untap it: the draw ability is spent, the mana ability isn't. The mana ability (an
    // exhaust ability too) can be activated once.
    t.g.untap(loot);
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::G, 1);
    assert!(!activatable(&mut t, P0, loot, draw));
    assert!(can_activate(&mut t, P0, loot, mana));
    activate_uid(&mut t, P0, loot, mana).unwrap();
    assert_eq!(pool(&t, P0), 4);
    t.g.untap(loot);
    add_mana(&mut t, P0, ManaType::G, 1);
    assert!(!can_activate(&mut t, P0, loot, mana));
    assert!(activate_uid(&mut t, P0, loot, mana).is_err());
}

#[test]
fn a_permanent_that_returns_to_the_battlefield_can_exhaust_again() {
    cr!("702.177a", "400.7");
    ruling!(
        "Pacesetter Paragon",
        "If an exhaust ability of a permanent is activated, and then that permanent leaves the battlefield and returns to the battlefield, it becomes a new object so its exhaust ability can be activated again."
    );
    let mut t = TestGame::new(2);
    let paragon = t.battlefield(P0, "Pacesetter Paragon");
    let exhaust = ability_uid(&mut t, paragon, "Exhaust");
    add_mana(&mut t, P0, ManaType::R, 3);
    activate_uid(&mut t, P0, paragon, exhaust).unwrap();
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(!activatable(&mut t, P0, paragon, exhaust));
    let again = flicker(&mut t, paragon);
    let exhaust = ability_uid(&mut t, again, "Exhaust");
    assert!(activatable(&mut t, P0, again, exhaust));
    activate_uid(&mut t, P0, again, exhaust).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(again, "+1/+1"), 1);
}

#[test]
fn whenever_you_activate_an_exhaust_ability_resolves_first() {
    cr!("702.177a");
    assert_supported("Rangers' Refueler");
    ruling!(
        "Pacesetter Paragon",
        "If an ability triggers whenever you activate an exhaust ability, that ability resolves before the exhaust ability resolves."
    );
    let mut t = TestGame::new(2);
    // Rangers' Refueler: "Whenever you activate an exhaust ability, draw a card."
    // "Exhaust — {4}: This Vehicle becomes an artifact creature. Put a +1/+1 counter on
    // it."
    let refueler = t.battlefield(P0, "Rangers' Refueler");
    let paragon = t.battlefield(P0, "Pacesetter Paragon");
    let hand = t.hand_size(P0);
    // Another ability isn't an exhaust ability.
    let crew = ability_uid(&mut t, refueler, "Crew");
    activate_uid(&mut t, P0, refueler, crew).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let exhaust = ability_uid(&mut t, paragon, "Exhaust");
    add_mana(&mut t, P0, ManaType::R, 3);
    activate_uid(&mut t, P0, paragon, exhaust).unwrap();
    t.settle();
    // The trigger is on top of the exhaust ability.
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.obj(top).stack.as_ref().unwrap().kind,
        StackKind::Triggered { .. }
    ));
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(paragon, "+1/+1"), 0);
    t.resolve_all();
    assert_eq!(t.counters(paragon, "+1/+1"), 1);
}

#[test]
fn a_vehicle_that_exhausts_stays_an_artifact_creature() {
    cr!("702.177a");
    assert_supported("Rocketeer Boostbuggy");
    let mut t = TestGame::new(2);
    let buggy = t.battlefield(P0, "Rocketeer Boostbuggy");
    assert!(!t.obj(buggy).is_creature());
    let exhaust = ability_uid(&mut t, buggy, "Exhaust");
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_uid(&mut t, P0, buggy, exhaust).unwrap();
    t.resolve_all();
    assert!(t.obj(buggy).is(CardType::Creature));
    assert_eq!(t.counters(buggy, "+1/+1"), 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj(buggy).is(CardType::Creature));
}

#[test]
fn an_effect_may_allow_exhaust_abilities_again_if_none_was_activated_this_turn() {
    cr!("702.177b");
    assert_supported("Elvish Refueler");
    let mut t = TestGame::new(2);
    // Elvish Refueler: "During your turn, as long as you haven't activated an exhaust
    // ability this turn, you may activate exhaust abilities as though they haven't been
    // activated." "Exhaust — {1}{G}: Put a +1/+1 counter on this creature."
    let paragon = t.battlefield(P0, "Pacesetter Paragon");
    let exhaust = ability_uid(&mut t, paragon, "Exhaust");
    add_mana(&mut t, P0, ManaType::R, 3);
    activate_uid(&mut t, P0, paragon, exhaust).unwrap();
    t.resolve_all();
    let elf = t.battlefield(P0, "Elvish Refueler");
    let elf_exhaust = ability_uid(&mut t, elf, "Exhaust");
    add_mana(&mut t, P0, ManaType::R, 3);
    add_mana(&mut t, P0, ManaType::G, 2);
    // This turn an exhaust ability was already activated.
    assert!(!activatable(&mut t, P0, paragon, exhaust));
    // Next turn, none has been activated yet: the spent one can be activated again.
    t.advance_to(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 3);
    // Not during an opponent's turn.
    assert!(!activatable(&mut t, P0, paragon, exhaust));
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 3);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(activatable(&mut t, P0, paragon, exhaust));
    assert!(activatable(&mut t, P0, elf, elf_exhaust));
    activate_uid(&mut t, P0, paragon, exhaust).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(paragon, "+1/+1"), 2);
    // Having activated one this turn, only exhaust abilities never activated remain.
    assert!(!activatable(&mut t, P0, paragon, exhaust));
    assert!(activatable(&mut t, P0, elf, elf_exhaust));
    activate_uid(&mut t, P0, elf, elf_exhaust).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 1);
    assert!(!activatable(&mut t, P0, elf, elf_exhaust));
}

#[test]
fn an_exhaust_ability_can_give_a_keyword_counter() {
    cr!("702.177a", "122.1b");
    assert_supported("Mai, Jaded Edge");
    // Mai, Jaded Edge: "Exhaust — {3}: Put a double strike counter on Mai."
    let mut t = TestGame::new(2);
    let mai = t.battlefield(P0, "Mai, Jaded Edge");
    let exhaust = ability_uid(&mut t, mai, "Exhaust");
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_uid(&mut t, P0, mai, exhaust).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(mai, "double strike"), 1);
    assert!(t.obj(mai).chars.has_keyword(KeywordKind::DoubleStrike));
    add_mana(&mut t, P0, ManaType::C, 3);
    assert!(!activatable(&mut t, P0, mai, exhaust));
}
