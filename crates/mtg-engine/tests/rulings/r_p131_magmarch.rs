//! Rulings batch P131 — Exterminator Magmarch: an intervening-"if" cast trigger that copies
//! a single-target spell onto a permanent of another opponent (CR 707.10, 707.10e, 603.4).

use crate::r_p131_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A three-player game with Exterminator Magmarch on P0's side.
fn table() -> TestGame {
    supported("Exterminator Magmarch");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Exterminator Magmarch");
    t
}

/// The candidates of the "choose one of those permanents" choice asked of P0 since `from`.
fn offered(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn magmarch_doesnt_trigger_without_another_opponents_targetable_permanent() {
    cr!("603.4", "702.11b");
    ruling!(
        "Exterminator Magmarch",
        "Exterminator Magmarch's first ability won't trigger if no other opponent controls any nonland permanents that spell could target."
    );
    // The other opponent controls only a land and a hexproof creature.
    let mut t = table();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Forest");
    t.battlefield(P2, "Gladecover Scout");
    cast(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The trigger is there, but the other opponent's creature is gone as it resolves.
    let mut t = table();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P2, "Grizzly Bears");
    let bolt = cast(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    kill(&mut t, other);
    t.resolve();
    assert!(spell_copies(&t).is_empty());
    assert_eq!(t.g.stack, vec![bolt]);
}

#[test]
fn magmarch_copies_a_spell_targeting_one_permanent_several_times() {
    cr!("707.10e", "115.3");
    ruling!(
        "Exterminator Magmarch",
        "If a spell has multiple targets but all of them are the same nonland permanent an opponent controls, Exterminator Magmarch's ability will trigger as long as another opponent controls one or more nonland permanents that spell could target. The copy will target only the chosen permanent and no other permanents."
    );
    supported("Seeds of Strength");
    let mut t = table();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P2, "Hill Giant");
    let seeds = cast(
        &mut t,
        P0,
        "Seeds of Strength",
        &[obj(bears), obj(bears), obj(bears)],
    );
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_choose(P0, &[obj(theirs)]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(theirs); 3]);
    assert_eq!(targets_of(&t, seeds), vec![obj(bears); 3]);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (6, 6));
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn magmarchs_copy_of_a_modal_spell_has_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Exterminator Magmarch",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same modes. You can't choose different ones."
    );
    supported("Abrade");
    let mut t = table();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P2, "Craw Wurm");
    let ring = t.battlefield(P2, "Sol Ring");
    lands_for_cost(&mut t, P0, "Abrade");
    let card = t.hand(P0, "Abrade");
    t.cast(P0, card).modes(&[0]).target(bears).go();
    t.settle();
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(wurm)]);
    t.resolve();
    // Only what the spell could target with its chosen mode is offered: the creature.
    assert_eq!(offered(&t, from), vec![vec![obj(wurm)]]);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert!(t.on_battlefield(ring));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn magmarchs_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10");
    ruling!(
        "Exterminator Magmarch",
        "effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Burst Lightning");
    let mut t = table();
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P2, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    let from = t.asked().len();
    t.cast(P0, card).kicked(true).target(giant).go();
    t.answer_choose(P0, &[obj(wurm)]);
    t.resolve_all();
    // 4 damage each: the 6/4 dies too.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P2, "Craw Wurm"));
    // Nobody was asked about kicking the copy.
    let kicker_asks = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count();
    assert_eq!(kicker_asks, 1);
}

#[test]
fn magmarchs_copy_isnt_cast() {
    cr!("707.10", "702.108a");
    ruling!(
        "Exterminator Magmarch",
        "The copy that Exterminator Magmarch's ability creates is created on the stack, so it's not \"cast.\" Other abilities that trigger when a player casts a spell won't trigger."
    );
    let mut t = table();
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P2, "Grizzly Bears");
    cast(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    t.answer_choose(P0, &[obj(other)]);
    t.resolve_all();
    assert!(!t.on_battlefield(other));
    // Prowess triggered once (for the Bolt), not for the copy.
    assert_eq!(t.pt(spear), (2, 3));
}
