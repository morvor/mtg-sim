//! Rulings batch P024 — copy effects interacting with other rules: type-changing effects
//! and "if it's a creature" exceptions (CR 707.2, 707.9f), copy effects ending at cleanup
//! (CR 514.2), face-down permanents (CR 708.2), characteristic-defining abilities (CR
//! 604.3, 707.9d), replacement effects and indestructible (CR 614, 702.12b), summoning
//! sickness (CR 302.6, 702.10), and Equipment that stops or starts being an Equipment (CR
//! 301.5c, 704.5p).

use crate::r_p024_common::*;
use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Spark Double ---------------------------------------------------------------------------

#[test]
fn spark_double_uses_its_own_characteristics_as_it_enters_for_its_counters() {
    cr!("707.2", "707.9f", "614.1c", "306.5b");
    ruling!("Spark Double", "If the copied permanent is affected by a type-changing effect, Spark Double may enter the battlefield with different permanent types than the copied permanent currently has. Use the characteristics of Spark Double as it enters the battlefield, not of the copied permanent, to determine whether it enters with an additional counter on it. Notably, if Spark Double copies a Gideon planeswalker that’s a creature because its loyalty ability caused it to become a planeswalker creature, Spark Double enters as a noncreature planeswalker and doesn’t get a +1/+1 counter. On the other hand, if Spark Double copies Gideon Blackblade during your turn, Spark Double enters as a planeswalker creature and gets both kinds of counters.");
    supported("Spark Double");
    supported("Gideon, Ally of Zendikar");
    supported("Gideon Blackblade");
    // Gideon, Ally of Zendikar's "+1: Until end of turn, Gideon becomes a 5/5 ... creature":
    // a type-changing effect, not copied. The Double is a noncreature planeswalker with
    // four loyalty and one more, and no +1/+1 counter.
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon, Ally of Zendikar");
    activate_containing(&mut t, P0, gideon, "becomes a 5/5").expect("+1");
    t.resolve_all();
    assert!(is_creature(&t, gideon));
    let double = enter_copying(&mut t, P0, "Spark Double", gideon);
    assert_eq!(name_of(&t, double), "Gideon, Ally of Zendikar");
    assert!(!is_creature(&t, double));
    assert!(t.obj_now(double).is(CardType::Planeswalker));
    assert_eq!(t.counters(double, counters::PLUS1), 0);
    assert_eq!(t.counters(double, counters::LOYALTY), 5);
    // Gideon Blackblade is a creature during its controller's turn by its own text: the
    // Double copies that and gets both counters.
    let mut t = TestGame::new(2);
    let blackblade = t.battlefield(P0, "Gideon Blackblade");
    assert!(is_creature(&t, blackblade));
    let double = enter_copying(&mut t, P0, "Spark Double", blackblade);
    assert!(is_creature(&t, double) && t.obj_now(double).is(CardType::Planeswalker));
    assert_eq!(t.counters(double, counters::PLUS1), 1);
    assert_eq!(t.counters(double, counters::LOYALTY), 5);
    // Not legendary: both stay.
    assert_eq!(t.named_on_battlefield("Gideon Blackblade").len(), 2);
}

// --- Renegade Doppelganger --------------------------------------------------------------------

#[test]
fn renegade_doppelganger_copys_end_step_trigger_resolves_before_the_copy_ends() {
    cr!("514.2", "513.1", "603.2b", "611.2a", "707.2");
    ruling!("Renegade Doppelganger", "If the creature Renegade Doppelganger copies has an ability that triggers at the beginning of the end step, that ability will trigger and resolve before Renegade Doppelganger's copy effect wears off.");
    supported("Renegade Doppelganger");
    supported("Alert Heedbonder");
    // Alert Heedbonder: vigilance and "At the beginning of your end step, you gain 1 life
    // for each creature you control with vigilance."
    let mut t = TestGame::new(2);
    let dopp = t.battlefield(P0, "Renegade Doppelganger");
    t.answer_yes(P0, true);
    t.enter(P0, "Alert Heedbonder");
    t.resolve_all();
    assert_eq!(name_of(&t, dopp), "Alert Heedbonder");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Both triggered, each counting two creatures with vigilance.
    assert_eq!(t.life(P0), 24, "{}", t.dump_log());
    assert_eq!(name_of(&t, dopp), "Alert Heedbonder");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, dopp), "Renegade Doppelganger");
}

// --- Face-down creatures ------------------------------------------------------------------------

/// Asserts `id` is a face-up colorless 2/2 creature with no name, types or abilities.
fn blank_2_2(t: &TestGame, id: ObjectId) {
    let o = t.obj_now(id);
    assert!(!o.face_down);
    assert!(o.chars.name.is_empty(), "{:?}", o.chars.name);
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.subtypes.is_empty() && o.chars.abilities.is_empty());
    assert!(o
        .chars
        .mana_cost
        .as_ref()
        .is_none_or(|m| m.mana_value() == 0));
    assert_eq!(t.pt(id), (2, 2));
}

