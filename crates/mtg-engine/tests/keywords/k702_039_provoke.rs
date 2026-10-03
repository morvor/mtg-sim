//! CR 702.39 Provoke.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_038_051::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Declares attackers and puts the attack triggers on the stack.
fn attack(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    attack_with(t, attackers);
    t.settle();
}

fn blockers_of(t: &TestGame, attacker: ObjectId) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.blockers_of(t.g.current(attacker)))
        .unwrap_or_default()
}

#[test]
fn provoke_untaps_the_target_which_must_block_if_able() {
    cr!("702.39", "702.39a");
    assert_supported("Goblin Grappler");
    let mut t = TestGame::new(2);
    let grappler = t.battlefield(P0, "Goblin Grappler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack(&mut t, &[(grappler, Entity::Player(P1))]);
    assert_eq!(triggers_named(&t, "Provoke").len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    // P1 tries not to block: that declaration disobeys the requirement, so the Bears
    // block the Goblin Grappler.
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(blockers_of(&t, grappler), vec![bears]);
    assert!(!t.g.is_blocking(other));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(grappler));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn other_creatures_may_block_too_and_the_requirement_ends_with_combat() {
    cr!("702.39a");
    let mut t = TestGame::new(2);
    let grappler = t.battlefield(P0, "Goblin Grappler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack(&mut t, &[(grappler, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let requirement = |t: &TestGame| {
        t.g.rule_effects.iter().any(|e| {
            matches!(
                e.restriction,
                ability::Restriction::MustBlockAttacker { .. }
            )
        })
    };
    assert!(requirement(&t));
    // Both Bears may block it.
    declare_blocks(&mut t, P1, &[(bears, grappler), (other, grappler)]);
    let mut b = blockers_of(&t, grappler);
    b.sort();
    let mut want = vec![bears, other];
    want.sort();
    assert_eq!(b, want);
    // "this combat": the requirement ends with the combat phase.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!requirement(&t));
}

#[test]
fn declining_leaves_the_creature_tapped_and_free() {
    cr!("702.39a");
    let mut t = TestGame::new(2);
    let grappler = t.battlefield(P0, "Goblin Grappler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack(&mut t, &[(grappler, Entity::Player(P1))]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    declare_blocks(&mut t, P1, &[]);
    assert!(blockers_of(&t, grappler).is_empty());
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn only_creatures_the_defending_player_controls_can_be_targeted() {
    cr!("702.39a");
    let mut t = TestGame::new(2);
    let grappler = t.battlefield(P0, "Goblin Grappler");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack(&mut t, &[(grappler, Entity::Player(P1))]);
    let cands = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("provoke target asked");
    assert!(cands.contains(&Entity::Object(theirs)));
    assert!(!cands.contains(&Entity::Object(mine)));
    assert!(!cands.contains(&Entity::Object(grappler)));
}

#[test]
fn the_provoked_creature_blocks_only_if_able() {
    cr!("702.39a");
    // Swooping Talon has flying; a provoked creature without flying or reach can't block
    // it, until it loses flying.
    assert_supported("Swooping Talon");
    let mut t = TestGame::new(2);
    let talon = t.battlefield(P0, "Swooping Talon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack(&mut t, &[(talon, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    declare_blocks(&mut t, P1, &[]);
    assert!(blockers_of(&t, talon).is_empty());
}

#[test]
fn losing_flying_makes_the_provoked_creature_able_to_block() {
    cr!("702.39a");
    let mut t = TestGame::new(2);
    let talon = t.battlefield(P0, "Swooping Talon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack(&mut t, &[(talon, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.resolve_all();
    // "{1}: This creature loses flying until end of turn."
    t.activate(P0, talon, 0, &[]).unwrap();
    t.resolve_all();
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(blockers_of(&t, talon), vec![bears]);
}

#[test]
fn each_instance_of_provoke_triggers_separately() {
    cr!("702.39b");
    let def = custom_card(
        "Twice Provoking Goblin",
        "Creature — Goblin",
        Some((1, 1)),
        "Provoke\nProvoke",
    );
    let mut t = TestGame::new(2);
    let goblin = bf(&mut t, P0, def);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(b1)]);
    t.answer_targets(P0, &[Entity::Object(b2)]);
    attack(&mut t, &[(goblin, Entity::Player(P1))]);
    assert_eq!(triggers_named(&t, "Provoke").len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    declare_blocks(&mut t, P1, &[]);
    let mut b = blockers_of(&t, goblin);
    b.sort();
    let mut want = vec![b1, b2];
    want.sort();
    assert_eq!(b, want);
}
