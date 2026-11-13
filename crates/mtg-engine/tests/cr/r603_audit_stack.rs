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
    interp!("ability-triggered-cause-is-the-event");
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
    interp!("ability-triggered-cause-is-the-event");
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
    interp!("ability-triggered-cause-is-the-event");
    supported("Strict Proctor");
    supported("Triarch Praetorian");
    supported("Desecrated Tomb");
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
    // Desecrated Tomb's "Whenever one or more creature cards leave your graveyard" isn't
    // caused by the permanent entering: the card leaving the graveyard and the permanent
    // entering are two separate things, and the Tomb's trigger event is the leaving (the
    // user's decision of Nov 12).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.battlefield(P0, "Desecrated Tomb");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let zombify = t.hand(P0, "Zombify");
    t.cast(P0, zombify).target(bears).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Desecrated Tomb"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Bat"), 1);
}

#[test]
fn strict_proctor_doesnt_tax_a_leaves_your_graveyard_ability_on_a_reanimation() {
    cr!("603.2");
    interp!("ability-triggered-cause-is-the-event");
    supported("Strict Proctor");
    supported("Desecrated Tomb");
    // P0 could pay {2} (six Swamps for Zombify's {3}{B}), but isn't asked to: the Tomb's
    // ability matched the Bears leaving the graveyard, not a permanent entering.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.battlefield(P0, "Desecrated Tomb");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    let zombify = t.hand(P0, "Zombify");
    t.cast(P0, zombify).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(tokens_named(&t, P0, "Bat"), 1, "the Bat is made");
    let asked_to_pay = t.asked().into_iter().any(|(p, d)| {
        p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay"))
    });
    assert!(!asked_to_pay, "no {{2}} is asked");
}

#[test]
fn strict_proctor_judges_which_of_several_trigger_events_matched_a_reanimation() {
    cr!("603.2", "603.2c");
    interp!("ability-triggered-cause-is-the-event");
    use crate::r600_common::{gain, trig, CB};
    use mtg_engine::ability::{Body, Filter, PlayerRel, TriggerCond, ZoneKind};
    supported("Strict Proctor");
    supported("Syr Konrad, the Grim");
    // Syr Konrad, the Grim: "Whenever another creature dies, or a creature card is put into
    // a graveyard from anywhere other than the battlefield, or a creature card leaves your
    // graveyard, Syr Konrad deals 1 damage to each opponent." A reanimation matches only
    // its last trigger event, the card leaving the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.battlefield(P0, "Syr Konrad, the Grim");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let zombify = t.hand(P0, "Zombify");
    t.cast(P0, zombify).target(bears).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Syr Konrad, the Grim"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    t.resolve_all();
    assert_eq!(
        t.life(P1),
        19,
        "not countered (P0 had no mana left for {{2}})"
    );
    // "Whenever a creature card leaves your graveyard or a creature enters": a reanimation
    // matches both trigger events and triggers it once (603.2c). The entering is one of
    // them, so a permanent entering caused it to trigger, whichever the text names first.
    let leaves = TriggerCond::ZoneChange {
        filter: Filter::and(vec![Filter::creature(), Filter::OwnedBy(PlayerRel::You)]),
        from: Some(ZoneKind::Graveyard),
        to: None,
    };
    let enters = TriggerCond::EntersBattlefield(Filter::creature());
    for conds in [
        vec![leaves.clone(), enters.clone()],
        vec![enters.clone(), leaves.clone()],
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Strict Proctor");
        let watcher = CB::new("Two-Way Watcher")
            .enchantment()
            .ability(trig(TriggerCond::AnyOf(conds), Body::effect(gain(1))))
            .build();
        t.custom(P0, watcher, Zone::Battlefield);
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let zombify = t.hand(P0, "Zombify");
        t.cast(P0, zombify).target(bears).go();
        t.resolve();
        assert_eq!(triggers_of(&t, "Two-Way Watcher"), 1);
        assert_eq!(triggers_of(&t, "Strict Proctor"), 1);
    }
}

