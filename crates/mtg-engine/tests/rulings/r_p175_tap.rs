//! Rulings batch P175 — effects that tap permanents: what they can target (untapped or
//! already tapped permanents, any controller), "doesn't untap during its controller's next
//! untap step" across control changes and skipped untap steps (Lorthos), skipping untap
//! steps (Yosei), illegal targets (Yosei, Magma Opus), and "permanents enter tapped this
//! turn" (Due Respect).

use crate::r_p175_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Yosei dies with its trigger targeting `player` and `perms`; the trigger is left on
/// the stack.
fn yosei_dies(t: &mut TestGame, player: PlayerId, perms: &[ObjectId]) -> ObjectId {
    let yosei = t.battlefield(P0, "Yosei, the Morning Star");
    t.answer_targets(P0, &[pl(player)]);
    let ps: Vec<Entity> = perms.iter().map(|p| obj(*p)).collect();
    t.answer_targets(P0, &ps);
    t.g.destroy(yosei, None);
    t.settle();
    assert_eq!(t.stack_len(), 1, "Yosei's trigger on the stack");
    yosei
}

/// Advances to `p`'s next upkeep (after that player's untap step, skipped or not).
fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        pass_turn(t);
    }
    t.advance_to(p, Step::Upkeep);
}

#[test]
fn yosei_skips_as_many_untap_steps_as_times_it_died() {
    cr!("614.10", "614.10a", "502.3");
    ruling!(
        "Yosei, the Morning Star",
        "If more than one effect instructs a player to skip their next untap step, such as if Yosei dies more than once in a turn, that player skips that many untap steps. This is different from effects that state that something doesn't untap during an untap step."
    );
    supported("Yosei, the Morning Star");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Yosei dies twice in P0's turn (a second Yosei, as if it returned and died again).
    yosei_dies(&mut t, P1, &[bears]);
    t.resolve_all();
    yosei_dies(&mut t, P1, &[bears]);
    t.resolve_all();
    assert!(tapped(&t, bears));
    // P1's next two untap steps are skipped; the third untaps the Bears.
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, bears), "first untap step skipped");
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, bears), "second untap step skipped");
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears), "third untap step happens");
    // Control: two "doesn't untap during its controller's next untap step" effects on the
    // same permanent both end in that one untap step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lorthos = t.battlefield(P0, "Lorthos, the Tidemaker");
    for i in 0..2 {
        // Lorthos and the Bears untapped again, so that each attack's ability shows.
        untap(&mut t, lorthos);
        untap(&mut t, bears);
        lorthos_taps(&mut t, lorthos, &[bears]);
        assert!(tapped(&t, bears), "Lorthos's ability {i} tapped the Bears");
        t.set_step(P0, Step::PrecombatMain);
        t.g.combat = None;
    }
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, bears));
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears), "both effects ended in one untap step");
}

/// P0's Lorthos attacks P1 and P0 pays {8} to tap `targets`; everything resolves.
fn lorthos_taps(t: &mut TestGame, lorthos: ObjectId, targets: &[ObjectId]) {
    let ts: Vec<Entity> = targets.iter().map(|p| obj(*p)).collect();
    t.answer_targets(P0, &ts);
    t.answer_yes(P0, true);
    attack_with(t, &[(lorthos, pl(P1))]);
    lots(t, P0);
    lots(t, P0);
    t.resolve_all();
}

#[test]
fn a_doesnt_untap_effect_waits_for_an_untap_step_that_isnt_skipped() {
    cr!("614.10a", "502.3");
    ruling!(
        "Yosei, the Morning Star",
        "If a permanent won't untap during its controller's next untap step, that effect waits to apply until its controller has an untap step that isn't skipped."
    );
    supported("Lorthos, the Tidemaker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lorthos = t.battlefield(P0, "Lorthos, the Tidemaker");
    // Lorthos taps the Bears ("don't untap during their controllers' next untap steps"),
    // then Yosei dies: P1 skips their next untap step.
    lorthos_taps(&mut t, lorthos, &[bears]);
    assert!(tapped(&t, bears));
    t.set_step(P0, Step::PostcombatMain);
    yosei_dies(&mut t, P1, &[]);
    t.resolve_all();
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, bears), "untap step skipped");
    next_upkeep(&mut t, P1);
    assert!(
        tapped(&t, bears),
        "Lorthos's effect applies in the first untap step that isn't skipped"
    );
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears));
}

