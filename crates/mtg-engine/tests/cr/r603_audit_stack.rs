//! CR 603.2–603.3 audit: when abilities trigger and how they go on the stack. An event
//! that's replaced or prevented doesn't happen and triggers nothing (603.2g); "becomes"
//! abilities trigger only on a change of state (603.2e); triggers wait for the next time
//! a player would receive priority — above a spell whose cost caused them, after the
//! spell or ability whose resolution caused them (603.3); targets and modes are chosen
//! as the ability is put on the stack, and an ability with no legal choice is removed
//! (603.3c–d).

use crate::r603_audit_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_trigger_with_itself_as_the_only_legal_target_must_target_itself() {
    cr!("603.3d", "601.2c");
    ruling!(
        "Flametongue Kavu",
        "If it’s the only creature on the battlefield as its ability triggers, its ability must target itself."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let fk = t.hand(P0, "Flametongue Kavu");
    t.cast(P0, fk).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Flametongue Kavu"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Flametongue Kavu"));
}

#[test]
fn a_trigger_with_no_legal_target_is_removed_without_asking_its_may() {
    cr!("603.3d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let hunter = t.hand(P0, "Fiend Hunter");
    t.cast(P0, hunter).go();
    let asked_before = t.asked().len();
    t.resolve();
    assert_eq!(t.stack_len(), 0, "no other creature: removed from the stack");
    assert!(
        !t.asked()[asked_before..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::YesNo { .. })),
        "its \"you may\" was never asked"
    );
    assert!(t.on_battlefield(t.g.current(hunter)));
}

