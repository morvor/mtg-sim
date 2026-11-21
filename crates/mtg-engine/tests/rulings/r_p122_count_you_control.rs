//! Rulings batch P122 — "with power (mana value) less than or equal to the number of
//! [permanents] you control": Goma Fada Vanguard, and the cards that compile through the
//! same phrase (Beguiler of Wills, Dominating Vampire, Lay Down Arms, Anticausal Vestige,
//! Beseech the Queen, Vedalken Shackles). Each clause of each card is exercised. The
//! number is counted as targets are chosen and again as the ability resolves (CR 601.2c,
//! 608.2b).

use crate::r_p122_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s09_common::to_combat;
use crate::r_s24_common::controller;
use crate::r_s28_common::cast_card;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn can_block(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.can_block_at_all(id)
}

/// The candidates of the first target choice asked of P0 since decision `from`.
fn first_candidates(t: &TestGame, from: usize) -> Vec<Entity> {
    target_candidates(t, P0, from)
        .into_iter()
        .next()
        .expect("a target was chosen")
}

#[test]
fn goma_fada_vanguard_power_checked_on_stack_and_resolution() {
    cr!("601.2c", "603.3d", "608.2b");
    ruling!(
        "Goma Fada Vanguard",
        "The power of the target creature and the number of Warriors you control are checked only as Goma Fada Vanguard's ability is put onto the stack and as that ability resolves. If the creature's power becomes greater after the ability has resolved, it still can't block."
    );
    supported("Goma Fada Vanguard");
    // One Warrior: only a creature with power 1 or less can be targeted. Its power
    // growing afterwards doesn't matter.
    let mut t = TestGame::new(2);
    let vanguard = t.battlefield(P0, "Goma Fada Vanguard");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(elves)]);
    to_combat(&mut t, P0);
    let from = t.asked().len();
    attack_with(&mut t, &[(vanguard, Entity::Player(P1))]);
    let offered = first_candidates(&t, from);
    assert!(offered.contains(&obj(elves)) && !offered.contains(&obj(bears)));
    t.resolve_all();
    assert!(!can_block(&mut t, elves));
    t.answer_targets(P1, &[obj(elves)]);
    cast_card(&mut t, P1, "Giant Growth");
    t.resolve_all();
    assert_eq!(t.pt(elves).0, 4);
    assert!(!can_block(&mut t, elves), "it still can't block");
    // Two Warriors: a 2-power creature can be targeted; if a Warrior leaves before the
    // ability resolves, the target is illegal and the ability doesn't resolve.
    let mut t = TestGame::new(2);
    let vanguard = t.battlefield(P0, "Goma Fada Vanguard");
    let javelineer = t.battlefield(P0, "Merciless Javelineer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(vanguard, Entity::Player(P1))]);
    destroy(&mut t, javelineer);
    t.resolve_all();
    assert!(can_block(&mut t, bears));
}

#[test]
fn beguiler_of_wills_counts_creatures_you_control() {
    cr!("601.2c", "602.2b");
    supported("Beguiler of Wills");
    let mut t = TestGame::new(2);
    let beguiler = t.battlefield(P0, "Beguiler of Wills");
    t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.activate(P0, beguiler, 0, &[obj(bears)]).unwrap();
    let offered = first_candidates(&t, from);
    assert!(offered.contains(&obj(bears)) && !offered.contains(&obj(giant)));
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
}

#[test]
fn dominating_vampire_counts_vampires_you_control() {
    cr!("601.2c", "603.3d");
    supported("Dominating Vampire");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(elves);
    t.answer_targets(P0, &[obj(elves)]);
    let from = t.asked().len();
    t.enter(P0, "Dominating Vampire");
    t.settle();
    // Itself is the only Vampire: mana value 1 or less.
    let offered = first_candidates(&t, from);
    assert!(offered.contains(&obj(elves)) && !offered.contains(&obj(bears)));
    t.resolve_all();
    assert_eq!(controller(&mut t, elves), P0);
    assert!(!t.obj_now(elves).tapped, "untapped");
    assert!(t
        .obj_now(elves)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(controller(&mut t, elves), P1);
}

#[test]
fn lay_down_arms_counts_plains_you_control() {
    cr!("601.2c", "608.2c");
    supported("Lay Down Arms");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Lay Down Arms");
    let from = t.asked().len();
    t.cast(P0, card).target(obj(bears)).go();
    let offered = first_candidates(&t, from);
    assert!(offered.contains(&obj(bears)) && !offered.contains(&obj(giant)));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 23, "its controller gains 3 life");
}

#[test]
fn anticausal_vestige_counts_lands_you_control() {
    cr!("603.6c", "608.2c");
    supported("Anticausal Vestige");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let vestige = t.battlefield(P0, "Anticausal Vestige");
    let bears = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    destroy(&mut t, vestige);
    t.resolve_all();
    let choices: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 1);
    assert!(choices[0].contains(&obj(bears)));
    assert!(
        choices[0].iter().all(|e| t.g.obj(e.object().unwrap()).chars.name != "Hill Giant"),
        "mana value 4 is more than two lands"
    );
    let bears_now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears_now.len(), 1);
    assert!(t.g.obj(bears_now[0]).tapped, "onto the battlefield tapped");
    // It drew a card first; Grizzly Bears left the hand.
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn beseech_the_queen_counts_lands_you_control() {
    cr!("701.23a");
    supported("Beseech the Queen");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let lib = stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears"]);
    let card = t.hand(P0, "Beseech the Queen");
    mana(&mut t, P0, ManaType::B, 3);
    t.answer_choose(P0, &[obj(lib[1])]);
    let from = t.asked().len();
    t.cast(P0, card).go();
    t.resolve_all();
    let choices: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(!choices.is_empty());
    assert!(choices[0].contains(&obj(lib[1])) && !choices[0].contains(&obj(lib[0])));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn vedalken_shackles_counts_islands_while_tapped() {
    cr!("601.2c", "611.2b");
    supported("Vedalken Shackles");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let shackles = t.battlefield(P0, "Vedalken Shackles");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::C, 2);
    let from = t.asked().len();
    t.activate(P0, shackles, 0, &[obj(bears)]).unwrap();
    let offered = first_candidates(&t, from);
    assert!(offered.contains(&obj(bears)) && !offered.contains(&obj(giant)));
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    // For as long as it remains tapped.
    t.g.untap(shackles);
    t.g.flush_events();
    t.settle();
    assert_eq!(controller(&mut t, bears), P1);
}
