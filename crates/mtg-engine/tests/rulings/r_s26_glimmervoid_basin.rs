//! Rulings batch S26 — Glimmervoid Basin (a plane, CR 311, 901): a spell with a single
//! target is copied for each other object or player it could target (CR 707.10d); the
//! copies have the effects of the original's additional costs (CR 707.2).

use crate::r_s01_common::supported;
use crate::r_s19_common::{chaos, planechase_game, start_planar_deck};
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn glimmervoid_basin_copies_have_the_originals_additional_cost_effects() {
    cr!("707.10", "707.2", "707.10d");
    ruling!(
        "Glimmervoid Basin",
        "The controller of a copy can't choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Glimmervoid Basin");
    supported("Fling");
    // Fling: "As an additional cost to cast this spell, sacrifice a creature. Fling deals
    // damage equal to the sacrificed creature's power to any target."
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Glimmervoid Basin"]);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, fling).target(P1).go();
    t.settle();
    // The trigger resolves: one copy, targeting the only other legal target (P0).
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_on_stack(&t, copies[0]), vec![Entity::Player(P0)]);
    t.resolve_all();
    // The copy dealt 3 damage too, without another sacrifice.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 17);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn glimmervoid_basin_copies_a_single_target_spell_for_each_other_target() {
    cr!("707.10d", "115.9a");
    supported("Glimmervoid Basin");
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Glimmervoid Basin"]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    t.resolve();
    // Copies for P0, the Bears and the Elves; each has a different target.
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 3);
    let mut targets: Vec<String> = copies
        .iter()
        .flat_map(|c| targets_on_stack(&t, *c))
        .map(|e| format!("{e:?}"))
        .collect();
    targets.sort();
    let mut want: Vec<String> = [
        Entity::Player(P0),
        Entity::Object(bears),
        Entity::Object(elves),
    ]
    .iter()
    .map(|e| format!("{e:?}"))
    .collect();
    want.sort();
    assert_eq!(targets, want);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 17);
    assert!(!t.g.is_live(bears) && !t.g.is_live(elves));
    // A spell with two targets isn't copied.
    let trail = t.hand(P0, "Arc Trail");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, trail).target(P1).target(P0).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn glimmervoid_basin_chaos_gives_the_other_players_a_copy() {
    cr!("311.7", "707.2");
    supported("Glimmervoid Basin");
    // "Whenever chaos ensues, choose target creature. Each player except that creature's
    // controller creates a token that's a copy of that creature."
    let mut t = planechase_game(3);
    start_planar_deck(&mut t, P0, &["Glimmervoid Basin"]);
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[Entity::Object(angel)]);
    let before = t.g.battlefield.clone();
    chaos(&mut t, P0);
    t.resolve_all();
    assert_eq!(new_tokens(&t, P0, &before).len(), 1);
    assert_eq!(new_tokens(&t, P1, &before).len(), 0);
    assert_eq!(new_tokens(&t, P2, &before).len(), 1);
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 3);
}
