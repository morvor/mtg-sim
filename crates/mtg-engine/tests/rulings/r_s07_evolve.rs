//! Rulings batch S07 — evolve (CR 702.100): "Whenever a creature you control enters, if
//! that creature's power is greater than this creature's power and/or that creature's
//! toughness is greater than this creature's toughness, put a +1/+1 counter on this
//! creature."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts real cards onto the battlefield under `p`'s control at the same time, and puts
/// the triggers on the stack.
fn enter_together(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let moves = names
        .iter()
        .map(|n| MoveEv {
            obj: t.g.create_card_object(card(n), p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    t.settle();
    ids
}

/// The target creature gets +p/+t until end of turn (as a resolving effect of P1's).
fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    run_from(
        t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

#[test]
fn evolve_doesnt_trigger_unless_a_stat_of_the_new_creature_is_greater() {
    cr!("702.100a", "603.4");
    ruling!(
        "Gyre Sage",
        "Whenever a creature enters the battlefield under your control, check its power and toughness against the power and toughness of the creature with evolve. If neither stat of the new creature is greater, evolve won't trigger at all."
    );
    supported("Gyre Sage");
    // Gyre Sage: 1/2, evolve.
    let mut t = TestGame::new(2);
    let sage = t.battlefield(P0, "Gyre Sage");
    // Memnite (1/1) has neither a greater power nor a greater toughness: evolve doesn't
    // trigger at all.
    enter(&mut t, P0, "Memnite");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
    // Wall of Wood (0/3) has a greater toughness: it triggers.
    enter(&mut t, P0, "Wall of Wood");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(t.pt(sage), (2, 3));
}

#[test]
fn evolve_doesnt_trigger_unless_a_stat_is_greater_ray_fillet() {
    cr!("702.100a", "603.4");
    ruling!(
        "Ray Fillet, Wave Warrior",
        "Whenever a creature you control enters check its power and toughness against the power and toughness of the creature with evolve. If neither stat of the new creature is greater, evolve won't trigger at all."
    );
    supported("Ray Fillet, Wave Warrior");
    // Ray Fillet: 0/2 flying, evolve. Ornithopter (0/2) isn't greater; Memnite (1/1) is.
    let mut t = TestGame::new(2);
    let ray = t.battlefield(P0, "Ray Fillet, Wave Warrior");
    enter(&mut t, P0, "Ornithopter");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
    enter(&mut t, P0, "Memnite");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(t.pt(ray), (1, 3));
}

#[test]
fn the_greater_stat_may_change_before_evolve_resolves() {
    cr!("702.100a", "603.4");
    ruling!(
        "Adaptive Snapjaw",
        "When comparing the stats as the evolve ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on the creature with evolve."
    );
    supported("Adaptive Snapjaw");
    // Adaptive Snapjaw: 6/2, evolve. Hill Giant (3/3) has the greater toughness; in
    // response, it gets +4/-2 (7/1): now its power is greater.
    let mut t = TestGame::new(2);
    let snapjaw = t.battlefield(P0, "Adaptive Snapjaw");
    let giant = enter(&mut t, P0, "Hill Giant");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    pump(&mut t, giant, 4, -2);
    assert_eq!(t.pt(giant), (7, 1));
    t.resolve_all();
    assert_eq!(plus1(&t, snapjaw), 1);
    assert_eq!(t.pt(snapjaw), (7, 3));
}

#[test]
fn the_greater_stat_may_change_before_evolve_resolves_dinosaur_egg() {
    cr!("702.100a", "603.4");
    ruling!(
        "Dinosaur Egg",
        "When comparing the stats as the evolve ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on the creature with evolve. For example, if you control a 2/2 creature with evolve and a 1/3 creature enters the battlefield under your control, its toughness is greater"
    );
    supported("Dinosaur Egg");
    // Dinosaur Egg: 0/3, evolve. Wall of Stone (0/8) has the greater toughness; in
    // response it gets +1/-7 (1/1): now its power is greater.
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Dinosaur Egg");
    let wall = enter(&mut t, P0, "Wall of Stone");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    pump(&mut t, wall, 1, -7);
    assert_eq!(t.pt(wall), (1, 1));
    t.resolve_all();
    assert_eq!(plus1(&t, egg), 1);
}

#[test]
fn each_instance_of_evolve_compares_as_it_resolves() {
    cr!("702.100a", "702.100d");
    ruling!(
        "Tyranid Prime",
        "Multiple instances of evolve trigger separately and, similar to above, the stat comparison takes place for each one independently as they try to resolve."
    );
    supported("Tyranid Prime");
    supported("Cloudfin Raptor");
    // Tyranid Prime (0/4, evolve): "Other creatures you control have evolve." Cloudfin
    // Raptor (0/1, evolve) has two instances of evolve.
    let mut t = TestGame::new(2);
    let prime = t.battlefield(P0, "Tyranid Prime");
    let raptor = t.battlefield(P0, "Cloudfin Raptor");
    // Memnite (1/1) enters: the Raptor's two instances and the Prime's trigger.
    enter(&mut t, P0, "Memnite");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 3);
    t.resolve_all();
    // The first of the Raptor's abilities makes it 1/2; then Memnite (1/1) isn't greater,
    // so the second does nothing.
    assert_eq!(plus1(&t, raptor), 1);
    assert_eq!(t.pt(raptor), (1, 2));
    assert_eq!(plus1(&t, prime), 1);
}

#[test]
fn creatures_entering_together_are_compared_as_each_ability_resolves() {
    cr!("702.100a", "603.4");
    ruling!(
        "Ray Fillet, Wave Warrior",
        "If multiple creatures enter at the same time, evolve may trigger multiple times, although the stat comparison will take place each time one of those abilities tries to resolve."
    );
    // Ray Fillet (0/2); two Memnites (1/1) enter at the same time: two triggers; after the
    // first, Ray Fillet is 1/3 and the second does nothing.
    let mut t = TestGame::new(2);
    let ray = t.battlefield(P0, "Ray Fillet, Wave Warrior");
    enter_together(&mut t, P0, &["Memnite", "Memnite"]);
    assert_eq!(triggers_on_stack(&t, "Evolve"), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, ray), 1);
    assert_eq!(t.pt(ray), (1, 3));
}

#[test]
fn counters_the_creature_enters_with_are_considered() {
    cr!("702.100a", "614.1c");
    ruling!(
        "Ray Fillet, Wave Warrior",
        "If a creature enters with +1/+1 counters on it, consider those counters when determining if evolve will trigger."
    );
    supported("Arcbound Worker");
    // Arcbound Worker: a 0/0 that enters with a +1/+1 counter (modular 1): a 1/1 as it
    // enters, with a greater power than Ray Fillet (0/2).
    let mut t = TestGame::new(2);
    let ray = t.battlefield(P0, "Ray Fillet, Wave Warrior");
    enter(&mut t, P0, "Arcbound Worker");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, ray), 1);
}

