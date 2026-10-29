//! Triggers and grants created by effects (cards4-triggers-durations): "this turn,
//! whenever" (Harsh Justice), the triggering event's amount in a this-turn trigger (False
//! Cure), a this-turn trigger naming a qualified recipient (Acidic Dagger), a delayed
//! value determined as the delayed ability resolves (Hazezon Tamar), losing control of
//! an object (Magus of the Unseen, Stolen Uniform), and a "for as long as it has a
//! counter" untap restriction on a group (Dread Wight).

use crate::combat2_common::{declare, declare_blocks, supported, to_end_of_combat};
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

fn ready(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

#[test]
fn harsh_justice_reflects_combat_damage_dealt_to_you_this_turn() {
    cr!("603.7b", "603.7c");
    supported("Harsh Justice");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    ready(&mut t, bears);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    let hj = t.hand(P1, "Harsh Justice");
    mana(&mut t, P1);
    t.cast_with(P1, hj, &[]).expect("castable while attacked");
    t.resolve_all();
    to_end_of_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18, "the Bears dealt 2 damage to their controller");
}

#[test]
fn false_cure_makes_each_life_gain_cost_twice_as_much() {
    cr!("603.7b", "603.7c");
    supported("False Cure");
    let mut t = TestGame::new(2);
    let cure = t.hand(P0, "False Cure");
    mana(&mut t, P0);
    t.cast_with(P0, cure, &[]).unwrap();
    t.resolve_all();
    t.g.gain_life(P1, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "gained 3, then lost 6");
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // Only this turn.
    t.advance_to(P1, Step::Upkeep);
    t.g.gain_life(P1, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// Acidic Dagger targets `ours`, which attacks and is blocked by `blocker`.
fn dagger_combat(blocker: &str) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let dagger = t.battlefield(P0, "Acidic Dagger");
    let giant = t.battlefield(P0, "Hill Giant");
    ready(&mut t, giant);
    let b = t.battlefield(P1, blocker);
    ready(&mut t, b);
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.activate(P0, dagger, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b, giant)]);
    to_end_of_combat(&mut t);
    t.resolve_all();
    (t, dagger, giant, b)
}

#[test]
fn acidic_dagger_destroys_the_non_wall_creature_it_damages_and_is_sacrificed() {
    cr!("603.7b", "603.7c");
    ruling!(
        "Acidic Dagger",
        "does not affect the creature if all damage is prevented or redirected"
    );
    supported("Acidic Dagger");
    // The Wurm (6/4) takes 3 damage and is destroyed; the Giant dies to the Wurm's damage,
    // so the Dagger is sacrificed.
    let (t, dagger, giant, wurm) = dagger_combat("Craw Wurm");
    assert!(!t.on_battlefield(wurm));
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(dagger));
    assert!(t.in_graveyard(P0, "Acidic Dagger"));
}

#[test]
fn acidic_dagger_doesnt_destroy_a_wall() {
    cr!("603.7b");
    let (t, dagger, giant, wall) = dagger_combat("Wall of Stone");
    assert!(t.on_battlefield(wall));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(dagger));
}

fn sand_warriors(t: &TestGame) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Sand"))
        .collect()
}

#[test]
fn hazezon_tamar_counts_lands_as_the_delayed_ability_resolves() {
    cr!("603.7a", "603.7c", "608.2h");
    ruling!(
        "Hazezon Tamar",
        "It doesn't matter who controls Hazezon Tamar at that time, or even if it's on the battlefield"
    );
    supported("Hazezon Tamar");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let hazezon = t.enter(P0, "Hazezon Tamar");
    t.resolve_all();
    // Not the opponent's upkeep; more lands by P0's next upkeep, and Hazezon gone.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(sand_warriors(&t).is_empty());
    t.lands(P0, "Forest", 3);
    let hazezon = t.g.current(hazezon);
    t.g.destroy(hazezon, None);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let warriors = sand_warriors(&t);
    assert_eq!(warriors.len(), 5);
    let c = t.obj_now(warriors[0]).chars.colors;
    assert!(c.contains(Color::Red) && c.contains(Color::Green) && c.contains(Color::White));
    assert_eq!(t.obj_now(warriors[0]).controller, P0);
    // Once.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(sand_warriors(&t).len(), 5);
}

