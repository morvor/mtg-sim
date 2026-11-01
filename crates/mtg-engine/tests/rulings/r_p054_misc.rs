//! Rulings batch P054 — the rest: Haunted Plate Mail animated in response to another,
//! indestructible and what still puts it into the graveyard (CR 702.12b), mana
//! abilities on creatures (CR 605.1a-b), Equipment and Auras that pump (durations,
//! counters left behind, control of an Aura, CR 303.4e), characteristic-defining
//! abilities in every zone (CR 604.3), copies of a spell keeping their targets
//! (CR 707.10c), modal triggers with no legal target for a mode (CR 700.2a), "with
//! islandwalk" and snow landwalk (CR 702.14c), shroud and Auras, and exhaust-style
//! "activate only once" on a new object (CR 400.7).

use crate::r_p054_common::*;
use crate::r_s01_common::custom_card;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s06_common::{attach_new, damage};
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn haunted_plate_mail_can_be_animated_in_response_to_another() {
    cr!("602.5b", "117.3c");
    ruling!(
        "Haunted Plate Mail",
        "It's possible to activate the first activated ability of Haunted Plate Mail in response to activating the first ability of another Haunted Plate Mail and have them both become creatures until end of turn."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Haunted Plate Mail");
    let b = t.battlefield(P0, "Haunted Plate Mail");
    t.activate(P0, a, 0, &[]).expect("first");
    t.activate(P0, b, 0, &[]).expect("in response");
    t.resolve_all();
    for id in [a, b] {
        assert!(t.obj_now(id).is(CardType::Creature));
        assert_eq!(t.pt(id), (4, 4));
    }
    // Now P0 controls creatures: it can't be activated again.
    let c = t.battlefield(P0, "Haunted Plate Mail");
    assert!(t.activate(P0, c, 0, &[]).is_err());
}

#[test]
fn indestructible_permanents_still_go_to_the_graveyard_for_other_reasons() {
    cr!("702.12b", "704.5f", "704.5m", "701.21a");
    ruling!(
        "Elspeth, Knight-Errant",
        "Lethal damage and effects that say “destroy” won’t cause a permanent with indestructible to be put into the graveyard. However, a permanent with indestructible can be put into the graveyard for a number of reasons. The most likely reasons are if it’s sacrificed, if it’s a creature whose toughness is 0 or less, or if it’s an Aura that’s either unattached or attached to something illegal."
    );
    let mut t = TestGame::new(2);
    let el = t.battlefield(P0, "Elspeth, Knight-Errant");
    t.g.add_counters(o(el), "loyalty", 10, None);
    t.activate(P0, el, 2, &[]).expect("-8");
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let host = t.battlefield(P0, "Hill Giant");
    let aura = attach_new(&mut t, P0, "Holy Strength", host);
    let src = t.battlefield(P1, "Craw Wurm");
    destroy(&mut t, a);
    damage(&mut t, src, 6, a);
    assert!(t.on_battlefield(a), "destroy and lethal damage");
    t.g.sacrifice(b, P0);
    t.settle();
    assert!(!t.on_battlefield(b), "sacrificed");
    crate::r_p017_common::modify_no_settle(
        &mut t,
        c,
        vec![Modification::ModifyPT(Value::c(0), Value::c(-2))],
    );
    t.settle();
    assert!(!t.on_battlefield(c), "toughness 0");
    move_to(&mut t, host, Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Holy Strength"), "unattached Aura");
    let _ = aura;
}

#[test]
fn raggadragga_counts_activated_and_triggered_mana_abilities() {
    cr!("605.1a", "605.1b");
    ruling!(
        "Raggadragga, Goreguts Boss",
        "Most mana abilities found on creatures are activated mana abilities. An activated mana ability is an activated ability that could produce mana as it resolves, does not require a target, and isn't a loyalty ability."
    );
    ruling!(
        "Raggadragga, Goreguts Boss",
        "Some creatures may have triggered mana abilities. A triggered mana ability is a triggered ability that could produce mana as it resolves, triggers on the resolution of an activated mana ability or on mana being added, and doesn't require a target."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raggadragga, Goreguts Boss");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let vorinclex = t.battlefield(P0, "Vorinclex, Voice of Hunger");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.g.recompute();
    assert_eq!(t.pt(elves), (3, 3), "activated mana ability");
    assert_eq!(t.pt(vorinclex), (9, 8), "triggered mana ability");
    assert_eq!(
        t.pt(pyro),
        (1, 1),
        "an activated ability that isn't a mana ability"
    );
}

/// Equipment and Auras whose rulings restate their text.
#[test]
fn equipment_and_aura_pumps_as_written() {
    cr!("301.5", "303.4", "611.2a", "514.2", "122.2");
    ruling!(
        "Starforged Sword",
        "Starforged Sword can be attached to a creature that didn't have flying to begin with."
    );
    ruling!("Fit of Rage", "The +3/+3 is also only until end of turn.");
    ruling!(
        "Unstable Mutation",
        "The -1/-1 counters stay even if the Aura is removed, and the +3/+3 goes away when the Aura does."
    );
    ruling!(
        "Helm of the Gods",
        "If you cast an Aura spell targeting a permanent controlled by an opponent, you still control that Aura. It will count toward the bonus given by Helm of the Gods."
    );
    // Starforged Sword on a creature without flying.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Starforged Sword");
    pool(&mut t, P0);
    t.activate(P0, sword, 0, &[o(bear)]).expect("equip");
    t.resolve_all();
    assert_eq!(t.obj_now(sword).attached_to, Some(o(bear)));
    assert_eq!(t.pt(bear), (5, 5));
    // Fit of Rage ends at the cleanup step.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Fit of Rage", bear);
    t.resolve_all();
    assert_eq!(t.pt(bear), (5, 5));
    assert!(t.obj_now(bear).has_keyword(KeywordKind::FirstStrike));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bear), (2, 2));
    assert!(!t.obj_now(bear).has_keyword(KeywordKind::FirstStrike));
    // Unstable Mutation: the counter stays, the bonus goes.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let um = attach_new(&mut t, P0, "Unstable Mutation", giant);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    assert_eq!(t.pt(giant), (5, 5));
    move_to(&mut t, um, Zone::Graveyard(P0));
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    assert_eq!(t.pt(giant), (2, 2));
    // Helm of the Gods counts P0's Aura on P1's creature.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Helm of the Gods", bear);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Pacifism", theirs);
    t.resolve_all();
    let pac = t.named_on_battlefield("Pacifism")[0];
    assert_eq!(t.obj_now(pac).controller, P0);
    assert_eq!(t.pt(bear), (3, 3));
}

#[test]
fn rubblehulks_power_and_toughness_apply_in_every_zone() {
    cr!("604.3", "208.2a");
    ruling!(
        "Rubblehulk",
        "The ability that defines Rubblehulk's power and toughness works in all zones, not just the battlefield."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let hand = t.hand(P0, "Rubblehulk");
    let gy = t.graveyard(P0, "Rubblehulk");
    t.g.recompute();
    assert_eq!(t.pt(hand), (3, 3));
    assert_eq!(t.pt(gy), (3, 3));
    t.lands(P0, "Mountain", 1);
    t.g.recompute();
    assert_eq!(t.pt(hand), (4, 4));
}

#[test]
fn lumarets_favor_copy_keeps_the_target_unless_changed() {
    cr!("707.10c", "707.10");
    ruling!(
        "Lumaret's Favor",
        "The copy will have the same target as the original spell unless you choose a new one. If the target is illegal but you can't choose a new legal target, it remains unchanged."
    );
    // The same target: +4/+8 in total.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.answer_yes(P0, false);
    cast_at(&mut t, P0, "Lumaret's Favor", bear);
    t.resolve_all();
    assert_eq!(t.pt(bear), (6, 10));
    // The target became illegal and there's no other: the copy keeps it and fizzles.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    cast_at(&mut t, P0, "Lumaret's Favor", bear);
    move_to(&mut t, bear, Zone::Exile);
    t.answer_yes(P0, true);
    t.resolve(); // the infusion trigger: the copy
    let copies = crate::r_s11_common::spells_copied(&t);
    assert_eq!(copies, 1);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(
        crate::r_p006_common::targets_of(&t, copy),
        vec![o(bear)],
        "unchanged"
    );
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_modal_trigger_cant_choose_a_mode_without_legal_targets() {
    cr!("700.2a", "603.3c");
    ruling!(
        "Hooded Assassin",
        "If a mode requires a target and there are no legal targets available, that mode can't be chosen. You must choose another mode if able."
    );
    ruling!(
        "Defiant Ogre",
        "If a mode requires a target and there are no legal targets available, that mode can't be chosen. You must choose another mode if able."
    );
    for name in ["Hooded Assassin", "Defiant Ogre"] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        let id = enter(&mut t, P0, name);
        t.resolve_all();
        assert_eq!(t.counters(id, "+1/+1"), 1, "{name}: the counter mode");
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    }
}

#[test]
fn merfolk_assassin_can_target_a_creature_with_snow_islandwalk() {
    cr!("702.14a", "702.14c", "115.1");
    interp!("qualified-landwalk-is-that-landwalk");
    ruling!(
        "Merfolk Assassin",
        "Can target a creature with snow islandwalk."
    );
    let mut t = TestGame::new(2);
    let ma = t.battlefield(P0, "Merfolk Assassin");
    let snowy = t.custom(
        P1,
        custom_card(
            "Snowy Merfolk",
            "Creature — Merfolk",
            "{U}",
            Some((1, 1)),
            "Snow islandwalk",
        ),
        Zone::Battlefield,
    );
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Snow swampwalk isn't islandwalk; plain islandwalk is.
    let legions = t.battlefield(P1, "Legions of Lim-Dûl");
    let plain = t.custom(
        P1,
        custom_card(
            "Walker",
            "Creature — Fish",
            "{U}",
            Some((1, 1)),
            "Islandwalk",
        ),
        Zone::Battlefield,
    );
    let mark = t.asked().len();
    t.activate(P0, ma, 0, &[o(snowy)]).expect("target it");
    let cands = target_candidates(&t, P0, mark);
    assert!(cands[0].contains(&o(snowy)) && cands[0].contains(&o(plain)));
    assert!(!cands[0].contains(&o(bear)) && !cands[0].contains(&o(legions)));
    t.resolve_all();
    assert!(!t.on_battlefield(snowy));
}

#[test]
fn homarid_warriors_shroud_keeps_its_auras_and_it_taps_on_resolution() {
    cr!("702.18a", "303.4c", "602.2");
    ruling!(
        "Homarid Warrior",
        "Does not cause Auras on it to be removed when the effect is activated. An Aura on the battlefield is neither a spell nor an ability."
    );
    ruling!(
        "Homarid Warrior",
        "It taps when the ability resolves if it was not already tapped. It may be used even if it is already tapped."
    );
    let mut t = TestGame::new(2);
    let hw = t.battlefield(P0, "Homarid Warrior");
    attach_new(&mut t, P1, "Pacifism", hw);
    pool(&mut t, P0);
    t.activate(P0, hw, 0, &[]).expect("activate");
    assert!(!t.obj_now(hw).tapped, "not a cost");
    t.resolve_all();
    assert!(t.obj_now(hw).tapped);
    assert!(t.obj_now(hw).has_keyword(KeywordKind::Shroud));
    assert!(
        !t.named_on_battlefield("Pacifism").is_empty(),
        "the Aura stays"
    );
    // Already tapped: it can still be activated.
    assert!(t.activate(P0, hw, 0, &[]).is_ok());
    t.resolve_all();
}

#[test]
fn emissary_of_grudges_is_a_new_object_after_leaving() {
    cr!("400.7", "602.5b");
    ruling!(
        "Emissary of Grudges",
        "If Emissary of Grudges leaves the battlefield and returns, it’s considered a new object. You may choose a new opponent and you may activate its last ability again."
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[pl(P1)]);
    let em = enter(&mut t, P0, "Emissary of Grudges");
    let bolt = |t: &mut TestGame, p: PlayerId| {
        pool(t, p);
        let b = t.hand(p, "Lightning Bolt");
        t.cast(p, b).target(pl(P0)).go()
    };
    let s = bolt(&mut t, P1);
    t.activate(P0, em, 0, &[o(s)]).expect("first activation");
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let s = bolt(&mut t, P1);
    assert!(t.activate(P0, em, 0, &[o(s)]).is_err(), "only once");
    t.clear_answers();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // It leaves and returns: P2 chosen this time, and it can be activated again.
    move_to(&mut t, em, Zone::Exile);
    t.answer_choose(P0, &[pl(P2)]);
    move_to(&mut t, em, Zone::Battlefield);
    let em2 = t.g.current(em);
    t.set_step(P0, Step::PrecombatMain);
    let s = bolt(&mut t, P2);
    t.activate(P0, em2, 0, &[o(s)]).expect("a new object");
    t.answer_targets(P0, &[pl(P2)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P2), 17);
}

#[test]
fn aurochs_is_still_its_printed_size_when_attack_legality_is_checked() {
    cr!("508.1c", "508.1m", "508.2");
    ruling!(
        "Aurochs",
        "The bonus is not applied until after you check for attack legality, so it is a 2/3 creature when declaring attackers."
    );
    crate::r_s01_common::supported("Aurochs");
    // Ensnaring Bridge with two cards in hand: power 2 may attack, power 3 may not.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ensnaring Bridge");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let a = t.battlefield(P0, "Aurochs");
    let b = t.battlefield(P0, "Aurochs");
    crate::r_s01_common::attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    t.resolve_all();
    let attacking = |t: &TestGame, id: ObjectId| {
        t.g.combat
            .as_ref()
            .is_some_and(|c| c.attack_target(t.g.current(id)).is_some())
    };
    assert!(attacking(&t, a) && attacking(&t, b));
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
}

#[test]
fn commanders_plate_uses_both_commanders_combined_color_identity() {
    cr!("903.4", "702.16a");
    ruling!(
        "Commander's Plate",
        "If you have two commanders and attach Commander's Plate to one, it has protection from each color not in your two commanders' combined color identity."
    );
    crate::r_s01_common::supported("Commander's Plate");
    let mut t = crate::r_s13_common::commander_game();
    // Thrasios (green-blue) and Tymna (white-black): protection from red only.
    crate::r_s13_common::commander(&mut t, P0, "Thrasios, Triton Hero");
    let tymna = crate::r_s13_common::commander(&mut t, P0, "Tymna the Weaver");
    move_to(&mut t, tymna, Zone::Battlefield);
    let tymna = t.g.current(tymna);
    attach_new(&mut t, P0, "Commander's Plate", tymna);
    assert_eq!(t.pt(tymna), (5, 5));
    let can_target = |t: &mut TestGame, p: PlayerId, name: &str| -> bool {
        pool(t, p);
        let c = t.hand(p, name);
        let mark = t.asked().len();
        t.answer_targets(p, &[o(tymna)]);
        let ok = t.cast(p, c).try_go().is_ok();
        let offered = target_candidates(t, p, mark)
            .iter()
            .any(|c| c.contains(&o(tymna)));
        t.clear_answers();
        t.resolve_all();
        ok && offered
    };
    assert!(!can_target(&mut t, P1, "Lightning Bolt"), "red");
    assert!(can_target(&mut t, P1, "Giant Growth"), "green");
    assert!(can_target(&mut t, P1, "Disfigure"), "black");
}

/// The same reading for "creatures with islandwalk can be blocked as though they didn't
/// have islandwalk" (Undertow): it covers snow islandwalk.
#[test]
fn undertow_lets_a_snow_islandwalker_be_blocked() {
    cr!("702.14c", "509.1b");
    interp!("qualified-landwalk-is-that-landwalk");
    for undertow in [false, true] {
        let mut t = TestGame::new(2);
        let snowy = t.custom(
            P0,
            custom_card(
                "Snowy Merfolk",
                "Creature — Merfolk",
                "{U}",
                Some((1, 1)),
                "Snow islandwalk",
            ),
            Zone::Battlefield,
        );
        t.battlefield(P1, "Snow-Covered Island");
        let wall = t.battlefield(P1, "Wall of Wood");
        if undertow {
            t.battlefield(P1, "Undertow");
        }
        crate::r_s01_common::attack_with(&mut t, &[(snowy, pl(P1))]);
        assert_eq!(
            crate::r_s21_common::legal_blocks(&mut t, P1, &[(wall, snowy)]),
            undertow,
            "Undertow: {undertow}"
        );
    }
}