#[test]
fn becoming_a_copy_of_a_face_down_creature_makes_a_blank_2_2() {
    cr!("707.2", "708.2", "116.2b");
    ruling!("Cytoshape", "If the creature becomes a copy of a face-down creature, it will become a 2/2 creature with no name, creature type, abilities, mana cost, or color. It will not become face-down and thus can’t be turned face up.");
    ruling!("Shapesharer", "If the targeted Shapeshifter becomes a copy of a face-down creature, it will become a 2/2 creature with no name, creature type, abilities, mana cost, or color. It will not become face down and thus can’t be turned face up.");
    supported("Shapesharer");
    // Cytoshape.
    let mut t = TestGame::new(2);
    let fd = crate::r_s12_common::morph(&mut t, P0, "Den Protector");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(fd)]);
    cast_spell(
        &mut t,
        P0,
        "Cytoshape",
        "{1}{G}{U}",
        &[obj(bears)],
        None,
        None,
    );
    blank_2_2(&t, bears);
    cant_be_turned_face_up(&mut t, bears, fd);
    // Shapesharer (a changeling Shapeshifter, so it loses its creature types too).
    let mut t = TestGame::new(2);
    let fd = crate::r_s12_common::morph(&mut t, P0, "Den Protector");
    let sharer = t.battlefield(P0, "Shapesharer");
    lands_for(&mut t, P0, "{2}{U}");
    t.activate(P0, sharer, 0, &[obj(sharer), obj(fd)])
        .expect("activation");
    t.resolve_all();
    blank_2_2(&t, sharer);
    // The face-down creature itself is still face down.
    assert!(t.obj_now(fd).face_down);
    cant_be_turned_face_up(&mut t, sharer, fd);
}

/// With mana for Den Protector's megamorph cost, P0 may turn the face-down `fd` face up
/// (CR 702.37e, 116.2b) but not `copy`, a face-up copy of it.
fn cant_be_turned_face_up(t: &mut TestGame, copy: ObjectId, fd: ObjectId) {
    use mtg_engine::decision::{Action, SpecialAction};
    t.lands(P0, "Forest", 2);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let face_up: Vec<ObjectId> =
        t.g.legal_actions(P0)
            .into_iter()
            .filter_map(|a| match a {
                Action::Special(SpecialAction::TurnFaceUp { obj }) => Some(obj),
                _ => None,
            })
            .collect();
    assert!(!face_up.contains(&t.g.current(copy)));
    assert_eq!(face_up, vec![t.g.current(fd)]);
}

// --- Characteristic-defining abilities ------------------------------------------------------

#[test]
fn unstable_shapeshifter_copying_tarmogoyf_is_recalculated() {
    cr!("604.3", "707.2", "613.4a");
    ruling!("Unstable Shapeshifter", "If the creature has power/toughness that is continuously recalculated, so does this copy of it.");
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.graveyard(P1, "Grizzly Bears");
    t.enter(P1, "Tarmogoyf");
    t.resolve_all();
    assert_eq!(name_of(&t, shifter), "Tarmogoyf");
    // A creature card in a graveyard: 1/2.
    assert_eq!(t.pt(shifter), (1, 2));
    t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(shifter), (2, 3));
}

#[test]
fn dollhouse_of_horrors_doesnt_copy_a_power_toughness_cda() {
    cr!("707.9d", "604.3");
    ruling!("Dollhouse of Horrors", "If the exiled creature card has any characteristic-defining abilities that define its power and/or toughness, those abilities are not copied.");
    supported("Dollhouse of Horrors");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let house = t.battlefield(P0, "Dollhouse of Horrors");
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(goyf)]);
    let before = t.g.battlefield.clone();
    t.activate(P0, house, 0, &[]).expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    // A 0/0 with +1/+1 for the one Construct: 1/1, not Tarmogoyf's */1+*.
    assert_eq!(name_of(&t, toks[0]), "Tarmogoyf");
    assert_eq!(t.pt(toks[0]), (1, 1));
}

#[test]
fn saw_in_half_doesnt_copy_a_power_toughness_cda() {
    cr!("707.9d", "604.3", "608.2h");
    ruling!("Saw in Half", "If the original creature had a characteristic-defining ability that determined its power and/or toughness, that ability is not copied.");
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P1, "Tarmogoyf");
    // Creature and land cards in graveyards: Tarmogoyf is 2/3.
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    let before = t.g.battlefield.clone();
    cast_spell(
        &mut t,
        P0,
        "Saw in Half",
        "{2}{B}",
        &[obj(goyf)],
        None,
        None,
    );
    let toks = new_tokens(&t, P1, &before);
    assert_eq!(toks.len(), 2);
    // Half its 2/3 rounded up (an instant, Saw in Half, is in a graveyard now too): 1/2,
    // and they stay so as more card types reach graveyards.
    t.graveyard(P1, "Bonesplitter");
    t.g.recompute();
    for c in toks {
        assert_eq!(name_of(&t, c), "Tarmogoyf");
        assert_eq!(t.pt(c), (1, 2));
    }
}