#[test]
fn strict_proctor_doesnt_count_an_ability_that_matched_a_card_leaving_any_zone() {
    cr!("603.2");
    interp!("ability-triggered-cause-is-the-event");
    use crate::r600_common::{gain, trig, CB};
    use mtg_engine::ability::{Body, Filter, PlayerRel, TriggerCond, ZoneKind};
    supported("Strict Proctor");
    // "Whenever a card leaves your hand" (library, exile), and Grizzly Bears put onto the
    // battlefield from that zone by an effect: the ability's trigger event is the card
    // leaving the zone it came from, not the permanent entering, whatever the zone.
    for from in [ZoneKind::Hand, ZoneKind::Library, ZoneKind::Exile] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Strict Proctor");
        let leaves = TriggerCond::ZoneChange {
            filter: Filter::OwnedBy(PlayerRel::You),
            from: Some(from),
            to: None,
        };
        let watcher = CB::new("Departure Watcher")
            .enchantment()
            .ability(trig(leaves, Body::effect(gain(1))))
            .build();
        t.custom(P0, watcher, Zone::Battlefield);
        let bears = match from {
            ZoneKind::Hand => t.hand(P0, "Grizzly Bears"),
            ZoneKind::Library => t.library_top(P0, "Grizzly Bears"),
            _ => t.exile(P0, "Grizzly Bears"),
        };
        let now =
            t.g.move_object(bears, Zone::Battlefield, MoveCause::Effect, Some(P0))
                .expect("moved");
        t.settle();
        assert!(t.on_battlefield(now), "{from:?}");
        assert_eq!(triggers_of(&t, "Departure Watcher"), 1, "{from:?}");
        assert_eq!(triggers_of(&t, "Strict Proctor"), 0, "{from:?}");
        t.resolve_all();
        assert_eq!(t.life(P0), 21, "{from:?}");
    }
    // A trigger event qualified by a condition: Kishla Skimmer's "Whenever a card leaves
    // your graveyard during your turn, draw a card. This ability triggers only once each
    // turn." on a reanimation.
    supported("Kishla Skimmer");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.battlefield(P0, "Kishla Skimmer");
    t.library_top(P0, "Island");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let zombify = t.hand(P0, "Zombify");
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, zombify).target(bears).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Kishla Skimmer"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "not countered");
}

/// P0 reanimates Elvish Visionary ("When this creature enters, draw a card.") with
/// Zombify while `setup` is on the battlefield and P0 controls Desecrated Tomb ("Whenever
/// one or more creature cards leave your graveyard, create a 1/1 black Bat creature token
/// with flying."). Returns how many Bats P0 made and how many cards the Visionary's
/// ability drew.
fn reanimate_with_tomb(setup: &[(PlayerId, &str)]) -> (usize, usize) {
    let mut t = TestGame::new(2);
    for (p, name) in setup {
        t.battlefield(*p, name);
    }
    t.battlefield(P0, "Desecrated Tomb");
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let elf = t.graveyard(P0, "Elvish Visionary");
    t.lands(P0, "Swamp", 4);
    let zombify = t.hand(P0, "Zombify");
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, zombify).target(elf).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(elf)), "{setup:?}");
    (tokens_named(&t, P0, "Bat"), t.hand_size(P0) - hand)
}

#[test]
fn torpor_orb_and_its_family_dont_stop_a_leaves_your_graveyard_ability_on_a_reanimation() {
    cr!("603.2", "603.6a");
    interp!("ability-triggered-cause-is-the-event");
    ruling!(
        "Torpor Orb",
        "Torpor Orb stops a creature's own enters-the-battlefield triggered abilities as well as other triggered abilities that would trigger when a creature enters the battlefield."
    );
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Elesh Norn's second and third ability each affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    supported("Torpor Orb");
    supported("Hushbringer");
    supported("Elesh Norn, Mother of Machines");
    supported("Desecrated Tomb");
    supported("Elvish Visionary");
    // The card leaving the graveyard and the permanent entering are two separate things
    // (decided by the user, Nov 12): "Creatures entering don't cause abilities to trigger"
    // stops the reanimated Visionary's own enters ability, but not Desecrated Tomb's,
    // whose trigger event is the card leaving the graveyard.
    assert_eq!(reanimate_with_tomb(&[]), (1, 1), "nothing in the way");
    assert_eq!(reanimate_with_tomb(&[(P0, "Torpor Orb")]), (1, 0));
    // Hushbringer ("Creatures entering or dying don't cause abilities to trigger.").
    assert_eq!(reanimate_with_tomb(&[(P1, "Hushbringer")]), (1, 0));
    // An opponent's Elesh Norn: "Permanents entering don't cause abilities of permanents
    // your opponents control to trigger."
    assert_eq!(
        reanimate_with_tomb(&[(P1, "Elesh Norn, Mother of Machines")]),
        (1, 0)
    );
}

