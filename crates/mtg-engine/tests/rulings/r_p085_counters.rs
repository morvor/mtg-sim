//! Rulings batch P085 — hosers of green and +1/+1 counters: Blightbeetle's "can't have
//! +1/+1 counters put on them", Dunerider Outlaw's end-step check, Rank and File's locked-in
//! set of affected creatures, and Water Wurm's single bonus.

use crate::r_p085_common::*;
use crate::r_s01_common::custom_card;
use crate::r_s06_common::give_control;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn blightbeetle_existing_and_entering_counters() {
    cr!("614.1c", "122.1");
    ruling!(
        "Blightbeetle",
        "Blightbeetle doesn’t remove any +1/+1 counters already on creatures your opponents control."
    );
    ruling!(
        "Blightbeetle",
        "Creatures your opponents control won’t receive +1/+1 counters as they enter the battlefield."
    );
    supported("Blightbeetle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.battlefield(P0, "Blightbeetle");
    t.settle();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
    // Spike Feeder (0/0, enters with two +1/+1 counters) enters without them and dies.
    let feeder = t.enter(P1, "Spike Feeder");
    t.settle();
    assert_eq!(t.zone(feeder), Zone::Graveyard(P1));
    // P0's own creatures still get them.
    let mine = t.enter(P0, "Spike Feeder");
    t.settle();
    assert_eq!(t.counters(mine, "+1/+1"), 2);
}

#[test]
fn blightbeetle_replacement_effects_that_add_counters() {
    cr!("614.1c", "615.1", "616.1");
    ruling!(
        "Blightbeetle",
        "If the original event is entirely replaced (such as by applying Vigor’s replacement effect), the entire original event simply doesn’t happen."
    );
    supported("Vigor");
    // Vigor prevents the damage to P1's Bears; the replacing counters can't be put.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blightbeetle");
    t.battlefield(P1, "Vigor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    // Hardened Scales may modify the event, but no counters are put.
    supported("Hardened Scales");
    supported("Oath of the Ancient Wood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blightbeetle");
    t.battlefield(P1, "Hardened Scales");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.answer_yes(P1, true);
    t.enter(P1, "Oath of the Ancient Wood");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn blightbeetle_costs_and_may_put() {
    cr!("614.17b", "118.3", "608.2d");
    ruling!(
        "Blightbeetle",
        "If the cost of an ability or an additional cost of a spell requires putting +1/+1 counters on a creature affected by Blightbeetle, that cost can’t be paid."
    );
    // "You may put a +1/+1 counter on target creature": P1 can't choose to put one on
    // their own creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blightbeetle");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.answer_yes(P1, true);
    t.enter(P1, "Oath of the Ancient Wood");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    // A cost of putting a +1/+1 counter on an affected creature can't be paid.
    let def = custom_card(
        "Counter Eater",
        "Creature — Beast",
        "{1}",
        Some((1, 1)),
        "Put a +1/+1 counter on Counter Eater: You gain 1 life.",
    );
    let eater = t.custom(P1, def.clone(), Zone::Battlefield);
    assert!(t.activate(P1, eater, 0, &[]).is_err());
    assert_eq!(t.life(P1), 20);
    // Without Blightbeetle's effect on it (P0's own), it can.
    let mine = t.custom(P0, def, Zone::Battlefield);
    t.activate(P0, mine, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
}

/// P0's Dunerider Outlaw deals 1 damage to each of `hit`, then `new_controller` (if any)
/// gains control of it, and the game goes to the end step: the +1/+1 counters it has then.
fn outlaw_counters(players: usize, hit: &[PlayerId], new_controller: Option<PlayerId>) -> u32 {
    supported("Dunerider Outlaw");
    let mut t = TestGame::new(players);
    let outlaw = t.battlefield(P0, "Dunerider Outlaw");
    for p in hit {
        t.g.deal_damage(outlaw, Entity::Player(*p), 1, false);
    }
    t.g.flush_events();
    if let Some(p) = new_controller {
        give_control(&mut t, outlaw, p);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.counters(outlaw, "+1/+1")
}

#[test]
fn dunerider_outlaw_checks_current_controllers_opponents() {
    cr!("603.4", "102.2");
    ruling!(
        "Dunerider Outlaw",
        "If it dealt damage to player A, then player A takes control of it, the ability doesn't trigger."
    );
    assert_eq!(outlaw_counters(2, &[P1], None), 1);
    assert_eq!(outlaw_counters(2, &[P1], Some(P1)), 0);
    // Multiplayer: P1 is still an opponent of P2.
    assert_eq!(outlaw_counters(3, &[P1], Some(P2)), 1);
}

#[test]
fn dunerider_outlaw_one_counter_per_turn() {
    cr!("603.4");
    ruling!(
        "Dunerider Outlaw",
        "gives it at most one +1/+1 counter each turn, regardless of how many opponents were dealt damage"
    );
    assert_eq!(outlaw_counters(3, &[P1, P2], None), 1);
}

#[test]
fn rank_and_file_affects_only_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Rank and File",
        "It will not apply to green creatures that enter later in the turn."
    );
    supported("Rank and File");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Rank and File");
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(t.pt(giant), (3, 3), "a red creature was affected");
    let later = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(later), (2, 2));
}

#[test]
fn water_wurm_bonus_once() {
    cr!("613.4c");
    ruling!(
        "Water Wurm",
        "Only gets the bonus once even if more than one opponent has an Island on the battlefield."
    );
    supported("Water Wurm");
    let mut t = TestGame::new(3);
    let wurm = t.battlefield(P0, "Water Wurm");
    t.battlefield(P1, "Island");
    t.battlefield(P2, "Island");
    t.battlefield(P2, "Island");
    t.settle();
    assert_eq!(t.pt(wurm), (1, 2));
}