// --- Saw in Half: the creature must die -----------------------------------------------------

#[test]
fn saw_in_half_makes_no_tokens_unless_the_creature_dies() {
    cr!("614.6", "702.12b", "700.4");
    ruling!("Saw in Half", "If the creature isn't destroyed (perhaps because it has indestructible), its controller doesn't create any tokens. The same is true if the creature is destroyed but instead of going to its owner's graveyard, a replacement effect moves it elsewhere, such as exile.");
    supported("Rest in Peace");
    // Indestructible.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let before = t.g.battlefield.clone();
    cast_spell(&mut t, P0, "Saw in Half", "{2}{B}", &[obj(myr)], None, None);
    assert!(t.on_battlefield(myr));
    assert!(new_tokens(&t, P1, &before).is_empty());
    // Destroyed, but exiled instead of going to the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    let giant = t.battlefield(P1, "Hill Giant");
    let before = t.g.battlefield.clone();
    cast_spell(
        &mut t,
        P0,
        "Saw in Half",
        "{2}{B}",
        &[obj(giant)],
        None,
        None,
    );
    assert!(t.in_exile("Hill Giant"));
    assert!(new_tokens(&t, P1, &before).is_empty());
    // Positive control: it dies.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let before = t.g.battlefield.clone();
    cast_spell(
        &mut t,
        P0,
        "Saw in Half",
        "{2}{B}",
        &[obj(giant)],
        None,
        None,
    );
    assert_eq!(new_tokens(&t, P1, &before).len(), 2);
}

// --- Mardu Siegebreaker ---------------------------------------------------------------------

#[test]
fn mardu_siegebreaker_copies_of_a_noncreature_card_enter_tapped_but_not_attacking() {
    cr!("506.3a", "707.2");
    ruling!("Mardu Siegebreaker", "If the exiled card isn’t a creature card, the token copies will still be created and they’ll still enter tapped. They just won’t be attacking.");
    supported("Mardu Siegebreaker");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    // Mutavault, a 2/2 creature until end of turn, is exiled by the Siegebreaker.
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, vault, "becomes a 2/2").expect("animate");
    t.resolve_all();
    t.answer_targets(P0, &[obj(vault)]);
    let siege = t.enter(P0, "Mardu Siegebreaker");
    t.resolve_all();
    assert!(t.in_exile("Mutavault"), "{}", t.dump_log());
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(siege, Entity::Player(P1))]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    let tok = toks[0];
    assert_eq!(name_of(&t, tok), "Mutavault");
    assert!(t.obj_now(tok).tapped);
    let attacking =
        t.g.combat
            .as_ref()
            .unwrap()
            .attackers
            .iter()
            .any(|a| a.id == tok);
    assert!(!attacking);
}

// --- Summoning sickness -----------------------------------------------------------------------

/// P0's Millstone (entered this turn when `sick`; no mana ability, so it isn't tapped to
/// pay for True Polymorph) becomes a copy of P0's Prodigal Sorcerer ("{T}: This creature
/// deals 1 damage to any target.") by Saheeli, Sublime Artificer's −2 or by True
/// Polymorph, with Fervor ("Creatures you control have haste.") out when `haste`.
fn sorcerer_stone(sick: bool, saheeli: bool, haste: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    if haste {
        t.battlefield(P0, "Fervor");
    }
    let stone = if sick {
        t.battlefield_sick(P0, "Millstone")
    } else {
        t.battlefield(P0, "Millstone")
    };
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    if saheeli {
        let s = t.battlefield(P0, "Saheeli, Sublime Artificer");
        t.answer_targets(P0, &[obj(stone)]);
        t.answer_targets(P0, &[obj(sorcerer)]);
        activate_containing(&mut t, P0, s, "becomes a copy").expect("−2");
        t.resolve_all();
    } else {
        cast_spell(
            &mut t,
            P0,
            "True Polymorph",
            "{4}{U}{U}",
            &[obj(stone), obj(sorcerer)],
            None,
            None,
        );
    }
    assert_eq!(name_of(&t, stone), "Prodigal Sorcerer");
    assert!(is_creature(&t, stone) && is_artifact(&t, stone) == saheeli);
    (t, stone)
}