#[test]
fn panharmonicon_yarok_and_elesh_norn_dont_double_a_leaves_your_graveyard_ability() {
    cr!("603.2", "603.2d", "603.6a");
    interp!("ability-triggered-cause-is-the-event");
    ruling!(
        "Panharmonicon",
        "Panharmonicon affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    ruling!(
        "Yarok, the Desecrated",
        "Yarok affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Elesh Norn's second and third ability each affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    supported("Panharmonicon");
    supported("Yarok, the Desecrated");
    supported("Elesh Norn, Mother of Machines");
    supported("Desecrated Tomb");
    supported("Elvish Visionary");
    // "If an artifact or creature entering causes a triggered ability of a permanent you
    // control to trigger, that ability triggers an additional time": the Visionary's own
    // enters ability triggers twice, Desecrated Tomb's once (decided by the user, Nov 12).
    for statik in [
        "Panharmonicon",
        "Yarok, the Desecrated",
        "Elesh Norn, Mother of Machines",
    ] {
        assert_eq!(reanimate_with_tomb(&[(P0, statik)]), (1, 2), "{statik}");
    }
}

#[test]
fn a_single_leaves_your_graveyard_ability_is_judged_like_a_one_or_more_one() {
    cr!("603.2", "603.2d");
    interp!("ability-triggered-cause-is-the-event");
    supported("Torpor Orb");
    supported("Panharmonicon");
    supported("Syr Konrad, the Grim");
    // Syr Konrad, the Grim's "... or a creature card leaves your graveyard, Syr Konrad
    // deals 1 damage to each opponent" triggers once per card (not "one or more"): neither
    // Torpor Orb nor Panharmonicon changes it on a reanimation.
    for statik in ["Torpor Orb", "Panharmonicon"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, statik);
        t.battlefield(P0, "Syr Konrad, the Grim");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let zombify = t.hand(P0, "Zombify");
        t.cast(P0, zombify).target(bears).go();
        t.resolve();
        assert_eq!(triggers_of(&t, "Syr Konrad, the Grim"), 1, "{statik}");
        t.resolve_all();
        assert_eq!(t.life(P1), 19, "{statik}");
    }
}

#[test]
fn an_enters_from_a_graveyard_ability_is_still_doubled_or_stopped_on_a_reanimation() {
    cr!("603.2d", "603.6a");
    interp!("ability-triggered-cause-is-the-event");
    supported("Torpor Orb");
    supported("Panharmonicon");
    supported("Triarch Praetorian");
    // Triarch Praetorian's "When this creature enters from a graveyard, you draw two cards
    // and you lose 2 life" names the zone the card came from, but its trigger event is the
    // permanent entering: Panharmonicon makes it trigger twice and Torpor Orb stops it.
    for (statik, n) in [("Panharmonicon", 2), ("Torpor Orb", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, statik);
        let praetorian = t.graveyard(P0, "Triarch Praetorian");
        t.lands(P0, "Swamp", 4);
        let zombify = t.hand(P0, "Zombify");
        t.cast(P0, zombify).target(praetorian).go();
        t.resolve();
        assert!(t.on_battlefield(t.g.current(praetorian)), "{statik}");
        assert_eq!(triggers_of(&t, "Triarch Praetorian"), n, "{statik}");
    }
}

