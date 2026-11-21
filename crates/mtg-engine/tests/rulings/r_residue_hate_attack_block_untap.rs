//! Rulings residue `rulings-residue-hate-attack-block` (P079, P080, P082): "doesn't untap
//! during its controller's (next) untap step" effects on attackers and blockers (Ice Floe,
//! Spore Cloud, Tangle, Joven's Ferrets): they follow the object, not its controller
//! (CR 502.3, 611.2c), apply to the creatures identified as the spell or ability resolves
//! (CR 611.2c), and a "next untap step" effect is used up by that untap step (CR 502.3).

use crate::r_p079_common::{cast_card, enter_attacking};
use crate::r_residue_hate_attack_block_common::*;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 has Ice Floe; P0 attacks P1 with a Hill Giant and P1 taps Ice Floe targeting it in
/// the declare attackers step. (Ice Floe, the Giant.)
fn floe_on_giant(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Ice Floe");
    let floe = t.battlefield(P1, "Ice Floe");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(t, &[(giant, Entity::Player(P1))]);
    t.activate(P1, floe, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(tapped(t, giant) && tapped(t, floe));
    (floe, giant)
}

#[test]
fn ice_floe_doesnt_remove_the_attacker_from_combat() {
    cr!("506.4b");
    ruling!(
        "Ice Floe",
        "The targeted creature is not removed from combat. Its attack will continue as normal."
    );
    let mut t = TestGame::new(2);
    let (_, giant) = floe_on_giant(&mut t);
    assert!(attacking(&t, giant));
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn ice_floe_follows_the_creature_to_a_new_controller() {
    cr!("502.3");
    ruling!(
        "Ice Floe",
        "Ice Floe doesn’t track the creature’s controller. If the affected creature changes controllers, Ice Floe will prevent it from being untapped during its new controller’s untap step."
    );
    let mut t = TestGame::new(2);
    let (floe, giant) = floe_on_giant(&mut t);
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::PostcombatMain);
    give_control(&mut t, giant, P1);
    // P1 chooses not to untap Ice Floe.
    t.answer_yes(P1, false);
    through_untap_of(&mut t, P1);
    assert!(tapped(&t, floe));
    assert!(tapped(&t, giant), "{}", t.dump_log());
}

#[test]
fn ice_floe_effect_ends_when_it_untaps_or_leaves_without_untapping_the_creature() {
    cr!("611.2b", "502.3");
    ruling!(
        "Ice Floe",
        "If Ice Floe untaps or leaves the battlefield, its effect will end. This has no immediate visible affect on the creature. (It doesn’t untap immediately, for example.) The creature will just untap as normal during its controller’s next untap step."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let (floe, giant) = floe_on_giant(&mut t);
        if leaves {
            crate::r_s02_common::destroy(&mut t, floe);
        } else {
            untap(&mut t, floe);
        }
        assert!(tapped(&t, giant), "leaves: {leaves}");
        no_blocks(&mut t, P1);
        through_untap_of(&mut t, P0);
        assert!(!tapped(&t, giant), "leaves: {leaves}");
    }
}

#[test]
fn ice_floe_effect_continues_if_the_creature_later_gains_flying() {
    cr!("608.2b", "611.2a");
    ruling!(
        "Ice Floe",
        "If the ability resolves, then the creature gains flying later, Ice Floe’s effect will not end. (The ability checks whether the creature has flying only at the time you activate it and the time it resolves.)"
    );
    let mut t = TestGame::new(2);
    let (_, giant) = floe_on_giant(&mut t);
    cast_card(&mut t, P0, "Jump", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(t
        .obj_now(giant)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    no_blocks(&mut t, P1);
    // P1 keeps Ice Floe tapped during their untap step.
    t.answer_yes(P1, false);
    through_untap_of(&mut t, P0);
    assert!(tapped(&t, giant));
    // Flying given in response, before the ability resolves: its target is illegal
    // (CR 608.2b), so the Giant untaps as usual.
    let mut t = TestGame::new(2);
    let floe = t.battlefield(P1, "Ice Floe");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    t.activate(P1, floe, 0, &[obj(giant)]).unwrap();
    cast_card(&mut t, P0, "Jump", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(tapped(&t, floe) && tapped(&t, giant));
    no_blocks(&mut t, P1);
    t.answer_yes(P1, false);
    through_untap_of(&mut t, P0);
    assert!(tapped(&t, floe));
    assert!(!tapped(&t, giant), "{}", t.dump_log());
}

#[test]
fn ice_floe_effect_survives_the_creature_being_untapped_and_tapped_again() {
    cr!("611.2a", "502.3");
    ruling!(
        "Ice Floe",
        "If the affected creature is untapped by some other spell or ability, Ice Floe’s effect will not end. If you keep Ice Floe tapped, and that creature becomes tapped again, Ice Floe will continue to prevent it from being untapped during its controller’s untap step."
    );
    let mut t = TestGame::new(2);
    let (_, giant) = floe_on_giant(&mut t);
    untap(&mut t, giant);
    assert!(!tapped(&t, giant));
    tap(&mut t, giant);
    no_blocks(&mut t, P1);
    // P1 keeps Ice Floe tapped during their untap step.
    t.answer_yes(P1, false);
    through_untap_of(&mut t, P0);
    assert!(tapped(&t, giant));
}

/// P0 attacks P1 with `attackers`; P1 casts `spell` (Spore Cloud or Tangle) in the declare
/// attackers step, before blocks.
fn fog_after_attacks(t: &mut TestGame, spell: &str, attackers: &[ObjectId]) {
    supported(spell);
    let at: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attacks(t, &at);
    cast_card(t, P1, spell, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn spore_cloud_affects_only_creatures_already_attacking_or_blocking() {
    cr!("608.2h", "502.3");
    ruling!(
        "Spore Cloud",
        "Only makes attackers or blockers that have been declared before the spell is cast unable to untap as normal next turn. So, if used before attackers or blockers are declared, it is simply a Fog-like effect. If used after attackers are declared but before blockers are declared, it does the Fog effect and makes attackers not untap as normal. If done after blockers are declared, it has full effect."
    );
    // Before attackers: just a Fog.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    cast_card(&mut t, P1, "Spore Cloud", &[]).unwrap();
    t.resolve_all();
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    through_untap_of(&mut t, P0);
    assert!(!tapped(&t, giant));
    // After attackers, before blockers: the attacker doesn't untap, a blocker declared
    // later isn't affected (it isn't tapped, and untaps if it becomes tapped).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fog_after_attacks(&mut t, "Spore Cloud", &[giant]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(bears, giant)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(bears) && !tapped(&t, bears));
    tap(&mut t, bears);
    through_untap_of(&mut t, P1);
    assert!(!tapped(&t, bears));
    through_untap_of(&mut t, P0);
    assert!(tapped(&t, giant));
    // After blockers: the blocker is tapped and doesn't untap either.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, &[giant], &[(bears, giant)]);
    cast_card(&mut t, P1, "Spore Cloud", &[]).unwrap();
    t.resolve_all();
    assert!(tapped(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    through_untap_of(&mut t, P1);
    assert!(tapped(&t, bears));
}

#[test]
fn next_untap_step_effects_are_used_up_by_that_untap_step() {
    cr!("502.3", "611.2a");
    ruling!(
        "Spore Cloud",
        "If a creature affected by Spore Cloud is untapped at the time its controller’s next untap step begins, the “doesn’t untap” effect doesn’t do anything. It won’t apply at some later time when that creature is tapped."
    );
    // Serra Angel attacks without tapping (vigilance).
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    fog_after_attacks(&mut t, "Spore Cloud", &[angel]);
    no_blocks(&mut t, P1);
    assert!(!tapped(&t, angel));
    through_untap_of(&mut t, P0);
    tap(&mut t, angel);
    through_untap_of(&mut t, P0);
    assert!(!tapped(&t, angel));
}

#[test]
fn next_untap_step_effects_follow_the_creature_to_a_new_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Spore Cloud",
        "Spore Cloud doesn’t track the creatures’ controllers. If an affected creature changes controllers before its old controller’s next untap step, Spore Cloud will prevent it from being untapped during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    fog_after_attacks(&mut t, "Spore Cloud", &[giant]);
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::PostcombatMain);
    give_control(&mut t, giant, P1);
    through_untap_of(&mut t, P1);
    assert!(tapped(&t, giant));
    // Used up: it untaps during P1's following untap step.
    through_untap_of(&mut t, P1);
    assert!(!tapped(&t, giant));
}

#[test]
fn tangle_identifies_the_attackers_as_it_resolves() {
    cr!("608.2h", "508.4");
    ruling!(
        "Tangle",
        "It identifies the creatures that are not going to untap at the time this spell resolves."
    );
    ruling!(
        "Tangle",
        "The creatures that are not going to untap are determined at the time this spell resolves."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fog_after_attacks(&mut t, "Tangle", &[giant, bears]);
    // Afterwards: a creature put onto the battlefield attacking, and the Bears removed
    // from combat.
    let late = enter_attacking(&mut t, P0, "Craw Wurm", Entity::Player(P1));
    assert!(attacking(&t, late));
    mtg_engine::combat::remove_from_combat(&mut t.g, bears);
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    through_untap_of(&mut t, P0);
    assert!(tapped(&t, giant));
    assert!(tapped(&t, bears));
    assert!(!tapped(&t, late));
}

/// P0 attacks P1 with Joven's Ferrets and P1 blocks it with `blocker`.
fn ferrets_blocked_by(t: &mut TestGame, blocker: &str) -> (ObjectId, ObjectId) {
    supported("Joven's Ferrets");
    let ferrets = t.battlefield(P0, "Joven's Ferrets");
    let b = t.battlefield(P1, blocker);
    blocks(t, &[ferrets], &[(b, ferrets)]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    (ferrets, b)
}

#[test]
fn joven_s_ferrets_must_survive_to_end_of_combat() {
    cr!("511.2", "603.2");
    ruling!(
        "Joven's Ferrets",
        "The second ability will trigger only if Joven’s Ferrets is still on the battlefield at end of combat. If it’s been dealt lethal combat damage and destroyed during that combat (or has left the battlefield during that combat by any other means), its ability won’t trigger and the creatures that blocked it won’t be affected."
    );
    // A 1/3 (attacking) blocked by Hill Giant: destroyed by combat damage.
    let mut t = TestGame::new(2);
    let (ferrets, giant) = ferrets_blocked_by(&mut t, "Hill Giant");
    assert!(!t.on_battlefield(ferrets));
    assert!(!tapped(&t, giant));
}

#[test]
fn joven_s_ferrets_untapped_blocker_isnt_affected_later() {
    cr!("502.3", "611.2a");
    ruling!(
        "Joven's Ferrets",
        "If a creature affected by the second ability is untapped at the time its controller’s next untap step begins, the “doesn’t untap” effect doesn’t do anything. It won’t apply at some later time when that creature is tapped."
    );
    let mut t = TestGame::new(2);
    let (_, bears) = ferrets_blocked_by(&mut t, "Grizzly Bears");
    assert!(tapped(&t, bears));
    untap(&mut t, bears);
    through_untap_of(&mut t, P1);
    tap(&mut t, bears);
    through_untap_of(&mut t, P1);
    assert!(!tapped(&t, bears));
}

#[test]
fn joven_s_ferrets_follows_the_blocker_to_a_new_controller() {
    cr!("502.3");
    ruling!(
        "Joven's Ferrets",
        "The second ability doesn’t track the creatures’ controllers. If an affected creature changes controllers before its old controller’s next untap step, Joven’s Ferrets will prevent it from being untapped during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let (_, bears) = ferrets_blocked_by(&mut t, "Grizzly Bears");
    assert!(tapped(&t, bears));
    t.advance_to(P0, Step::PostcombatMain);
    give_control(&mut t, bears, P0);
    through_untap_of(&mut t, P0);
    assert!(tapped(&t, bears));
}