#[test]
fn an_artifact_that_entered_this_turn_and_becomes_a_creature_copy_is_summoning_sick() {
    cr!("302.6", "707.2", "702.10b", "702.10c");
    ruling!("True Polymorph", "If the first target becomes a creature the same turn it entered the battlefield, you can't attack with it or use any of its {T} abilities unless it has haste.");
    ruling!("Saheeli, Sublime Artificer", "If the target artifact becomes a creature the same turn it enters the battlefield, you can't attack with it or use any of its {T} abilities unless it has haste.");
    supported("True Polymorph");
    supported("Saheeli, Sublime Artificer");
    supported("Fervor");
    // (entered this turn, Saheeli rather than True Polymorph, haste from Fervor)
    for (sick, saheeli, haste) in [
        (true, false, false),
        (false, false, false),
        (true, true, false),
        (false, true, false),
        (true, false, true),
        (true, true, true),
    ] {
        let case = format!("sick: {sick}, Saheeli: {saheeli}, haste: {haste}");
        let can = !sick || haste;
        // Its {T} ability: 1 damage to P1.
        let (mut t, stone) = sorcerer_stone(sick, saheeli, haste);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let r = activate_containing(&mut t, P0, stone, "damage to any target");
        assert_eq!(r.is_ok(), can, "{case}: {r:?}");
        t.resolve_all();
        assert_eq!(t.life(P1), if can { 19 } else { 20 }, "{case}");
        // Declared as an attacker: 1 combat damage (or the declaration is illegal and
        // nothing attacks).
        let (mut t, stone) = sorcerer_stone(sick, saheeli, haste);
        crate::r_s01_common::attack_with(&mut t, &[(stone, Entity::Player(P1))]);
        crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), if can { 19 } else { 20 }, "{case}");
    }
}

// --- Equipment that stops or starts being an Equipment ----------------------------------------

#[test]
fn an_equipment_that_becomes_a_copy_of_a_non_equipment_becomes_unattached() {
    cr!("301.5c", "704.5p", "707.2");
    ruling!("Saheeli, Sublime Artificer", "If the target artifact is an Equipment attached to a creature and it becomes a copy of a non-Equipment permanent, it becomes unattached. If it becomes a copy of an Equipment, it remains attached.");
    ruling!("True Polymorph", "If the target permanent is an Equipment attached to a creature and it becomes a copy of a non-Equipment permanent, it becomes unattached. If it becomes a copy of an Equipment, it remains attached.");
    for (saheeli, into_equipment) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let splitter = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
        assert_eq!(t.pt(bears), (4, 2));
        let into = if into_equipment {
            t.battlefield(P0, "Leonin Scimitar")
        } else {
            t.battlefield(P0, "Ornithopter")
        };
        if saheeli {
            let s = t.battlefield(P0, "Saheeli, Sublime Artificer");
            t.answer_targets(P0, &[obj(splitter)]);
            t.answer_targets(P0, &[obj(into)]);
            activate_containing(&mut t, P0, s, "becomes a copy").expect("−2");
            t.resolve_all();
        } else {
            cast_spell(
                &mut t,
                P0,
                "True Polymorph",
                "{4}{U}{U}",
                &[obj(splitter), obj(into)],
                None,
                None,
            );
        }
        t.settle();
        let case = format!("Saheeli: {saheeli}, into an Equipment: {into_equipment}");
        if into_equipment {
            assert_eq!(name_of(&t, splitter), "Leonin Scimitar", "{case}");
            assert_eq!(attached_to(&t, splitter), Some(obj(bears)));
            // Leonin Scimitar: +1/+1.
            assert_eq!(t.pt(bears), (3, 3));
        } else {
            assert_eq!(name_of(&t, splitter), "Ornithopter", "{case}");
            assert_eq!(attached_to(&t, splitter), None);
            assert_eq!(t.pt(bears), (2, 2));
        }
    }
}

#[test]
fn an_artifact_that_was_a_copy_of_an_equipment_becomes_unattached_when_it_stops() {
    cr!("301.5c", "704.5p", "514.2");
    ruling!("Saheeli, Sublime Artificer", "If the target artifact isn't an Equipment and becomes a copy of an Equipment, it'll become unattached when it becomes a non-Equipment artifact again.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let stone = t.battlefield(P0, "Mind Stone");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let s = t.battlefield(P0, "Saheeli, Sublime Artificer");
    t.answer_targets(P0, &[obj(stone)]);
    t.answer_targets(P0, &[obj(splitter)]);
    activate_containing(&mut t, P0, s, "becomes a copy").expect("−2");
    t.resolve_all();
    assert_eq!(name_of(&t, stone), "Bonesplitter");
    // Equip {1}.
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, stone, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, stone), Some(obj(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    // The copy effect ends: a Mind Stone again, unattached.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, stone), "Mind Stone");
    assert_eq!(attached_to(&t, stone), None);
    assert_eq!(t.pt(bears), (2, 2));
}