#[test]
fn torpor_orb_judges_delayed_abilities_by_the_trigger_event_that_matched() {
    cr!("603.2", "603.7");
    interp!("ability-triggered-cause-is-the-event");
    use crate::r600_common::{gain, CB};
    use mtg_engine::ability::{BatchPer, Body, Effect, Filter, PlayerRel, TriggerCond, ZoneKind};
    supported("Torpor Orb");
    // "This turn, whenever a creature enters" (or "a creature card leaves your graveyard"),
    // per event or "one or more", "you gain 1 life", and Grizzly Bears reanimated: under
    // Torpor Orb, only the abilities whose trigger event is the card leaving the graveyard.
    let enters = TriggerCond::EntersBattlefield(Filter::creature());
    let leaves = TriggerCond::ZoneChange {
        filter: Filter::and(vec![Filter::creature(), Filter::OwnedBy(PlayerRel::You)]),
        from: Some(ZoneKind::Graveyard),
        to: None,
    };
    for (cond, entering) in [(enters, true), (leaves, false)] {
        for batched in [false, true] {
            for orb in [false, true] {
                let mut t = TestGame::new(2);
                if orb {
                    t.battlefield(P0, "Torpor Orb");
                }
                let trigger = if batched {
                    TriggerCond::Batched {
                        trigger: Box::new(cond.clone()),
                        per: BatchPer::Batch,
                    }
                } else {
                    cond.clone()
                };
                let rite = CB::new("Watchful Rite")
                    .sorcery()
                    .cost("{0}")
                    .spell(Body::effect(Effect::DelayedTrigger {
                        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
                        body: Box::new(Body::effect(gain(1))),
                        once: false,
                    }))
                    .build();
                let rite = t.custom(P0, rite, Zone::Hand(P0));
                t.cast(P0, rite).go();
                t.resolve();
                let bears = t.graveyard(P0, "Grizzly Bears");
                t.lands(P0, "Swamp", 4);
                let zombify = t.hand(P0, "Zombify");
                t.cast(P0, zombify).target(bears).go();
                t.resolve_all();
                let case = format!("entering: {entering}, batched: {batched}, orb: {orb}");
                assert!(t.on_battlefield(t.g.current(bears)), "{case}");
                let gained = if orb && entering { 0 } else { 1 };
                assert_eq!(t.life(P0), 20 + gained, "{case}");
            }
        }
    }
}

#[test]
fn aboleth_spawn_and_firebender_ascension_judge_the_trigger_event_that_matched() {
    cr!("603.2", "603.6a");
    interp!("ability-triggered-cause-is-the-event");
    use crate::r600_common::{gain, trig, CB};
    use mtg_engine::ability::{Body, Filter, PlayerRel, TriggerCond, ZoneKind};
    supported("Aboleth Spawn");
    supported("Firebender Ascension");
    supported("Sun Titan");
    // Aboleth Spawn copies an ability of a creature entering under an opponent's control
    // only if the entering made it trigger: an enters ability, but not the creature's
    // "Whenever a card leaves your hand", which the permanent sees itself match (it doesn't
    // look back) by the card leaving the zone it came from.
    for (leaving, copies) in [(true, 0), (false, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Aboleth Spawn");
        let cond = if leaving {
            TriggerCond::ZoneChange {
                filter: Filter::OwnedBy(PlayerRel::You),
                from: Some(ZoneKind::Hand),
                to: None,
            }
        } else {
            TriggerCond::EntersBattlefield(Filter::Source)
        };
        let hatchling = CB::new("Test Hatchling")
            .creature(1, 1)
            .cost("{1}")
            .ability(trig(cond, Body::effect(gain(1))))
            .build();
        let card = t.custom(P1, hatchling, Zone::Hand(P1));
        t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P1))
            .expect("moved");
        t.settle();
        assert_eq!(triggers_of(&t, "Test Hatchling"), 1, "leaving: {leaving}");
        assert_eq!(
            triggers_of(&t, "Aboleth Spawn"),
            copies,
            "leaving: {leaving}"
        );
    }
    // Firebender Ascension ("a creature you control attacking causes a triggered ability of
    // that creature to trigger") counts Sun Titan's "enters or attacks" ability when its
    // attack made it trigger, not when its entering did.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firebender Ascension");
    t.graveyard(P0, "Grizzly Bears");
    t.enter(P0, "Sun Titan");
    t.settle();
    assert_eq!(triggers_of(&t, "Sun Titan"), 1);
    assert_eq!(triggers_of(&t, "Firebender Ascension"), 0);
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Firebender Ascension");
    let titan = t.battlefield(P0, "Sun Titan");
    t.graveyard(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(titan, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(triggers_of(&t, "Sun Titan"), 1);
    assert_eq!(triggers_of(&t, "Firebender Ascension"), 1);
    t.resolve();
    assert_eq!(t.counters(asc, "quest"), 1);
}