/// Runs P0's turn until `obj` is back under P1's control with nothing waiting, during that
/// turn's cleanup step.
fn until_control_returns(t: &mut TestGame, obj: ObjectId) {
    let ok = t.g.run_until(10_000, |g| {
        g.turn.active != P0
            || (g.obj(obj).controller == P1 && g.stack.is_empty() && g.pending_triggers.is_empty())
    });
    assert!(ok);
    assert_eq!(t.g.turn.active, P0, "control returned during P0's turn");
}

#[test]
fn magus_of_the_unseen_taps_the_artifact_as_control_returns() {
    cr!("603.7c", "603.10a");
    ruling!(
        "Magus of the Unseen",
        "You tap the artifact when you lose control of it for any reason"
    );
    supported("Magus of the Unseen");
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Unseen");
    ready(&mut t, magus);
    let ring = t.battlefield(P1, "Sol Ring");
    t.g.objects[ring.0 as usize].tapped = true;
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.activate(P0, magus, 0, &[Entity::Object(ring)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ring).controller, P0);
    assert!(!t.obj_now(ring).tapped);
    until_control_returns(&mut t, ring);
    assert!(t.obj_now(ring).tapped);
}

#[test]
fn stolen_uniform_unattaches_the_equipment_as_control_returns() {
    cr!("603.7b", "603.7c");
    ruling!(
        "Stolen Uniform",
        "because the \"until end of turn\" duration ended"
    );
    supported("Stolen Uniform");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let split = t.battlefield(P1, "Bonesplitter");
    t.g.attach(split, Entity::Object(theirs));
    t.advance_to(P0, Step::PrecombatMain);
    let uniform = t.hand(P0, "Stolen Uniform");
    mana(&mut t, P0);
    t.cast_with(P0, uniform, &[Entity::Object(bears), Entity::Object(split)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(split).controller, P0);
    assert_eq!(t.obj_now(split).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    until_control_returns(&mut t, split);
    assert_eq!(t.obj_now(split).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn dread_wight_paralyzes_its_blocker_until_the_counter_is_removed() {
    cr!("611.2b", "502.3");
    supported("Dread Wight");
    let mut t = TestGame::new(2);
    let wight = t.battlefield(P0, "Dread Wight");
    ready(&mut t, wight);
    let wall = t.battlefield(P1, "Wall of Stone");
    ready(&mut t, wall);
    declare(&mut t, P0, &[(wight, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, wight)]);
    to_end_of_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.counters(wall, "paralyzation"), 1);
    assert!(t.obj_now(wall).tapped);
    // It doesn't untap during P1's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(wall).tapped);
    // Its granted ability removes the counter; then it untaps as usual.
    mana(&mut t, P1);
    t.activate(P1, wall, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(wall, "paralyzation"), 0);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(wall).tapped);
}


#[test]
fn scars_of_the_veteran_counts_the_damage_prevented_at_the_next_end_step() {
    cr!("615.5", "603.7a", "603.7c");
    supported("Scars of the Veteran");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pyro = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    let scars = t.hand(P0, "Scars of the Veteran");
    mana(&mut t, P0);
    t.cast_with(P0, scars, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    t.g.deal_damage(pyro, Entity::Object(bears), 3, false);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.counters(bears, "+0/+1"), 0, "not before the end step");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+0/+1"), 3);
    assert_eq!(t.pt(bears), (2, 5));
}

#[test]
fn scars_of_the_veteran_on_a_player_puts_no_counters() {
    cr!("615.5");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    let scars = t.hand(P0, "Scars of the Veteran");
    mana(&mut t, P0);
    t.cast_with(P0, scars, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    t.g.deal_damage(pyro, Entity::Player(P0), 9, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "7 of the 9 damage prevented");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.g.delayed_triggers.is_empty());
}
