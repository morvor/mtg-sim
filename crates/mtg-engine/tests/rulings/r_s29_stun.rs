//! Rulings batch S29 — stun counters (CR 122.1d): "If a permanent with a stun counter on
//! it would become untapped, instead remove a stun counter from it." Nothing becomes
//! untapped, so "becomes untapped" abilities don't trigger; a cost that untaps a
//! permanent ({Q}, "Untap a tapped creature you control") can still be paid with a
//! stunned one (CR 118.3), and a {Q} ability's source isn't one of the other permanents
//! its cost untaps.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::run_from;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::ability::{Effect, Sel};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const STUN: &str = "stun";

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn becomes_untapped_abilities_dont_trigger_when_a_stun_counter_is_removed() {
    cr!("122.1d", "603.2", "502.3");
    ruling!(
        "Impede Momentum",
        "Abilities that trigger when a permanent “becomes untapped” won’t trigger if a stun counter is removed instead."
    );
    supported("Impede Momentum");
    supported("Kragma Butcher");
    // Impede Momentum: "Tap target creature and put three stun counters on it. Scry 1."
    // Kragma Butcher (2/3): "Inspired — Whenever this creature becomes untapped, it gets
    // +2/+0 until end of turn."
    let mut t = TestGame::new(2);
    let butcher = t.battlefield(P0, "Kragma Butcher");
    cast_and_resolve(&mut t, P0, "Impede Momentum", &[Entity::Object(butcher)]);
    assert!(tapped(&t, butcher));
    assert_eq!(t.counters(butcher, STUN), 3);
    // An untap effect: a stun counter is removed instead; no trigger.
    run_from(
        &mut t,
        P0,
        None,
        Effect::Untap {
            what: Sel::Target(0),
        },
        &[Entity::Object(butcher)],
    );
    assert!(tapped(&t, butcher));
    assert_eq!(t.counters(butcher, STUN), 2);
    assert_eq!(triggers_on_stack(&t, "becomes untapped"), 0);
    // P0's untap step: another one is removed; no trigger.
    next_upkeep(&mut t, P0);
    assert!(tapped(&t, butcher));
    assert_eq!(t.counters(butcher, STUN), 1);
    assert_eq!(triggers_on_stack(&t, "becomes untapped"), 0);
    next_upkeep(&mut t, P0);
    assert_eq!(t.counters(butcher, STUN), 0);
    assert_eq!(triggers_on_stack(&t, "becomes untapped"), 0);
    assert_eq!(t.pt(butcher), (2, 3));
    // With no stun counters left, it untaps: now it triggers.
    next_upkeep(&mut t, P0);
    assert!(!tapped(&t, butcher));
    assert_eq!(triggers_on_stack(&t, "becomes untapped"), 1);
    t.resolve_all();
    assert_eq!(t.pt(butcher), (4, 3));
}

#[test]
fn an_untap_cost_can_be_paid_with_a_stunned_permanent() {
    cr!("122.1d", "118.3", "601.2h");
    ruling!(
        "Frostfist Strider",
        "If untapping a permanent is part of a cost (such as that of Halo Fountain’s first ability), you may pay that cost by “untapping” a tapped permanent with a stun counter on it. The stun counter will be removed and the creature will remain tapped. However, the cost will still be paid."
    );
    supported("Frostfist Strider");
    supported("Halo Fountain");
    // P1's Frostfist Strider enters: "tap target creature an opponent controls and put a
    // stun counter on it." P0's Grizzly Bears is stunned.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.enter(P1, "Frostfist Strider");
    t.resolve_all();
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
    // Halo Fountain: "{W}, {T}, Untap a tapped creature you control: Create a 1/1 green
    // and white Citizen creature token."
    t.set_step(P0, Step::PrecombatMain);
    let fountain = t.battlefield(P0, "Halo Fountain");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, fountain, "Create a 1/1").expect("cost paid");
    // The stun counter was removed; the Bears stayed tapped; the cost was paid.
    assert_eq!(t.counters(bears, STUN), 0);
    assert!(tapped(&t, bears));
    assert!(tapped(&t, fountain));
    t.resolve_all();
    assert_eq!(
        crate::r_s01_common::with_subtype(&t, P0, "Citizen").len(),
        1
    );
    // The same for the untap symbol: Order of Whiteclay ("{1}{W}{W}, {Q}: Return target
    // creature card with mana value 3 or less from your graveyard to the battlefield."),
    // tapped with a stun counter, pays {Q}.
    supported("Order of Whiteclay");
    let mut t = TestGame::new(2);
    let order = t.battlefield(P0, "Order of Whiteclay");
    t.g.objects[order.0 as usize].tapped = true;
    put_counters(&mut t, order, STUN, 1);
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.activate(P0, order, 0, &[Entity::Object(dead)])
        .expect("{Q} paid with a stunned Order");
    assert_eq!(t.counters(order, STUN), 0);
    assert!(tapped(&t, order));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Without a stun counter, the tapped Order untaps to pay it (and can't pay it again
    // while untapped).
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.activate(P0, order, 0, &[Entity::Object(dead)])
        .expect("{Q} paid");
    assert!(!tapped(&t, order));
    t.resolve_all();
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    assert!(t.activate(P0, order, 0, &[Entity::Object(dead)]).is_err());
}

