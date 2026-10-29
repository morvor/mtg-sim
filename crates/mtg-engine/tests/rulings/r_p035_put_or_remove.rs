//! Rulings batch P035 — "put a [kind] counter on ~ or remove one from it" (Jinxed Choker,
//! Lavabrink Floodgates): the choice is made as the ability resolves.

use crate::r_p035_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn put_or_remove_choices(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseOption { options, .. }
                if options.iter().any(|o| o.starts_with("Remove a")))
        })
        .count()
}

#[test]
fn jinxed_choker_add_or_remove_is_chosen_on_resolution() {
    cr!("608.2c", "602.2");
    ruling!(
        "Jinxed Choker",
        "If you activate Jinxed Choker’s activated ability, you choose to either add or remove a counter when the ability resolves."
    );
    supported("Jinxed Choker");
    let mut t = TestGame::new(2);
    let choker = t.battlefield(P0, "Jinxed Choker");
    put(&mut t, choker, "charge", 2);
    t.lands(P0, "Wastes", 6);
    let from = t.asked().len();
    t.activate(P0, choker, 0, &[]).unwrap();
    assert_eq!(
        put_or_remove_choices(&t, from),
        0,
        "not chosen on activation"
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(put_or_remove_choices(&t, from), 1);
    assert_eq!(t.counters(choker, "charge"), 1, "removed one");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, choker, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(choker, "charge"), 2, "put one");
}

#[test]
fn jinxed_choker_you_is_its_current_controller() {
    cr!("109.5");
    ruling!(
        "Jinxed Choker",
        "“You” is always Jinxed Choker’s current controller."
    );
    let mut t = TestGame::new(2);
    let choker = t.battlefield(P0, "Jinxed Choker");
    put(&mut t, choker, "charge", 1);
    // At P0's end step, P1 gains control of it and puts a charge counter on it.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.obj_now(choker).controller, P1);
    assert_eq!(t.counters(choker, "charge"), 2);
    // At P1's upkeep, it deals 2 damage to P1, its controller now.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

/// Lavabrink Floodgates (controlled by P0) with `doom` counters, at the start of `p`'s
/// upkeep with its trigger on the stack.
fn floodgates_upkeep(doom: u32, p: PlayerId) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    // (Placed after P1's upkeep, when P0's upkeep is the next one.)
    if p == P0 {
        t.advance_to(P1, Step::End);
    }
    let gates = t.battlefield(P0, "Lavabrink Floodgates");
    if doom > 0 {
        put(&mut t, gates, "doom", doom);
    }
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    t.advance_to(p, Step::Upkeep);
    t.settle();
    (t, gates)
}

#[test]
fn lavabrink_floodgates_the_upkeep_player_may_put_remove_or_do_nothing() {
    cr!("603.12", "608.2c");
    ruling!(
        "Lavabrink Floodgates",
        "The player whose upkeep it is may put a doom counter on Lavabrink Floodgates, remove one from it, or do nothing."
    );
    supported("Lavabrink Floodgates");
    // On P1's upkeep, P1 decides: put one.
    let (mut t, gates) = floodgates_upkeep(1, P1);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.counters(gates, "doom"), 2);
    // Remove one.
    let (mut t, gates) = floodgates_upkeep(1, P1);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(t.counters(gates, "doom"), 0);
    // Do nothing.
    let (mut t, gates) = floodgates_upkeep(1, P1);
    t.answer_yes(P1, false);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(gates, "doom"), 1);
    assert_eq!(put_or_remove_choices(&t, from), 0);
    // P0 wasn't asked anything about P1's upkeep trigger.
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P1));
}

#[test]
fn lavabrink_floodgates_third_counter_sacrifice_and_damage() {
    cr!("603.12", "701.21a");
    ruling!(
        "Lavabrink Floodgates",
        "Lavabrink Floodgates’s reflexive triggered ability triggers and deals damage only if you sacrifice it while resolving its triggered ability."
    );
    // The third counter: it's sacrificed and deals 6 damage to each creature.
    let (mut t, gates) = floodgates_upkeep(2, P0);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lavabrink Floodgates"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert!(!t.g.is_live(gates));
    // Two counters after resolving: nothing happens.
    let (mut t, gates) = floodgates_upkeep(2, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.on_battlefield(gates));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Sacrificed some other way: no damage.
    let (mut t, gates) = floodgates_upkeep(0, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    let g = t.g.current(gates);
    t.g.sacrifice(g, P0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn lavabrink_floodgates_third_counter_some_other_way_isnt_sacrificed() {
    cr!("603.12");
    ruling!(
        "Lavabrink Floodgates",
        "Lavabrink Floodgates is sacrificed for having three or more doom counters on it only while its ability is resolving."
    );
    let mut t = TestGame::new(2);
    let gates = t.battlefield(P0, "Lavabrink Floodgates");
    put(&mut t, gates, "doom", 3);
    assert!(t.on_battlefield(gates));
    // At the next upkeep, the counters are checked again: removing one leaves two.
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert!(t.on_battlefield(gates));
    assert_eq!(t.counters(gates, "doom"), 2);
}
