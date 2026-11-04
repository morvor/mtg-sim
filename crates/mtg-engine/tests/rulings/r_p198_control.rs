//! Rulings batch P198 — control-changing effects (CR 613.1b) and exchanges of control
//! (CR 701.12): what happens when an exchange can't be completed, summoning sickness of
//! a creature taken this turn, choices made on resolution, and what the rulings say about
//! targets.

use crate::r_p198_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A Chromeshell Crab face down on P0's battlefield, P0's Grizzly Bears and P1's Hill
/// Giant. Turning the Crab face up puts its trigger on the stack targeting the two
/// creatures.
fn crab_setup(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let crab = t.battlefield(P0, "Chromeshell Crab");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, crab));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    assert!(mtg_engine::facedown::turn_face_up_by(&mut t.g, crab, P0));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (crab, bears, giant)
}

#[test]
fn chromeshell_crab_targets_on_trigger_and_chooses_to_exchange_on_resolution() {
    cr!("603.3d", "701.12b");
    ruling!(
        "Chromeshell Crab",
        "You choose the two target creatures when the triggered ability is put onto the stack. You do not have to choose Chromeshell Crab. You choose whether or not to do the exchange on resolution."
    );
    supported("Chromeshell Crab");
    let mut t = TestGame::new(2);
    let (crab, bears, giant) = crab_setup(&mut t);
    let asked = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.asked()[asked..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(ctl(&t, bears), P1);
    assert_eq!(ctl(&t, giant), P0);
    assert_eq!(ctl(&t, crab), P0);
    // Declining on resolution: nothing changes.
    let mut t = TestGame::new(2);
    let (_, bears, giant) = crab_setup(&mut t);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, giant), P1);
}

#[test]
fn chromeshell_crab_exchange_fails_if_a_target_is_gone() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Chromeshell Crab",
        "The exchange fails if either target is not still on the battlefield at that time."
    );
    let mut t = TestGame::new(2);
    let (_, bears, giant) = crab_setup(&mut t);
    kill(&mut t, giant);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn chromeshell_crab_exchange_fails_after_a_control_change() {
    cr!("701.12a");
    ruling!(
        "Chromeshell Crab",
        "The exchange fails if either you no longer control your creature or the other creature is now controlled by you."
    );
    supported("Ray of Command");
    // The opponent takes your creature in response: no exchange (each creature keeps its
    // new controller).
    let mut t = TestGame::new(2);
    let (_, bears, giant) = crab_setup(&mut t);
    cast_at(&mut t, P1, "Ray of Command", &[obj(bears)]);
    t.resolve();
    assert_eq!(ctl(&t, bears), P1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    assert_eq!(ctl(&t, giant), P1);
    // You take the other creature in response: no exchange; you keep your creature.
    let mut t = TestGame::new(2);
    let (_, bears, giant) = crab_setup(&mut t);
    cast_at(&mut t, P0, "Ray of Command", &[obj(giant)]);
    t.resolve();
    assert_eq!(ctl(&t, giant), P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, giant), P0);
}

#[test]
fn role_reversal_needs_a_shared_permanent_type_not_a_supertype() {
    cr!("701.12b", "115.1", "205.4a");
    ruling!(
        "Role Reversal",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker. Supertypes, like legendary, aren’t permanent types. For example, you can’t exchange control of a legendary creature with a legendary planeswalker."
    );
    supported("Role Reversal");
    // A legendary creature and a legendary planeswalker can't be targeted together.
    let mut t = TestGame::new(2);
    let legend = t.battlefield(P0, "Drana, the Last Bloodchief");
    let pw = t.battlefield(P1, "Archangel Elspeth");
    lots(&mut t, P0);
    let c = t.hand(P0, "Role Reversal");
    let r = t.cast(P0, c).targets(&[obj(legend), obj(pw)]).try_go();
    if r.is_ok() {
        t.resolve_all();
    }
    assert_eq!(ctl(&t, legend), P0);
    assert_eq!(ctl(&t, pw), P1);
    // Two creatures share a permanent type: exchanged.
    let mut t = TestGame::new(2);
    let legend = t.battlefield(P0, "Drana, the Last Bloodchief");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let c = t.hand(P0, "Role Reversal");
    t.cast(P0, c).targets(&[obj(legend), obj(bears)]).go();
    t.resolve_all();
    assert_eq!(ctl(&t, legend), P1);
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn simic_manipulator_checks_power_again_on_resolution() {
    cr!("608.2b", "601.2h");
    ruling!(
        "Simic Manipulator",
        "The power of the target creature is checked both as you target it and as the ability resolves. If the power of the target creature when the ability resolves is greater than the number of +1/+1 counters removed from Simic Manipulator, the ability won’t resolve and none of its effects will happen. You won’t gain control of any creature, but the counters removed as a cost remain removed."
    );
    supported("Simic Manipulator");
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        let sm = t.battlefield(P0, "Simic Manipulator");
        t.g.add_counters(obj(sm), "+1/+1", 2, None);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        act_only(&mut t, P0, sm, 0, &[obj(bears)]);
        assert_eq!(t.counters(sm, "+1/+1"), 0);
        if pump {
            cast_at(&mut t, P1, "Giant Growth", &[obj(bears)]);
            t.resolve();
        }
        t.resolve_all();
        assert_eq!(ctl(&t, bears), if pump { P1 } else { P0 }, "pump {pump}");
        assert_eq!(t.counters(sm, "+1/+1"), 0);
    }
}

