//! Rulings batch P012 — copies of spells made by Pyromancer Ascension, Psychic Rebuttal and
//! Ivy, Gleeful Spellthief: a copy keeps the original's X, division, modes and paid costs
//! (CR 707.10), isn't cast (CR 707.10), and "you" in it is its controller (CR 707.10, 109.5).

use crate::r_p012_common::*;
use crate::r_p030_common::{fling_giant, kill, resolve_until_copies};
use crate::r_s01_common::{give_mana_for as lands_for, supported};
use crate::r_s04_common::spell_targets;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets, spell_copies, targets_of, x_of};
use crate::r_s26_common::modes_on_stack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

// --- Pyromancer Ascension -------------------------------------------------------------------

/// Pyromancer Ascension for P0 with `n` quest counters.
fn ascension(t: &mut TestGame, n: u32) -> ObjectId {
    supported("Pyromancer Ascension");
    let a = t.battlefield(P0, "Pyromancer Ascension");
    if n > 0 {
        t.g.add_counters(obj(a), "quest", n, None);
    }
    a
}

#[test]
fn pyromancer_ascensions_copy_keeps_the_division() {
    cr!("707.10c", "601.2d");
    ruling!(
        "Pyromancer Ascension",
        "If the spell has damage divided as it was cast (like Fiery Justice does), the division can’t be changed (although the targets receiving that damage still can)."
    );
    let mut t = TestGame::new(2);
    ascension(&mut t, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Arc Lightning: 2 to the Bears, 1 to P1.
    t.answer_targets(P0, &[obj(bears), pl(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.answer_yes(P0, true);
    // The copy: the Giant instead of the Bears.
    change_copy_targets(&mut t, P0, &[Some(obj(giant)), None]);
    let from = t.asked().len();
    cast_card(&mut t, P0, "Arc Lightning", &[]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(giant), pl(P1)]);
    let divides = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Divide { .. }))
        .count();
    assert_eq!(divides, 1, "the copy's division isn't chosen again");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn pyromancer_ascensions_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Pyromancer Ascension",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Bonfire of the Damned does), the copy will have the same value of X."
    );
    let mut t = TestGame::new(2);
    ascension(&mut t, 2);
    t.lands(P0, "Mountain", 3);
    let blaze = t.hand(P0, "Blaze");
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.cast(P0, blaze).x(2).target(P1).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn pyromancer_ascension_copies_spells_without_targets() {
    cr!("707.10", "603.2");
    ruling!(
        "Pyromancer Ascension",
        "Pyromancer Ascension’s second ability can copy any instant or sorcery spell, not just one with targets."
    );
    let mut t = TestGame::new(2);
    ascension(&mut t, 2);
    t.answer_yes(P0, true);
    cast_card(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn pyromancer_ascensions_copy_ability_checks_the_counters_as_the_spell_is_cast() {
    cr!("603.2", "113.7a");
    ruling!(
        "Pyromancer Ascension",
        "The second ability triggers only if Pyromancer Ascension already has two quest counters on it at the time you cast an instant or sorcery spell. This means a spell can’t cause the second counter to be put on Pyromancer Ascension and then become copied. It also means a player can’t remove Pyromancer Ascension in response to you casting an instant or sorcery spell in order to prevent the ability from triggering."
    );
    // One counter, a Lightning Bolt in the graveyard: casting Bolt adds the second counter
    // but isn't copied.
    let mut t = TestGame::new(2);
    let asc = ascension(&mut t, 1);
    t.graveyard(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    cast_card(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    assert_eq!(t.stack_len(), 2, "only the quest counter trigger");
    t.resolve_all();
    assert_eq!(t.counters(asc, "quest"), 2);
    assert_eq!(t.life(P1), 17);

    // Two counters: the copy trigger resolves even if the Ascension is destroyed in
    // response.
    let mut t = TestGame::new(2);
    let asc = ascension(&mut t, 2);
    cast_card(&mut t, P0, "Divination", &[]);
    assert_eq!(t.stack_len(), 2);
    kill(&mut t, asc);
    assert!(!t.g.is_live(asc));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn pyromancer_ascension_and_a_spell_cast_from_the_graveyard() {
    cr!("603.2", "601.2a", "702.34a");
    ruling!(
        "Pyromancer Ascension",
        "If you cast an instant or sorcery spell from your graveyard (due to an ability such as flashback, for example), Pyromancer Ascension’s first ability won’t trigger unless another card with the same name is in your graveyard."
    );
    for other in [false, true] {
        let mut t = TestGame::new(2);
        let asc = ascension(&mut t, 0);
        t.lands(P0, "Island", 3);
        let tt = t.graveyard(P0, "Think Twice");
        if other {
            t.graveyard(P0, "Think Twice");
        }
        t.answer_yes(P0, true);
        t.cast(P0, tt)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .go();
        t.settle();
        assert_eq!(trigger_on_top(&t), other);
        t.resolve_all();
        assert_eq!(t.counters(asc, "quest"), other as u32);
    }
}

// --- Psychic Rebuttal -----------------------------------------------------------------------

/// P0 (with spell mastery: two Opts in their graveyard) casts Psychic Rebuttal on `spell`
/// and copies it, choosing the new targets `new`.
fn rebut_and_copy(t: &mut TestGame, spell: ObjectId, new: &[Option<Entity>]) {
    supported("Psychic Rebuttal");
    fill_graveyard(t, P0, "Opt", 2);
    cast_card(t, P0, "Psychic Rebuttal", &[obj(spell)]);
    t.answer_yes(P0, true);
    change_copy_targets(t, P0, new);
    t.resolve_all();
}

#[test]
fn psychic_rebuttals_copy_says_you_for_its_controller() {
    cr!("707.10", "109.5", "707.10c");
    ruling!(
        "Psychic Rebuttal",
        "If the copy says that it affects “you,” it affects the controller of the copy, not the controller of the original spell. Similarly, if the copy says that it affects an “opponent,” it affects an opponent of the copy’s controller, not an opponent of the original spell’s controller."
    );
    // P1's Lightning Helix at P0 ("deals 3 damage to any target and you gain 3 life"),
    // copied by P0 at P1: P0 gains the 3 life.
    let mut t = TestGame::new(2);
    let helix = cast_card(&mut t, P1, "Lightning Helix", &[pl(P0)]);
    rebut_and_copy(&mut t, helix, &[Some(pl(P1))]);
    assert_eq!(t.zone(helix), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn psychic_rebuttals_copy_keeps_the_paid_additional_costs() {
    cr!("707.10", "707.2");
    ruling!(
        "Psychic Rebuttal",
        "You can’t choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too. For example, if a player sacrifices a 3/3 creature to cast Fling, and you copy it, the copy of Fling will also deal 3 damage to its target."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fling = fling_giant(&mut t, P1, pl(P0));
    rebut_and_copy(&mut t, fling, &[Some(pl(P1))]);
    assert!(t.on_battlefield(bears), "P0 sacrificed nothing");
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
}

/// `p` casts Electrolyze dividing 1 damage to each of the two targets.
fn electrolyze(t: &mut TestGame, p: PlayerId, a: Entity, b: Entity) -> ObjectId {
    t.answer_targets(p, &[a, b]);
    t.answer(p, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    cast_card(t, p, "Electrolyze", &[])
}

#[test]
fn psychic_rebuttal_can_target_a_spell_with_other_targets_too() {
    cr!("115.1", "701.6a");
    ruling!(
        "Psychic Rebuttal",
        "The target instant or sorcery spell can have other targets as long as one of them is you."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = cast_card(&mut t, P1, "Shock", &[obj(bears)]);
    let ely = electrolyze(&mut t, P1, pl(P0), obj(bears));
    let legal = spell_targets(&mut t, P0, "Psychic Rebuttal");
    assert!(legal.contains(&obj(ely)));
    assert!(!legal.contains(&obj(shock)), "Shock doesn't target P0");
    t.answer_yes(P0, false);
    cast_card(&mut t, P0, "Psychic Rebuttal", &[obj(ely)]);
    t.resolve();
    assert_eq!(t.zone(ely), Zone::Graveyard(P1));
}

// --- Ivy, Gleeful Spellthief ----------------------------------------------------------------

/// P0's Ivy (with `counters` +1/+1 counters) and P1's Grizzly Bears.
fn ivy(t: &mut TestGame, counters: u32) -> (ObjectId, ObjectId) {
    supported("Ivy, Gleeful Spellthief");
    let ivy = t.battlefield(P0, "Ivy, Gleeful Spellthief");
    if counters > 0 {
        t.g.add_counters(obj(ivy), "+1/+1", counters, None);
    }
    (ivy, t.battlefield(P1, "Grizzly Bears"))
}

#[test]
fn ivy_triggers_on_a_spell_that_targets_only_one_creature() {
    cr!("603.2", "115.3");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "The second ability triggers whenever a player casts a spell that targets only one creature and no other object or player. It doesn't matter who controls the creature."
    );
    ruling!(
        "Ivy, Gleeful Spellthief",
        "If a spell has multiple targets but all of them are the same creature, Ivy, Gleeful Spellthief's second ability will trigger."
    );
    // P1's Giant Growth on their own Bears, and P0's on P1's Bears, trigger P0's Ivy.
    for caster in [P1, P0] {
        let mut t = TestGame::new(2);
        let (_, bears) = ivy(&mut t, 0);
        cast_card(&mut t, caster, "Giant Growth", &[obj(bears)]);
        assert!(trigger_on_top(&t), "{caster:?}");
    }
    // Electrolyze at the Bears and P0 doesn't.
    let mut t = TestGame::new(2);
    let (_, bears) = ivy(&mut t, 0);
    electrolyze(&mut t, P1, obj(bears), pl(P0));
    assert!(!trigger_on_top(&t));
    // Seeds of Strength with all three targets the Bears: Ivy's copy gives it +3/+3.
    let mut t = TestGame::new(2);
    let (ivy_id, bears) = ivy(&mut t, 0);
    t.answer_yes(P0, true);
    cast_card(
        &mut t,
        P1,
        "Seeds of Strength",
        &[obj(bears), obj(bears), obj(bears)],
    );
    assert!(trigger_on_top(&t));
    t.resolve_all();
    assert_eq!(t.pt(ivy_id), (5, 4));
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn ivy_cant_copy_a_spell_it_isnt_a_legal_target_for() {
    cr!("707.10e");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "If Ivy, Gleeful Spellthief isn't a legal target for the spell, no copy can be created."
    );
    // Smite the Monstrous ("Destroy target creature with power 4 or greater") at P0's
    // Craw Wurm: Ivy (2/1) can't be its target.
    let mut t = TestGame::new(2);
    let (ivy_id, _) = ivy(&mut t, 0);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    cast_card(&mut t, P1, "Smite the Monstrous", &[obj(wurm)]);
    t.resolve();
    assert!(spell_copies(&t).is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(ivy_id));
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn ivys_copy_keeps_the_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    let mut t = TestGame::new(2);
    let (ivy_id, bears) = ivy(&mut t, 0);
    lands_for(&mut t, P1, "Boros Charm");
    let charm = t.hand(P1, "Boros Charm");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    let charm = t.cast(P1, charm).modes(&[2]).target(bears).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(modes_on_stack(&t, copies[0]), modes_on_stack(&t, charm));
    let p0_modes = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { .. }))
        .count();
    assert_eq!(p0_modes, 0);
    t.resolve_all();
    assert!(t.obj_now(ivy_id).has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn ivys_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "707.2", "702.33d");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "The controller of a copy can't choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too. Notably, if the spell you copy was kicked, the copies will also be kicked."
    );
    // Ivy is a 5/4 here: an unkicked Burst Lightning (2) wouldn't kill it, a kicked one (4)
    // does.
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let (ivy_id, bears) = ivy(&mut t, 3);
        t.lands(P1, "Mountain", 5);
        let bl = t.hand(P1, "Burst Lightning");
        t.answer_yes(P0, true);
        let from = t.asked().len();
        t.cast(P1, bl).kicked(kicked).target(bears).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(ivy_id), !kicked, "kicked: {kicked}");
        let p0_costs = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::OptionalCost { .. }))
            .count();
        assert_eq!(p0_costs, 0);
    }
}

#[test]
fn ivys_copy_is_not_cast() {
    cr!("707.10", "603.2");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "The copy that Ivy, Gleeful Spellthief's second ability creates is created on the stack, so it's not \"cast.\" Other abilities that trigger when a player casts a spell won't trigger."
    );
    let mut t = TestGame::new(2);
    let (ivy_id, bears) = ivy(&mut t, 0);
    t.battlefield(P0, "Ebon Drake");
    t.answer_yes(P0, true);
    cast_card(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(ivy_id), (5, 4), "the copy resolved");
    assert_eq!(t.life(P0), 19, "Ebon Drake triggered only for the Giant Growth");
}