#[test]
fn yosei_with_illegal_permanent_targets() {
    cr!("608.2b", "614.10");
    ruling!(
        "Yosei, the Morning Star",
        "If all of the permanents targeted by Yosei's ability become illegal targets, nothing becomes tapped, but the target player still skips their untap step."
    );
    ruling!(
        "Yosei, the Morning Star",
        "If any of the targets of Yosei's ability are illegal, they won't be tapped."
    );
    // `stolen`: which of P1's two permanents P0 gains control of with the trigger on the
    // stack (no longer "that player controls": an illegal target).
    for stolen in [vec![0usize], vec![0, 1]] {
        let mut t = TestGame::new(2);
        let perms = [
            t.battlefield(P1, "Grizzly Bears"),
            t.battlefield(P1, "Hill Giant"),
        ];
        let other = t.battlefield(P1, "Forest");
        tap(&mut t, other);
        yosei_dies(&mut t, P1, &perms);
        for i in &stolen {
            give_control(&mut t, perms[*i], P0);
        }
        t.resolve_all();
        for (i, p) in perms.iter().enumerate() {
            assert_eq!(tapped(&t, *p), !stolen.contains(&i), "{stolen:?} {i}");
        }
        // P1 still skips their untap step: the Forest stays tapped.
        next_upkeep(&mut t, P1);
        assert!(tapped(&t, other), "{stolen:?}: P1 skipped the untap step");
        next_upkeep(&mut t, P1);
        assert!(!tapped(&t, other));
    }
}

#[test]
fn lorthos_effect_ends_if_the_permanent_is_untapped_at_that_untap_step() {
    cr!("502.3", "611.2a");
    ruling!(
        "Lorthos, the Tidemaker",
        "If a targeted permanent is untapped at the time its controller's next untap step begins, this ability has no effect on it. It won't apply at some later time when the targeted permanent is tapped."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lorthos = t.battlefield(P0, "Lorthos, the Tidemaker");
    lorthos_taps(&mut t, lorthos, &[bears]);
    assert!(tapped(&t, bears));
    // Something untaps the Bears before P1's untap step.
    untap(&mut t, bears);
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears));
    // Tapped later, it untaps as usual during P1's following untap step.
    tap(&mut t, bears);
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears));
}

#[test]
fn lorthos_effect_follows_a_permanent_to_its_new_controller() {
    cr!("502.3", "611.2c");
    ruling!(
        "Lorthos, the Tidemaker",
        "If an affected permanent changes controllers before its old controller's next untap step, this ability will prevent it from being untapped during its new controller's next untap step."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lorthos = t.battlefield(P0, "Lorthos, the Tidemaker");
    lorthos_taps(&mut t, lorthos, &[bears]);
    // P2 gains control of the Bears before P1's untap step.
    give_control(&mut t, bears, P2);
    next_upkeep(&mut t, P1);
    next_upkeep(&mut t, P2);
    assert!(tapped(&t, bears), "doesn't untap in P2's next untap step");
    next_upkeep(&mut t, P2);
    assert!(!tapped(&t, bears));
}

#[test]
fn lorthos_targets_any_permanents() {
    cr!("115.1", "601.2c", "502.3");
    ruling!(
        "Lorthos, the Tidemaker",
        "You may target any eight (or fewer) permanents. It's okay if any of them are already tapped, and it's okay if any of them are controlled by someone other than the defending player."
    );
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Forest");
    let tapped_one = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, tapped_one);
    let third = t.battlefield(P2, "Hill Giant");
    let lorthos = t.battlefield(P0, "Lorthos, the Tidemaker");
    // More permanents than eight, so the limit shows.
    t.lands(P2, "Forest", 6);
    // Lorthos attacks P1; it targets P0's own land, P1's tapped Bears and P2's Giant.
    let from = t.asked().len();
    lorthos_taps(&mut t, lorthos, &[mine, tapped_one, third]);
    assert_eq!(target_bounds(&t, P0, from), vec![(0, 8)]);
    let cands = target_choices(&t, P0, from, "");
    assert!(
        [mine, tapped_one, third, lorthos]
            .iter()
            .all(|p| cands[0].contains(&obj(*p))),
        "{cands:?}"
    );
    for p in [mine, tapped_one, third] {
        assert!(tapped(&t, p));
    }
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, tapped_one));
    next_upkeep(&mut t, P2);
    assert!(tapped(&t, third));
    next_upkeep(&mut t, P0);
    assert!(tapped(&t, mine));
}

