//! Rulings batch P078 — the rest: Starnheim Courser's cost reduction, Burning-Tree
//! Shaman and mana abilities, Focus Fire counting a Spacecraft creature once, Gossamer
//! Chains' "unblocked creature", and Labyrinth of Skophos removing a tapped creature from
//! combat.

use crate::r_p078_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_activate, target_candidates};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A custom {W} enchantment and a {1}{W} one (no abilities).
fn enchantment(t: &mut TestGame, cost: &str) -> ObjectId {
    let c = crate::r_s01_common::custom_card("Plain Charm", "Enchantment", cost, None, "");
    t.custom(P0, c, mtg_engine::object::Zone::Hand(P0))
}

#[test]
fn starnheim_courser_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Starnheim Courser",
        "The cost reduction applies only to generic mana in the costs of artifact and enchantment spells you cast. It can't reduce requirements of specific colors of mana."
    );
    supported("Starnheim Courser");
    // {1}{W} costs {W}: one Plains pays it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starnheim Courser");
    t.lands(P0, "Plains", 1);
    let e = enchantment(&mut t, "{1}{W}");
    assert!(t.cast(P0, e).try_go().is_ok());
    // {W}{W} still costs {W}{W}: one Plains doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starnheim Courser");
    t.lands(P0, "Plains", 1);
    let e = enchantment(&mut t, "{W}{W}");
    assert!(t.cast(P0, e).try_go().is_err());
}

#[test]
fn starnheim_courser_applies_after_cost_increases_and_keeps_the_mana_value() {
    cr!("601.2f", "202.3", "118.7");
    ruling!(
        "Starnheim Courser",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Starnheim Courser). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Thalia, Guardian of Thraben");
    // A {W} enchantment with Thalia ({1} more): {1}{W}, then {1} less: {W}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starnheim Courser");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Plains", 1);
    let e = enchantment(&mut t, "{W}");
    let spell = t.cast(P0, e).try_go().expect("castable for {W}");
    assert_eq!(t.g.mana_value_of(spell), 1);
    // A {1}{W} enchantment: {2}{W}, then {1}{W}; its mana value is still 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starnheim Courser");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Plains", 2);
    let e = enchantment(&mut t, "{1}{W}");
    let spell = t.cast(P0, e).try_go().expect("castable for {1}{W}");
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn burning_tree_shaman_ignores_mana_abilities_even_ones_that_cost_mana() {
    cr!("605.1a", "602.1");
    ruling!(
        "Burning-Tree Shaman",
        "A mana ability is an ability that adds mana, not one that costs mana to activate."
    );
    supported("Burning-Tree Shaman");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Burning-Tree Shaman");
    // Tapping a land for mana: a mana ability.
    let m = t.battlefield(P0, "Mountain");
    assert!(tap_for_mana(&mut t, P0, m, "{R}"));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Yurlok's "{1}, {T}: Each player adds {B}{R}{G}": costs mana, adds mana: a mana
    // ability too.
    let y = t.battlefield(P0, "Yurlok of Scorch Thrash");
    assert!(tap_for_mana(&mut t, P0, y, "Each player adds"));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
    // Welder Automaton's "{3}{R}: This creature deals 1 damage to each opponent": costs
    // mana, isn't a mana ability.
    let w = t.battlefield(P0, "Welder Automaton");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn focus_fire_counts_a_spacecraft_creature_once() {
    cr!("608.2h", "721.2");
    ruling!(
        "Focus Fire",
        "A Spacecraft you control that’s also a creature counts only once toward the value of X."
    );
    supported("Focus Fire");
    supported("Uthros Scanship");
    let mut t = TestGame::new(2);
    // Uthros Scanship with eight charge counters is an artifact creature Spacecraft.
    let ship = t.battlefield(P0, "Uthros Scanship");
    t.g.add_counters(obj(ship), "charge", 8, None);
    t.g.recompute();
    assert!(t.obj_now(ship).is(mtg_engine::types::CardType::Creature));
    // P1's Craw Wurm (6/4) attacks; X = 2 + 1 = 3 damage, not 4.
    t.set_step(P1, Step::BeginningOfCombat);
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(wurm, pl(P0))]);
    let ff = t.hand(P0, "Focus Fire");
    t.lands(P0, "Plains", 1);
    t.cast(P0, ff).target(obj(wurm)).go();
    t.resolve();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 3);
}

#[test]
fn gossamer_chains_needs_blockers_declared_for_an_unblocked_creature() {
    cr!("509.1h");
    ruling!(
        "Gossamer Chains",
        "A creature is only considered an “unblocked creature” if it is an attacker during combat after blockers are declared."
    );
    supported("Gossamer Chains");
    let mut t = TestGame::new(2);
    let chains = t.battlefield(P0, "Gossamer Chains");
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P0))]);
    // Declare attackers step: no unblocked creature yet.
    assert!(!can_activate(&mut t, P0, chains));
    // After blockers are declared (none): Grizzly Bears is unblocked.
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P1, Step::DeclareBlockers);
    assert!(can_activate(&mut t, P0, chains));
    let from = t.asked().len();
    t.activate(P0, chains, 0, &[obj(bears)]).unwrap();
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(bears)]]);
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn labyrinth_of_skophos_leaves_a_removed_creature_tapped() {
    cr!("506.4");
    ruling!(
        "Labyrinth of Skophos",
        "A creature removed from combat by Labyrinth of Skophos remains tapped if it was tapped."
    );
    supported("Labyrinth of Skophos");
    let mut t = TestGame::new(2);
    let lab = t.battlefield(P0, "Labyrinth of Skophos");
    t.lands(P0, "Wastes", 4);
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P0))]);
    assert!(t.obj_now(bears).tapped);
    t.activate(P0, lab, 1, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(!t.g.is_attacking(bears));
    assert!(t.obj_now(bears).tapped);
}