#[test]
fn a_modal_triggers_mode_is_chosen_as_it_goes_on_the_stack_among_the_legal_ones() {
    cr!("603.3c");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let prince = t.hand(P0, "Charming Prince");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.cast(P0, prince).go();
    t.resolve();
    // The mode was chosen as the ability went on the stack, before it resolves; the third
    // mode ("another target creature you own") had no legal target.
    let modes: Vec<Vec<usize>> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(modes, vec![vec![0, 1]]);
    assert_eq!(triggers_of(&t, "Charming Prince"), 1);
    assert_eq!(t.life(P0), 20);
    t.resolve();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn an_untap_trigger_triggers_only_for_permanents_that_were_tapped() {
    cr!("603.2e", "502.3");
    ruling!(
        "Mesmeric Orb",
        "If permanents become untapped during the untap step, Mesmeric Orb's ability will trigger that many times."
    );
    supported("Mesmeric Orb");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mesmeric Orb");
    let forests = t.lands(P0, "Forest", 3);
    t.g.tap(forests[0]);
    t.g.tap(forests[1]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.advance_to(P1, Step::Upkeep);
    // P1's untap step: only P1's tapped Bears untapped.
    t.settle();
    assert_eq!(triggers_of(&t, "Mesmeric Orb"), 1);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_of(&t, "Mesmeric Orb"), 2, "two of P0's three Forests");
}

#[test]
fn a_replaced_enter_doesnt_trigger_enters_abilities() {
    cr!("603.2g", "614.1");
    ruling!(
        "Containment Priest",
        "If Containment Priest enters the battlefield without being cast, its ability won't exile itself."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    // The Priest put onto the battlefield isn't exiled by itself, and the Warden sees it.
    t.enter(P1, "Containment Priest");
    t.settle();
    assert_eq!(triggers_of(&t, "Soul Warden"), 1);
    t.resolve_all();
    // A creature card put onto the battlefield without being cast is exiled instead: it
    // never entered, and nothing triggers.
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.g.move_object(bears, Zone::Battlefield, MoveCause::Effect, Some(P1));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_exile("Grizzly Bears"));
    // A cast one enters.
    t.lands(P0, "Forest", 2);
    let cast = t.hand(P0, "Grizzly Bears");
    t.cast(P0, cast).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Soul Warden"), 1);
}

#[test]
fn a_dies_event_replaced_by_exile_triggers_leaves_abilities_but_not_dies_ones() {
    cr!("603.2g", "603.6c", "700.4");
    ruling!(
        "Rest in Peace",
        "While Rest in Peace is on the battlefield, abilities that trigger whenever a creature dies won't trigger because cards and tokens are never put into a player's graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let hunter = t.hand(P0, "Fiend Hunter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, hunter).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // The Hunter is destroyed: exiled instead of dying. Blood Artist doesn't trigger; the
    // Hunter's leaves ability does and returns the Bears.
    t.g.destroy(t.g.current(hunter), None);
    t.settle();
    assert_eq!(triggers_of(&t, "Blood Artist"), 0);
    assert_eq!(triggers_of(&t, "Fiend Hunter"), 1);
    t.resolve_all();
    assert!(t.in_exile("Fiend Hunter"));
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn a_trigger_from_paying_a_cost_goes_on_the_stack_above_the_spell() {
    cr!("603.3", "601.2i");
    ruling!(
        "Mayhem Devil",
        "If a permanent is sacrificed to pay a cost of a spell or ability, Mayhem Devil's ability will resolve before that spell or ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mayhem Devil");
    t.battlefield(P0, "Lotus Petal");
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    assert!(t.in_graveyard(P0, "Lotus Petal"), "the Petal paid for the Bolt");
    t.settle();
    assert_eq!(t.stack.len(), 2);
    assert_eq!(t.stack[0], spell);
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 1, "on top of the Bolt");
}

#[test]
fn a_trigger_during_a_resolution_waits_until_the_spell_has_finished_resolving() {
    cr!("603.3", "608.2n");
    ruling!(
        "Mayhem Devil",
        "Conversely, if a permanent is sacrificed during the resolution of a spell or ability, that spell or ability will finish resolving before Mayhem Devil's ability is put onto the stack."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mayhem Devil");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let edict = t.hand(P0, "Diabolic Edict");
    t.cast(P0, edict).target(P1).go();
    t.g.resolve_top();
    // Nothing is on the stack yet: the Edict finished resolving (it's in the graveyard)
    // and the Devil's ability waits for the next time a player would receive priority.
    assert!(t.in_graveyard(P0, "Diabolic Edict"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 1);
    assert_eq!(trigger_controllers(&t, "Mayhem Devil"), vec![P0]);
}

#[test]
fn a_zone_change_trigger_cant_find_a_card_that_left_the_zone_it_went_to() {
    cr!("603.6", "603.6c", "400.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rancor = t.battlefield(P0, "Rancor");
    t.g.attach(rancor, Entity::Object(bears));
    t.g.recompute();
    t.g.destroy(bears, None);
    // The Rancor goes to the graveyard (state-based action) and its ability triggers.
    t.settle();
    assert_eq!(triggers_of(&t, "Rancor"), 1);
    // In response it's exiled from the graveyard: the ability can't return it.
    let in_gy = t.g.find_in_zone(Zone::Graveyard(P0), "Rancor")[0];
    t.g.exile_object(in_gy, None);
    t.resolve_all();
    assert!(t.in_exile("Rancor"));
    assert!(!t.in_hand(P0, "Rancor"));
    // It left and came back to the graveyard: a new object, which it doesn't find either.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rancor = t.battlefield(P0, "Rancor");
    t.g.attach(rancor, Entity::Object(bears));
    t.g.recompute();
    t.g.destroy(bears, None);
    t.settle();
    let in_gy = t.g.find_in_zone(Zone::Graveyard(P0), "Rancor")[0];
    let exiled = t.g.exile_object(in_gy, None).expect("exiled");
    t.g.move_object(exiled, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Rancor"));
    assert!(!t.in_hand(P0, "Rancor"));
}

#[test]
fn an_ability_triggering_on_enters_abilities_sees_every_wording_of_them() {
    cr!("603.2", "603.6a");
    ruling!(
        "Strict Proctor",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They’re often written as “[Trigger condition], [effect].”"
    );
    supported("Strict Proctor");
    supported("Ichor Wellspring");
    // "When this artifact enters or is put into a graveyard from the battlefield": its
    // entering causes it to trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    let well = t.enter(P0, "Ichor Wellspring");
    t.settle();
    assert_eq!(triggers_of(&t, "Ichor Wellspring"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 1, "a permanent entering caused it");
    t.resolve_all();
    // Its other trigger event isn't a permanent entering.
    t.g.destroy(well, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Ichor Wellspring"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    t.resolve_all();
    // "When this enchantment enters, if you control a creature with power 4 or greater"
    // (an intervening "if").
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.battlefield(P0, "Craw Wurm");
    t.enter(P0, "Garruk's Uprising");
    t.settle();
    assert_eq!(triggers_of(&t, "Garruk's Uprising"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 1);
}

#[test]
fn an_attack_trigger_of_an_enters_or_attacks_ability_isnt_an_entering_trigger() {
    cr!("603.2", "603.6a");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    let titan = t.battlefield(P0, "Sun Titan");
    t.graveyard(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(titan, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(triggers_of(&t, "Sun Titan"), 1, "its attack ability");
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    // Entering, the same ability is an enters ability.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.graveyard(P0, "Grizzly Bears");
    t.enter(P0, "Sun Titan");
    t.settle();
    assert_eq!(triggers_of(&t, "Sun Titan"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 1);
}

#[test]
fn an_enters_from_a_graveyard_ability_is_caused_by_the_permanent_entering() {
    cr!("603.2", "603.6a");
    supported("Strict Proctor");
    supported("Triarch Praetorian");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    let praetorian = t.graveyard(P0, "Triarch Praetorian");
    t.lands(P0, "Swamp", 4);
    let zombify = t.hand(P0, "Zombify");
    let (hand, life) = (t.hand_size(P0) - 1, t.life(P0));
    t.cast(P0, zombify).target(praetorian).go();
    t.resolve();
    // "When this creature enters from a graveyard" is an enters ability: the Proctor
    // sees the permanent that entered, not the card it was in the graveyard.
    assert_eq!(triggers_of(&t, "Triarch Praetorian"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 1);
    // P0 has no mana left for {2}: the Praetorian's ability is countered.
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), life);
}