#[test]
fn due_respect_taps_only_permanents_entering_after_it_resolves() {
    cr!("614.1d", "611.2a");
    ruling!(
        "Due Respect",
        "Due Respect won't tap any permanents that have entered before it resolved."
    );
    supported("Due Respect");
    let mut t = TestGame::new(2);
    let before = t.enter(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Due Respect", &[]);
    t.resolve_all();
    assert!(!tapped(&t, before));
    let after = t.enter(P1, "Hill Giant");
    assert!(tapped(&t, after));
}

#[test]
fn tempest_owl_kicked_taps_zero_to_three_tapped_or_untapped_permanents() {
    cr!("115.1", "601.2c", "603.3d");
    ruling!(
        "Tempest Owl",
        "If Tempest Owl was kicked, you may target zero, one, two, or three permanents. They don't have to be untapped."
    );
    supported("Tempest Owl");
    for n in [0usize, 1, 3] {
        let mut t = TestGame::new(2);
        let perms = [
            t.battlefield(P1, "Grizzly Bears"),
            t.battlefield(P1, "Hill Giant"),
            t.battlefield(P1, "Forest"),
        ];
        tap(&mut t, perms[2]);
        let ts: Vec<Entity> = perms[..n].iter().map(|p| obj(*p)).collect();
        lots(&mut t, P0);
        let owl = t.hand(P0, "Tempest Owl");
        t.cast(P0, owl).kicked(true).go();
        let from = t.asked().len();
        t.answer_targets(P0, &ts);
        t.resolve_all();
        assert_eq!(target_bounds(&t, P0, from), vec![(0, 3)]);
        // The tapped Forest is a legal target as well as the untapped creatures.
        let cands = target_choices(&t, P0, from, "");
        assert!(
            perms.iter().all(|p| cands[0].contains(&obj(*p))),
            "{cands:?}"
        );
        for (i, p) in perms.iter().enumerate() {
            assert_eq!(tapped(&t, *p), i < n || i == 2, "{n} targets, {i}");
        }
    }
}

#[test]
fn topple_the_statue_and_firbolg_flutist_target_regardless_of_tapped_status() {
    cr!("115.1", "608.2c", "701.26a");
    ruling!(
        "Topple the Statue",
        "Topple the Statue can target any permanent, even one that's already tapped."
    );
    ruling!(
        "Firbolg Flutist",
        "Firbolg Flutist's triggered ability can target any creature you don't control, even one that is untapped."
    );
    supported("Topple the Statue");
    supported("Firbolg Flutist");
    // Topple the Statue on an already tapped artifact: destroyed, and a card drawn.
    let mut t = TestGame::new(2);
    let rock = t.battlefield(P1, "Mind Stone");
    tap(&mut t, rock);
    let spell = t.hand(P0, "Topple the Statue");
    let hand = t.hand_size(P0);
    lots(&mut t, P0);
    t.cast(P0, spell).target(rock).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Mind Stone"));
    assert_eq!(t.hand_size(P0), hand);
    // Firbolg Flutist on an untapped creature: P0 gains control of it, untapped, with haste.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Firbolg Flutist");
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    assert!(!tapped(&t, bears));
}

/// P0 casts Magma Opus dividing its damage `split` among `dmg` and tapping `taps`; the
/// spell is left on the stack.
fn magma_opus(t: &mut TestGame, dmg: &[Entity], split: &[i64], taps: &[ObjectId]) -> ObjectId {
    lots(t, P0);
    lots(t, P0);
    let spell = t.hand(P0, "Magma Opus");
    let ts: Vec<Entity> = taps.iter().map(|p| obj(*p)).collect();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(split.to_vec()));
    t.cast(P0, spell).targets(dmg).targets(&ts).go()
}

