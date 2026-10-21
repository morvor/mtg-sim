//! Trigger events (cards3-triggers-durations): events caused by a spell or ability
//! (CR 603.2, 608.2), exact amounts, untap-step untaps, objects and players linked to the
//! source (CR 607.2a), and the other trigger events of `trigger_grammar_4.rs`.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn psychogenic_probe_shuffles_caused_by_abilities() {
    cr!("603.2", "701.24a");
    ruling!(
        "Psychogenic Probe",
        "This ability will trigger even if the player’s library is empty at the time they are supposed to shuffle it."
    );
    supported("Psychogenic Probe");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Psychogenic Probe");
    // An activated ability's shuffle (P0's library is empty).
    let elder = t.battlefield(P0, "Sakura-Tribe Elder");
    t.activate(P0, elder, 0, &[]).expect("activate");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // A shuffle no spell or ability performs (a mulligan's, say) doesn't trigger it.
    t.g.shuffle_library(P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn widespread_panic_only_its_controllers_shuffles() {
    cr!("603.2");
    ruling!(
        "Widespread Panic",
        "Widespread Panic’s ability is put onto the stack after the library is shuffled."
    );
    supported("Widespread Panic");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Widespread Panic");
    // P0's Path to Exile makes P1 search and shuffle: not its controller's.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Forest");
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Path to Exile", &[Entity::Object(bears)]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "top of their library"), 0);
    // P1's own ability makes P1 shuffle: P1 puts a card from hand on top.
    t.hand(P1, "Island");
    let elder = t.battlefield(P1, "Sakura-Tribe Elder");
    t.activate(P1, elder, 0, &[]).expect("activate");
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "top of their library"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn sacred_ground_returns_lands_an_opponent_destroys() {
    cr!("603.2", "701.8a");
    supported("Sacred Ground");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Sacred Ground");
    t.battlefield(P0, "Forest");
    let forest = t.named_on_battlefield("Forest")[0];
    cast_new(&mut t, P1, "Stone Rain", &[Entity::Object(forest)]);
    t.resolve();
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // P0's own spell: it stays in the graveyard.
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.named_on_battlefield("Forest")[0];
    cast_new(&mut t, P0, "Stone Rain", &[Entity::Object(forest)]);
    t.resolve();
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn firesong_and_sunspeaker_white_spell_life_gain() {
    cr!("603.2", "119.3");
    ruling!(
        "Firesong and Sunspeaker",
        "Firesong and Sunspeaker's last ability doesn't trigger if a triggered ability of a white instant or sorcery spell or card causes you to gain life, such as the triggered ability of Renewed Faith when it's cycled."
    );
    supported("Firesong and Sunspeaker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Renewed Faith", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "3 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Cycling it: its triggered ability's life gain doesn't trigger Firesong.
    let faith = t.hand(P0, "Renewed Faith");
    t.lands(P0, "Plains", 2);
    t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.activate(P0, faith, 0, &[]).expect("cycle");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ghyrson_starn_exactly_one_damage() {
    cr!("603.2", "603.2c");
    ruling!(
        "Ghyrson Starn, Kelermorph",
        "If another source deals exactly 1 damage to a permanent or player multiple times or deals exactly 1 damage to multiple permanents and/or players simultaneously"
    );
    supported("Ghyrson Starn, Kelermorph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghyrson Starn, Kelermorph");
    // "{T}: This creature deals 1 damage to any target."
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).expect("ping");
    t.settle();
    t.resolve_all();
    // 1 from the Sorcerer, 2 from Ghyrson (whose own 2 damage doesn't trigger it).
    assert_eq!(t.life(P1), 17);
    // A permanent: Ghyrson deals its 2 damage to that creature.
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.obj_mut(sorcerer).tapped = false;
    t.activate(P0, sorcerer, 0, &[Entity::Object(giant)]).expect("ping");
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
    // Not exactly 1: no trigger.
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn ob_nixilis_opponents_each_lose_exactly_one() {
    cr!("603.2c", "119.3");
    ruling!(
        "Ob Nixilis, Captive Kingpin",
        "Ob Nixilis's last ability will trigger if an opponent is dealt exactly 1 damage, pays exactly 1 life, or loses exactly 1 life some other way."
    );
    supported("Ob Nixilis, Captive Kingpin");
    let mut t = TestGame::new(3);
    let ob = t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    // "Whenever this creature or another creature you control dies, each opponent loses
    // 1 life and you gain 1 life.": both opponents lose exactly 1 at once: once.
    t.battlefield(P0, "Zulaport Cutthroat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(ob, "+1/+1"), 1);
    // 3 life at once: no trigger.
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(ob, "+1/+1"), 1);
}

#[test]
fn millennium_calendar_counts_permanents_untapped_in_your_untap_step() {
    cr!("502.3", "603.2c");
    ruling!(
        "The Millennium Calendar",
        "The Millenium Calendar's first ability triggers during the untap step."
    );
    supported("The Millennium Calendar");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let cal = t.battlefield(P0, "The Millennium Calendar");
    for l in t.lands(P0, "Forest", 3) {
        t.g.obj_mut(l).tapped = true;
    }
    // An opponent's tapped permanent isn't untapped by you.
    let theirs = t.battlefield(P1, "Forest");
    t.g.obj_mut(theirs).tapped = true;
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(cal, "time"), 3);
    // Untapping outside the untap step doesn't trigger it.
    let land = t.named_on_battlefield("Forest")[0];
    t.g.obj_mut(land).tapped = true;
    t.g.untap(land);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ninth_doctor_additional_upkeep_after_the_upkeep() {
    cr!("502.3", "500.9");
    ruling!(
        "The Ninth Doctor",
        "The additional upkeep is taken after that step, not after the untap step."
    );
    supported("The Ninth Doctor");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let doc = t.battlefield(P0, "The Ninth Doctor");
    t.g.obj_mut(doc).tapped = true;
    t.advance_to(P0, Step::Draw);
    // The trigger resolved in the upkeep: a second upkeep followed it, in the same turn.
    assert_eq!(t.g.turn.upkeeps, 2);
    assert_eq!(t.g.turn.step_log.iter().filter(|s| **s == Step::Upkeep).count(), 2);
    // An untapped Doctor doesn't become untapped: one upkeep the next turn.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.g.turn.upkeeps, 1);
}

#[test]
fn diabolic_servitude_tracks_the_creature_it_returned() {
    cr!("607.2a", "603.10a", "400.7");
    supported("Diabolic Servitude");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Diabolic Servitude");
    t.settle();
    t.resolve_all();
    let returned = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(returned.len(), 1);
    // Another creature dying doesn't trigger it.
    let other = t.battlefield(P0, "Hill Giant");
    crate::r_s02_common::destroy(&mut t, other);
    assert_eq!(t.stack_len(), 0);
    // The returned creature dies: it's exiled and Servitude returns to its owner's hand.
    crate::r_s02_common::destroy(&mut t, returned[0]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P0, "Diabolic Servitude"));
}

#[test]
fn diabolic_servitude_leaving_exiles_the_creature() {
    cr!("607.2a");
    ruling!(
        "Diabolic Servitude",
        "If the creature is on the battlefield and is returned to its owner’s hand or is exiled, this card loses track of the creature and has no further effects in the future."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let servitude = t.enter(P0, "Diabolic Servitude");
    t.settle();
    t.resolve_all();
    crate::r_s02_common::destroy(&mut t, servitude);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Returned to hand and recast: a new object Servitude doesn't track.
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let servitude = t.enter(P0, "Diabolic Servitude");
    t.settle();
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant")[0];
    let c = t.g.current(g);
    t.g.move_object(c, mtg_engine::object::Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    let g2 = t.hand(P0, "Hill Giant");
    let _ = g2;
    t.settle();
    crate::r_s02_common::destroy(&mut t, servitude);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn havengul_mystery_transforms_back_when_its_creature_leaves() {
    cr!("607.2a", "701.27a");
    supported("Havengul Laboratory");
    let mut t = TestGame::new(2);
    let lab = t.battlefield(P0, "Havengul Laboratory");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(mtg_engine::dfc::transform(&mut t.g, lab));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(lab).chars.name.as_str(), "Havengul Mystery");
    let returned = t.named_on_battlefield("Grizzly Bears")[0];
    // It leaves the battlefield (bounced): Havengul Mystery transforms.
    let c = t.g.current(returned);
    t.g.move_object(c, mtg_engine::object::Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(lab).chars.name.as_str(), "Havengul Laboratory");
}