#[test]
fn power_is_compared_to_power_and_toughness_to_toughness() {
    cr!("702.100a");
    ruling!(
        "Scurry Oak",
        "When comparing the characteristics of the two creatures for evolve, you always compare power to power and toughness to toughness."
    );
    supported("Scurry Oak");
    supported("Elite Vanguard");
    // Scurry Oak: 1/2, evolve; "Whenever one or more +1/+1 counters are put on this
    // creature, you may create a 1/1 green Squirrel creature token."
    let mut t = TestGame::new(2);
    let oak = t.battlefield(P0, "Scurry Oak");
    // Elite Vanguard (2/1): its power (2) is greater than the Oak's power (1), though not
    // its toughness (2).
    enter(&mut t, P0, "Elite Vanguard");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.pt(oak), (2, 3));
    // Wall of Wood (0/3) vs 2/3: neither its power nor its toughness is greater (its
    // toughness is greater than the Oak's power, which doesn't matter).
    enter(&mut t, P0, "Wall of Wood");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
}

#[test]
fn the_comparison_is_made_again_as_evolve_resolves_with_last_known_information() {
    cr!("702.100a", "603.4", "608.2h");
    ruling!(
        "Gluttonous Slug",
        "If evolve triggers, the comparison will happen again when the ability tries to resolve. If neither characteristic of the new creature is greater, the ability will do nothing. If the creature that entered the battlefield leaves the battlefield before evolve resolves, use its last known power and toughness"
    );
    supported("Gluttonous Slug");
    // Gluttonous Slug: 0/3 menace, evolve. Grizzly Bears (2/2) enters; in response it gets
    // -2/-0: the ability does nothing.
    let mut t = TestGame::new(2);
    let slug = t.battlefield(P0, "Gluttonous Slug");
    let bears = enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    pump(&mut t, bears, -2, 0);
    t.resolve_all();
    assert_eq!(plus1(&t, slug), 0);
    // Another Bears enters and is destroyed in response: its last known power (2) is
    // greater, so the Slug gets a counter.
    let bears = enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    destroy(&mut t, bears);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(plus1(&t, slug), 1);
    assert_eq!(t.pt(slug), (1, 4));
}
