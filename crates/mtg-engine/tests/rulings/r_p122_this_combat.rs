//! Rulings batch P122 — cards whose "can't block this combat", "can't be blocked this
//! combat" and "until your next turn, creatures can't attack you" effects compile through
//! the phrases added for this batch: Forgestoker Dragon, Chronomantic Escape, Ma Chao,
//! Western Warrior, Yuan Shao's Infantry, Sly Instigator and Taunt from the Rampart. Every
//! clause of each card is exercised.

use crate::r_p122_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s09_common::{declare_in_next_combat, legal_attack, to_combat};
use crate::r_s10_common::attacking;
use crate::r_s21_common::legal_blocks;
use crate::r_s28_common::cast_card;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn forgestoker_dragon_only_while_attacking_and_until_end_of_combat() {
    cr!("602.5", "509.1b", "611.2a");
    ruling!(
        "Forgestoker Dragon",
        "If you don't want the target creature to be able to block, Forgestoker Dragon's activated ability must be activated during the declare attackers step."
    );
    supported("Forgestoker Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Forgestoker Dragon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "Activate only if this creature is attacking."
    mana(&mut t, P0, ManaType::R, 2);
    assert!(t.activate(P0, dragon, 0, &[obj(bears)]).is_err());
    to_combat(&mut t, P0);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(t.activate(P0, dragon, 0, &[obj(bears)]).is_err());
    attack_with(&mut t, &[(dragon, Entity::Player(P1))]);
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, dragon, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    // 1 damage, and the creature can't block this combat.
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(!t.g.can_block_at_all(bears));
    t.advance_to(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.g.can_block_at_all(bears), "the effect lasts only this combat");
}

#[test]
fn forgestoker_dragon_can_target_any_creature() {
    cr!("115.1c");
    ruling!(
        "Forgestoker Dragon",
        "Forgestoker Dragon's activated ability can target any creature, not just creatures controlled by the defending player or ones that could block."
    );
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Forgestoker Dragon");
    let own = t.battlefield(P0, "Hill Giant");
    t.g.objects[own.0 as usize].tapped = true;
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(dragon, Entity::Player(P1))]);
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, dragon, 0, &[obj(own)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(own).damage, 1);
}

#[test]
fn chronomantic_escape_stops_creatures_that_arrive_later() {
    cr!("611.2c", "508.1c");
    ruling!(
        "Chronomantic Escape",
        "Chronomantic Escape can affect creatures that aren't on the battlefield at the time it resolves"
    );
    ruling!(
        "Chronomantic Escape",
        "Unless some effect explicitly says otherwise, a creature that can't attack you can still attack a planeswalker you control."
    );
    supported("Chronomantic Escape");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let escape = cast_card(&mut t, P0, "Chronomantic Escape");
    t.resolve_all();
    // "Exile Chronomantic Escape with three time counters on it." It's suspended.
    let exiled = t.g.current(escape);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    assert_eq!(t.counters(exiled, "time"), 3);
    // A creature with haste that arrives on the opponent's turn can't attack P0, but can
    // attack P0's planeswalker.
    t.advance_to(P1, Step::PrecombatMain);
    let goblin = t.enter(P1, "Raging Goblin");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[(goblin, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(goblin, obj(jace))]));
    // The effect ends as P0's next turn begins.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.can_attack_target(goblin, Entity::Player(P0)));
}

/// `name` attacks P1 alone, or together with a Hill Giant; P1 has `blocker`.
fn attacks_alone_unblockable(name: &str, blocker_name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, name);
    let blocker = t.battlefield(P1, blocker_name);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(blocker, attacker)]), "{name}");
    t.advance_to(P0, Step::PostcombatMain);
    // Attacking with another creature: it can be blocked.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, name);
    let giant = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, blocker_name);
    to_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[(attacker, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[(blocker, attacker)]), "{name}");
}

#[test]
fn ma_chao_attacking_alone_cant_be_blocked() {
    cr!("506.5", "509.1b", "702.31b");
    // A horsemanship blocker could otherwise block it (CR 702.31b).
    attacks_alone_unblockable("Ma Chao, Western Warrior", "Wei Strike Force");
}

#[test]
fn yuan_shaos_infantry_attacking_alone_cant_be_blocked() {
    cr!("506.5", "509.1b");
    attacks_alone_unblockable("Yuan Shao's Infantry", "Grizzly Bears");
}

#[test]
fn sly_instigator_goads_and_makes_unblockable_until_your_next_turn() {
    cr!("701.15a", "701.15b", "509.1b");
    supported("Sly Instigator");
    let mut t = TestGame::new(2);
    let instigator = t.battlefield(P0, "Sly Instigator");
    let wall = t.battlefield(P0, "Wall of Wood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, instigator, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.goaders(bears), vec![P0]);
    // On P1's turn the goaded creature attacks even though P1 declares no attackers,
    // and it can't be blocked.
    declare_in_next_combat(&mut t, P1, &[]);
    assert!(attacking(&t, bears));
    assert!(!legal_blocks(&mut t, P0, &[(wall, bears)]));
}

#[test]
fn taunt_from_the_rampart_goads_and_those_creatures_cant_block() {
    cr!("701.15a", "509.1b");
    supported("Taunt from the Rampart");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_card(&mut t, P0, "Taunt from the Rampart");
    t.resolve_all();
    assert_eq!(t.g.goaders(bears), vec![P0]);
    // A creature that arrives later wasn't one of "those creatures".
    let lions = t.battlefield(P1, "Savannah Lions");
    assert!(t.g.goaders(lions).is_empty());
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(lions, giant)]));
}