#[test]
fn magma_opus_chooses_all_targets_and_the_division_as_it_is_cast() {
    cr!("601.2c", "601.2d", "115.3");
    ruling!(
        "Magma Opus",
        "You choose all targets for Magma Opus as you cast the spell. This includes up to four targets for the first effect and two target permanents for the second effect."
    );
    ruling!(
        "Magma Opus",
        "You also choose how the 4 damage will be divided among the first group of targets as you cast Magma Opus. You must have at least 1 damage dealt to each of those targets."
    );
    supported("Magma Opus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let land = t.battlefield(P1, "Forest");
    let from = t.asked().len();
    let spell = magma_opus(
        &mut t,
        &[obj(bears), obj(giant), obj(elves), pl(P1)],
        &[1, 1, 1, 1],
        &[land, giant],
    );
    // Everything is chosen before the spell resolves: all six targets and the division
    // (at least 1 to each of the four).
    assert!(on_stack(&t, spell));
    assert_eq!(stack_targets(&t, spell).len(), 6);
    assert_eq!(target_bounds(&t, P0, from), vec![(0, 4), (2, 2)]);
    let divides: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                min_each,
                recipients,
                ..
            } => {
                assert_eq!(recipients.len(), 4);
                Some((*total, *min_each))
            }
            _ => None,
        })
        .collect();
    assert_eq!(divides, vec![(4, 1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(tapped(&t, land) && tapped(&t, giant));
}

#[test]
fn magma_opus_can_damage_and_tap_the_same_permanent() {
    cr!("115.3", "601.2c");
    ruling!(
        "Magma Opus",
        "The same permanent can be chosen as a target for each of the two instances of \"target.\""
    );
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    let land = t.battlefield(P1, "Forest");
    let spell = magma_opus(&mut t, &[obj(maw)], &[4], &[maw, land]);
    assert_eq!(
        stack_targets(&t, spell),
        vec![obj(maw), obj(maw), obj(land)]
    );
    t.resolve_all();
    assert!(tapped(&t, maw) && tapped(&t, land));
    assert_eq!(crate::r_p009_common::dmg(&t, maw), 4);
}

#[test]
fn magma_opus_with_all_targets_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Magma Opus",
        "If somehow all of Magma Opus's targets are illegal as it tries to resolve, none of its effects happen. You will not create an Elemental token or draw two cards."
    );
    // `all`: every target leaves the battlefield; otherwise one tap target stays.
    for all in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let land = t.battlefield(P1, "Forest");
        let hand = t.hand_size(P0);
        magma_opus(&mut t, &[obj(bears)], &[4], &[giant, land]);
        bounce(&mut t, bears);
        bounce(&mut t, giant);
        if all {
            bounce(&mut t, land);
        }
        t.resolve_all();
        let elementals = with_subtype(&t, P0, "Elemental").len();
        if all {
            assert_eq!(elementals, 0);
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert!(tapped(&t, land));
            assert_eq!(elementals, 1);
            assert_eq!(t.hand_size(P0), hand + 2);
        }
    }
}
