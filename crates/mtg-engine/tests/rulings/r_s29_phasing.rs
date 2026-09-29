//! Rulings batch S29 — phasing (CR 702.26) with Galadriel's Dismissal ("Kicker {2}{W}.
//! Target creature phases out. If this spell was kicked, each creature target player
//! controls phases out instead."): phased-out permanents phase in as their controller's
//! untap step begins, before untapping (CR 502.1, 702.26a), with their counters, their
//! attached Auras and Equipment, and the choices made as they entered.

use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::{entered_this_turn, tap_for_mana, to_beginning_of_combat};
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::lands_for_cost;
use crate::r_s29_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// P0 casts Galadriel's Dismissal targeting `creature` (unkicked), and it resolves.
fn dismiss(t: &mut TestGame, creature: ObjectId) {
    supported("Galadriel's Dismissal");
    lands_for_cost(t, P0, "Galadriel's Dismissal");
    let d = t.hand(P0, "Galadriel's Dismissal");
    t.cast(P0, d).kicked(false).target(creature).go();
    t.resolve_all();
}

#[test]
fn it_phases_in_before_untapping_with_its_counters_and_can_attack_and_tap() {
    cr!("502.1", "702.26a", "702.26d", "302.6");
    ruling!(
        "Galadriel's Dismissal",
        "Permanents phase back in during their controller's untap step, immediately before that player untaps their permanents. Creatures that phase in this way are able to attack during that turn, and their activated abilities with {T} in their costs can be activated. If a permanent had counters on it when it phased out, it will have those counters when it phases back in."
    );
    // Llanowar Elves entered this turn, tapped, with a +1/+1 counter.
    let mut t = TestGame::new(2);
    let elves = entered_this_turn(&mut t, P0, "Llanowar Elves");
    put_counters(&mut t, elves, counters::PLUS1, 1);
    t.g.objects[elves.0 as usize].tapped = true;
    dismiss(&mut t, elves);
    assert!(phased_out(&t, elves));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(phased_out(&t, elves));
    // P0's untap step: it phases in, then untaps.
    t.advance_to(P0, Step::Upkeep);
    assert!(!phased_out(&t, elves));
    assert!(!t.obj_now(elves).tapped);
    assert_eq!(t.counters(elves, counters::PLUS1), 1);
    assert_eq!(t.pt(elves), (2, 2));
    // It can attack and use its {T} ability this turn.
    assert!(tap_for_mana(&mut t, P0, elves, "Add {G}"));
    t.g.objects[elves.0 as usize].tapped = false;
    to_beginning_of_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(elves, Entity::Player(P1))]));
    attack_with(&mut t, &[(elves, Entity::Player(P1))]);
    assert!(t.g.is_attacking(elves));
}

#[test]
fn kicked_each_creature_target_player_controls_phases_out() {
    cr!("702.33d", "702.26a", "702.26g");
    ruling!(
        "Galadriel's Dismissal",
        "As a permanent is phased out, Auras and Equipment attached to it also phase out at the same time. Those Auras and Equipment will phase in at the same time that creature does, and they'll phase in still attached to that permanent."
    );
    supported("Bonesplitter");
    supported("Holy Strength");
    // Kicked, targeting P1: P1's creatures phase out, with the Equipment and Aura on one;
    // P0's creature doesn't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let splitter = attach_new(&mut t, P1, "Bonesplitter", bears);
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    let mine = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Galadriel's Dismissal");
    t.lands(P0, "Plains", 3);
    let d = t.hand(P0, "Galadriel's Dismissal");
    t.cast(P0, d).kicked(true).target(P1).go();
    t.resolve_all();
    for id in [bears, giant, splitter, aura] {
        assert!(phased_out(&t, id));
    }
    assert!(!phased_out(&t, mine));
    // They phase in together as P1's untap step begins, still attached.
    t.advance_to(P1, Step::Upkeep);
    for id in [bears, giant, splitter, aura] {
        assert!(!phased_out(&t, id));
    }
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
    // Bonesplitter (+2/+0) and Holy Strength (+1/+2).
    assert_eq!(t.pt(bears), (5, 4));
}

#[test]
fn choices_made_as_it_entered_are_remembered_when_it_phases_in() {
    cr!("702.26d", "614.12");
    ruling!(
        "Galadriel's Dismissal",
        "Choices made for permanents as they entered the battlefield are remembered when they phase in."
    );
    supported("Metallic Mimic");
    // Metallic Mimic: "As this creature enters, choose a creature type. This creature is
    // the chosen type in addition to its other types."
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    let mimic = t.enter(P0, "Metallic Mimic");
    t.resolve_all();
    assert!(t.obj_now(mimic).chars.has_subtype("Elf"));
    dismiss(&mut t, mimic);
    assert!(phased_out(&t, mimic));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!phased_out(&t, mimic));
    assert!(t.obj_now(mimic).chars.has_subtype("Elf"));
}