#[test]
fn crackleburr_needs_two_other_creatures_for_its_untap_ability() {
    cr!("118.3", "302.6");
    ruling!(
        "Crackleburr",
        "To activate either ability, you'll need Crackleburr plus two other creatures. Crackleburr must have been under your control since your most recent turn began (or have haste), but the other two creatures don't."
    );
    supported("Crackleburr");
    // "{U/R}{U/R}, {Q}, Untap two tapped blue creatures you control: Return target
    // creature to its owner's hand." Crackleburr is blue itself, but it's untapped by {Q}.
    let mut t = TestGame::new(2);
    let crackleburr = t.battlefield(P0, "Crackleburr");
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    for id in [crackleburr, merfolk] {
        t.g.objects[id.0 as usize].tapped = true;
    }
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, crackleburr));
    // A second tapped blue creature, which came under P0's control this turn.
    let sick = t.battlefield_sick(P0, "Coral Merfolk");
    t.g.objects[sick.0 as usize].tapped = true;
    assert!(crate::r_s02_common::can_activate(&mut t, P0, crackleburr));
    t.answer_targets(P0, &[Entity::Object(target)]);
    activate_containing(&mut t, P0, crackleburr, "Return target").expect("activated");
    for id in [crackleburr, merfolk, sick] {
        assert!(!tapped(&t, id));
    }
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Crackleburr itself must have been under P0's control since the turn began.
    let mut t = TestGame::new(2);
    let crackleburr = t.battlefield_sick(P0, "Crackleburr");
    let merfolk: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Coral Merfolk")).collect();
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    for id in merfolk.iter().chain([&crackleburr]) {
        t.g.objects[id.0 as usize].tapped = true;
    }
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, crackleburr));
}

#[test]
fn a_stunned_crackleburr_still_isnt_one_of_the_creatures_it_untaps() {
    cr!("118.3", "122.1d");
    ruling!(
        "Crackleburr",
        "To activate either ability, you'll need Crackleburr plus two other creatures."
    );
    // A tapped Crackleburr with two stun counters pays {Q} by losing one (it stays
    // tapped); it still isn't offered as one of the two tapped blue creatures to untap.
    let mut t = TestGame::new(2);
    let crackleburr = t.battlefield(P0, "Crackleburr");
    let merfolk: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Coral Merfolk")).collect();
    let target = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    for id in merfolk.iter().chain([&crackleburr]) {
        t.g.objects[id.0 as usize].tapped = true;
    }
    put_counters(&mut t, crackleburr, STUN, 2);
    let from = t.asked().len();
    t.answer_choose(
        P0,
        &[Entity::Object(crackleburr), Entity::Object(merfolk[0])],
    );
    t.answer_targets(P0, &[Entity::Object(target)]);
    activate_containing(&mut t, P0, crackleburr, "Return target").expect("activated");
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("untap") => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(!offered[0].contains(&Entity::Object(crackleburr)));
    // {Q} removed one stun counter; both Merfolk were untapped.
    assert_eq!(t.counters(crackleburr, STUN), 1);
    assert!(tapped(&t, crackleburr));
    for id in &merfolk {
        assert!(!tapped(&t, *id));
    }
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}
