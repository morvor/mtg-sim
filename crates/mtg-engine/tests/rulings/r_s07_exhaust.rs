//! Rulings batch S07 — exhaust (CR 702.177): "Exhaust — [Cost]: [Effect]" means
//! "[Cost]: [Effect]. Activate only once."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s07_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The uid of the first activated ability of `source` whose text starts with `prefix`.
fn ability_uid(t: &mut TestGame, source: ObjectId, prefix: &str) -> u64 {
    t.g.recompute();
    let source = t.g.current(source);
    t.g.obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no activated ability starting with {prefix:?}"))
}

/// Whether `p`'s legal actions include activating the ability `uid` of `source`.
fn activatable(t: &mut TestGame, p: PlayerId, source: ObjectId, uid: u64) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    let source = t.g.current(source);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, mtg_engine::decision::Action::Activate { source: s, ability }
            if *s == source && *ability == uid)
    })
}

/// Activates the ability `uid` of `source` for `p`.
fn activate_uid(t: &mut TestGame, p: PlayerId, source: ObjectId, uid: u64) {
    t.g.recompute();
    let source = t.g.current(source);
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, source, uid).expect("activate");
    t.g.flush_events();
}

#[test]
fn a_whenever_you_activate_an_exhaust_ability_trigger_resolves_first() {
    cr!("702.177a", "603.3", "405.5");
    ruling!(
        "Mindspring Merfolk",
        "If an ability triggers whenever you activate an exhaust ability, that ability resolves before the exhaust ability resolves."
    );
    supported("Mindspring Merfolk");
    supported("Rangers' Refueler");
    // Mindspring Merfolk: "Exhaust — {X}{U}{U}, {T}: Draw X cards. Put a +1/+1 counter on
    // each Merfolk creature you control." Rangers' Refueler: "Whenever you activate an
    // exhaust ability, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rangers' Refueler");
    let merfolk = t.battlefield(P0, "Mindspring Merfolk");
    let exhaust = ability_uid(&mut t, merfolk, "Exhaust");
    add_mana(&mut t, P0, ManaType::U, 3);
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_uid(&mut t, P0, merfolk, exhaust);
    t.settle();
    // The trigger is above the exhaust ability.
    assert_eq!(t.stack_len(), 2);
    let top = top_of_stack(&t);
    assert!(matches!(
        t.obj(top).stack.as_ref().unwrap().kind,
        StackKind::Triggered { .. }
    ));
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(merfolk, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.counters(merfolk, counters::PLUS1), 1);
}

#[test]
fn a_permanent_that_returns_to_the_battlefield_is_a_new_object_that_can_exhaust_again() {
    cr!("702.177a", "400.7");
    ruling!(
        "Jeong Jeong, the Deserter",
        "If an exhaust ability of a permanent is activated, and then that permanent leaves the battlefield and returns to the battlefield, it becomes a new object so its exhaust ability can be activated again."
    );
    supported("Jeong Jeong, the Deserter");
    // Jeong Jeong: "Exhaust — {3}: Put a +1/+1 counter on Jeong Jeong. ..."
    let mut t = TestGame::new(2);
    let jj = t.battlefield(P0, "Jeong Jeong, the Deserter");
    let exhaust = ability_uid(&mut t, jj, "Exhaust");
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_uid(&mut t, P0, jj, exhaust);
    t.resolve_all();
    assert_eq!(t.counters(jj, counters::PLUS1), 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    assert!(!activatable(&mut t, P0, jj, exhaust));
    // It's exiled and returned: a new object, whose exhaust ability can be activated.
    let exiled =
        t.g.move_object(jj, Zone::Exile, MoveCause::Effect, None)
            .unwrap();
    let back =
        t.g.move_object(exiled, Zone::Battlefield, MoveCause::Effect, None)
            .unwrap();
    t.settle();
    let exhaust = ability_uid(&mut t, back, "Exhaust");
    assert!(activatable(&mut t, P0, back, exhaust));
    activate_uid(&mut t, P0, back, exhaust);
    t.resolve_all();
    assert_eq!(t.counters(back, counters::PLUS1), 1);
}

#[test]
fn exhaust_abilities_can_be_activated_whenever_you_could_activate_an_ability() {
    cr!("702.177a", "602.2", "117.1b");
    ruling!(
        "Loot, the Pathfinder",
        "Exhaust abilities can be activated any time you could normally activate an ability."
    );
    supported("Loot, the Pathfinder");
    // Loot: "Exhaust — {U}, {T}: Draw three cards." In P1's turn, with a spell on the stack.
    let mut t = TestGame::new(2);
    let loot = t.battlefield(P0, "Loot, the Pathfinder");
    t.set_step(P1, Step::Upkeep);
    add_mana(&mut t, P1, ManaType::R, 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let draw = ability_uid(&mut t, loot, "Exhaust — {U}");
    add_mana(&mut t, P0, ManaType::U, 1);
    assert!(activatable(&mut t, P0, loot, draw));
    let hand = t.hand_size(P0);
    activate_uid(&mut t, P0, loot, draw);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // Only once.
    t.g.untap(loot);
    add_mana(&mut t, P0, ManaType::U, 1);
    assert!(!activatable(&mut t, P0, loot, draw));
}