#[test]
fn daring_thief_relevant_card_types_include_artifact() {
    cr!("701.12b", "205.2a");
    ruling!(
        "Daring Thief",
        "The relevant card types (that is, the ones a nonland permanent can have) are artifact, battle, creature, enchantment, kindred, and planeswalker."
    );
    supported("Daring Thief");
    // An artifact and an artifact creature share a card type; an artifact and a
    // nonartifact creature don't.
    for (theirs_name, exchanged) in [("Ornithopter", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let thief = t.battlefield(P0, "Daring Thief");
        let ring = t.battlefield(P0, "Sol Ring");
        let theirs = t.battlefield(P1, theirs_name);
        t.g.tap(thief);
        t.answer_targets(P0, &[obj(ring)]);
        t.answer_targets(P0, &[obj(theirs)]);
        t.answer_yes(P0, true);
        t.g.untap(thief);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(ctl(&t, ring) == P1, exchanged, "{theirs_name}");
        assert_eq!(ctl(&t, theirs) == P0, exchanged, "{theirs_name}");
    }
}

#[test]
fn dack_faydens_draw_and_discard_happen_in_one_resolution() {
    cr!("608.2c");
    ruling!(
        "Dack Fayden",
        "The targeted player draws two cards and discards two cards all while Dack's first ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Dack Fayden");
    let mut t = TestGame::new(2);
    let dack = t.battlefield(P0, "Dack Fayden");
    t.hand(P1, "Island");
    let drawn = stack_library(&mut t, P1, &["Hill Giant", "Grizzly Bears"]);
    // The two cards just drawn can be discarded by that same resolution.
    t.answer_choose_moved(P1, &[obj(drawn[0]), obj(drawn[1])]);
    let asked = t.asked().len();
    act(&mut t, P0, dack, 0, &[pl(P1)]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Island"));
    assert_eq!(t.hand_size(P1), 1);
    // No player received priority during the resolution.
    assert!(!t.asked()[asked..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn aminatous_draw_and_put_back_happen_in_one_resolution() {
    cr!("608.2c");
    ruling!(
        "Aminatou, the Fateshifter",
        "You draw a card and put one back all while Aminatou's first ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Aminatou, the Fateshifter");
    let mut t = TestGame::new(2);
    let am = t.battlefield(P0, "Aminatou, the Fateshifter");
    t.hand(P0, "Island");
    let drawn = stack_library(&mut t, P0, &["Hill Giant"]);
    t.answer_choose_moved(P0, &[obj(drawn[0])]);
    let asked = t.asked().len();
    act(&mut t, P0, am, 0, &[]);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Island"));
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Hill Giant");
    assert!(!t.asked()[asked..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn a_creature_taken_this_turn_is_summoning_sick_unless_it_has_haste() {
    cr!("302.6", "702.10c");
    ruling!(
        "Tempted by the Oriq",
        "Unless it has haste, you will not able to attack with a creature or activate its abilities that have {T} in the cost on the turn you gain control of it."
    );
    ruling!(
        "Invoke the Winds",
        "You will not be able to attack with the creature you gain control of or use its activated abilities with {T} in the cost unless it has haste."
    );
    supported("Tempted by the Oriq");
    supported("Invoke the Winds");
    for spell in ["Tempted by the Oriq", "Invoke the Winds"] {
        let mut t = TestGame::new(2);
        let sorc = t.battlefield(P1, "Prodigal Sorcerer");
        cast_at(&mut t, P0, spell, &[obj(sorc)]);
        t.resolve_all();
        assert_eq!(ctl(&t, sorc), P0, "{spell}");
        assert!(t.activate(P0, sorc, 0, &[pl(P1)]).is_err(), "{spell}");
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!t.g.can_attack(sorc), "{spell}");
        // A creature with haste can attack.
        let mut t = TestGame::new(2);
        let gob = t.battlefield(P1, "Raging Goblin");
        cast_at(&mut t, P0, spell, &[obj(gob)]);
        t.resolve_all();
        assert_eq!(ctl(&t, gob), P0, "{spell}");
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(t.g.can_attack(gob), "{spell}");
    }
}

#[test]
fn jon_irenicus_creature_cant_be_sacrificed_by_its_new_controller() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Jon Irenicus, Shattered One",
        "When Jon Irenicus's first ability resolves, the opponent will gain control of a creature and that creature will immediately gain “This creature can't be sacrificed” as part of the resolution of the ability."
    );
    supported("Jon Irenicus, Shattered One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jon Irenicus, Shattered One");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert!(t.obj_now(bears).tapped);
    assert!(has_ability(&t, bears, "can't be sacrificed"));
    // Its new controller is told to sacrifice a creature: it can't be.
    pass_turn(&mut t);
    cast_at(&mut t, P0, "Diabolic Edict", &[pl(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn karrthus_untaps_all_dragons_including_your_own() {
    cr!("608.2c");
    ruling!(
        "Karrthus, Tyrant of Jund",
        "When the triggered ability resolves, you'll untap all Dragons, including the ones you already control."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Shivan Dragon");
    let theirs = t.battlefield(P1, "Shivan Dragon");
    t.g.tap(mine);
    t.g.tap(theirs);
    let k = t.enter(P0, "Karrthus, Tyrant of Jund");
    t.resolve_all();
    assert_eq!(ctl(&t, theirs), P0);
    assert!(!t.obj_now(mine).tapped);
    assert!(!t.obj_now(theirs).tapped);
    // Other Dragons have haste.
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Haste));
    assert!(t.on_battlefield(k));
}

#[test]
fn aura_graft_can_move_an_aura_you_already_control() {
    cr!("701.3a", "303.4");
    ruling!(
        "Aura Graft",
        "You can target an Aura you already control just to move that Aura to a new permanent."
    );
    supported("Aura Graft");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let hs = cast_at(&mut t, P0, "Holy Strength", &[obj(a)]);
    t.resolve_all();
    let hs = t.g.current(hs);
    assert_eq!(t.pt(a), (3, 4));
    t.answer_choose(P0, &[obj(b)]);
    cast_at(&mut t, P0, "Aura Graft", &[obj(hs)]);
    t.resolve_all();
    assert_eq!(t.obj_now(hs).attached_to, Some(obj(b)));
    assert_eq!(ctl(&t, hs), P0);
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (4, 5));
}

#[test]
fn confiscation_coup_pays_energy_on_resolution_and_gets_none_if_it_fizzles() {
    cr!("608.2b");
    ruling!(
        "Confiscation Coup",
        "You choose the target artifact or creature as you cast Confiscation Coup. You don't choose whether or not to pay an amount of {E} until Confiscation Coup resolves. If the target becomes an illegal target, Confiscation Coup won't resolve and you won't get {E}{E}{E}{E}."
    );
    supported("Confiscation Coup");
    let energy = |t: &TestGame| t.g.player(P0).counters.get("energy").copied().unwrap_or(0);
    // Resolves: four energy, then pay 4 for Hill Giant (mana value 4).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Confiscation Coup", &[obj(giant)]);
    let asked = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.asked()[asked..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(ctl(&t, giant), P0);
    assert_eq!(energy(&t), 0);
    // The target leaves: no energy.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Confiscation Coup", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert_eq!(energy(&t), 0);
    assert!(t.in_graveyard(P0, "Confiscation Coup"));
}

#[test]
fn switcheroo_exchanges_two_creatures_you_dont_control() {
    cr!("701.12b", "115.1");
    ruling!("Switcheroo", "You don’t have to control either target.");
    supported("Switcheroo");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    cast_multi(&mut t, P0, "Switcheroo", &[obj(a), obj(b)]);
    t.resolve_all();
    assert_eq!(ctl(&t, a), P2);
    assert_eq!(ctl(&t, b), P1);
}
