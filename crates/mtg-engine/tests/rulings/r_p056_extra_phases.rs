//! Rulings batch P056 — "untap all creatures you control" with additional combat phases
//! (Aurelia, the Warleader; Akki Battle Squad; Aggravated Assault; Full Throttle; Port
//! Razer), attack triggers that untap (Ahn-Crop Champion, Copperhorn Scout), additional
//! beginning phases and upkeep steps (Sphinx of the Second Sun, Shadow of the Second Sun,
//! Paradox Haze), and Nature's Will (CR 500.8, 500.9, 505.1a, 506.4, 508.1).

use crate::r_p056_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s04_common::stack_items;
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s29_common::{cast_and_resolve, put_counters};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The steps from the first end of combat step on.
fn after_first_combat(steps: &[Step]) -> Vec<Step> {
    steps
        .iter()
        .copied()
        .skip_while(|s| *s != Step::EndOfCombat)
        .collect()
}

/// P0 attacks P1 with `attacker` (answering yes to any "may"), and the turn runs out.
fn attack_and_finish_turn(t: &mut TestGame, attacker: ObjectId) -> Vec<Step> {
    t.answer_yes(P0, true);
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    steps_of_this_turn(t)
}

#[test]
fn aurelia_adds_a_combat_phase_but_no_main_phase() {
    cr!("500.8", "506.1");
    ruling!(
        "Aurelia, the Warleader",
        "Aurelia's last ability doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    supported("Aurelia, the Warleader");
    let mut t = TestGame::new(2);
    let aurelia = t.battlefield(P0, "Aurelia, the Warleader");
    let steps = attack_and_finish_turn(&mut t, aurelia);
    let after = after_first_combat(&steps);
    assert_eq!(after[1], Step::BeginningOfCombat, "{steps:?}");
    assert_eq!(count(&steps, Step::EndOfCombat), 2, "{steps:?}");
    assert_eq!(count(&steps, Step::PostcombatMain), 1);
}

#[test]
fn akki_battle_squad_adds_a_combat_phase_but_no_main_phase() {
    cr!("500.8", "506.1");
    ruling!(
        "Akki Battle Squad",
        "Notably, the triggered ability of Akki Battle Squad doesn't give you any additional main phases."
    );
    supported("Akki Battle Squad");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akki Battle Squad");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    let steps = attack_and_finish_turn(&mut t, bears);
    let after = after_first_combat(&steps);
    assert_eq!(after[1], Step::BeginningOfCombat, "{steps:?}");
    assert_eq!(count(&steps, Step::EndOfCombat), 2, "{steps:?}");
    assert_eq!(count(&steps, Step::PostcombatMain), 1);
}

#[test]
fn aggravated_assault_can_be_activated_more_than_once_a_turn() {
    cr!("602.5d", "500.8");
    ruling!(
        "Aggravated Assault",
        "If you have enough mana, the ability may be activated more than once in a turn."
    );
    ruling!(
        "Aggravated Assault",
        "You will normally activate this ability during your postcombat main phase so you can untap any creatures that attacked."
    );
    supported("Aggravated Assault");
    let mut t = TestGame::new(2);
    let assault = t.battlefield(P0, "Aggravated Assault");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 10);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    next_step(&mut t, Step::PostcombatMain);
    assert!(tapped(&t, bears));
    // In the postcombat main phase: the creature that attacked untaps.
    t.activate(P0, assault, 0, &[]).expect("first activation");
    t.resolve_all();
    assert!(!tapped(&t, bears));
    t.activate(P0, assault, 0, &[]).expect("second activation");
    t.resolve_all();
    let steps = steps_of_this_turn(&mut t);
    // Two more combat phases, each followed by a main phase.
    assert_eq!(count(&steps, Step::EndOfCombat), 3, "{steps:?}");
    assert_eq!(count(&steps, Step::PostcombatMain), 3);
}

#[test]
fn full_throttle_combat_phases_have_no_main_phase_between_them() {
    cr!("500.8");
    ruling!(
        "Full Throttle",
        "There is no main phase between the two additional combat phases. If you cast this during your second main phase, you will go directly to the end step after the two additional combat phases are complete."
    );
    supported("Full Throttle");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PostcombatMain);
    cast_and_resolve(&mut t, P0, "Full Throttle", &[]);
    let steps = steps_of_this_turn(&mut t);
    let after: Vec<Step> = steps
        .iter()
        .copied()
        .filter(|s| !matches!(s, Step::DeclareAttackers | Step::DeclareBlockers))
        .collect();
    assert_eq!(
        after,
        vec![
            Step::BeginningOfCombat,
            Step::EndOfCombat,
            Step::BeginningOfCombat,
            Step::EndOfCombat,
            Step::End,
            Step::Cleanup
        ]
    );
}

#[test]
fn full_throttle_outside_a_main_phase_and_in_an_opponents_main_phase() {
    cr!("500.8", "506.2");
    ruling!(
        "Full Throttle",
        "If you somehow cast this spell when it’s not a main phase, the second ability still takes effect, but there are no additional combat phases this turn. If you cast it during an opponent’s main phase, there are two additional combat phases, but that opponent gets to attack during those combat phases, not you."
    );
    // Leyline of Anticipation: "You may cast spells as though they had flash."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Anticipation");
    t.set_step(P0, Step::Upkeep);
    cast_and_resolve(&mut t, P0, "Full Throttle", &[]);
    // At the beginning of the (only) combat, its delayed ability triggers.
    next_step(&mut t, Step::BeginningOfCombat);
    assert_eq!(
        stack_items(&t),
        vec!["ability: delayed trigger".to_string()]
    );
    let steps = steps_of_this_turn(&mut t);
    assert_eq!(count(&steps, Step::EndOfCombat), 1);
    // In P1's precombat main phase: P1 gets the two combat phases and attacks in them.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Anticipation");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(&mut t, P0, "Full Throttle", &[]);
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    let steps = steps_of_this_turn(&mut t);
    assert_eq!(count(&steps, Step::EndOfCombat), 3, "{steps:?}");
    // The Bears attacked in both additional combat phases (untapping at the beginning of
    // the second one): P0 was dealt 2 damage twice.
    assert_eq!(t.life(P0), 16, "{}", t.dump_log());
}

#[test]
fn port_razer_doesnt_have_to_attack_in_the_additional_combat() {
    cr!("508.1d", "506.4");
    ruling!(
        "Port Razer",
        "Port Razer doesn't have to attack in any additional combat it creates"
    );
    ruling!(
        "Port Razer",
        "Port Razer's last ability doesn't stop it from attacking a planeswalker or battle during any combat."
    );
    supported("Port Razer");
    let mut t = TestGame::new(2);
    let razer = t.battlefield(P0, "Port Razer");
    let walker = t.battlefield(P1, "Jace Beleren");
    attack_with(&mut t, &[(razer, Entity::Player(P1))]);
    next_step(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!tapped(&t, razer), "it untapped");
    // The additional combat phase: no attack is required, the player can't be attacked
    // again, but their planeswalker can.
    next_step(&mut t, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(razer, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[(razer, obj(walker))]));
}

#[test]
fn ahn_crop_champion_attackers_are_declared_at_once() {
    cr!("508.1a", "508.1f", "508.1g");
    ruling!(
        "Ahn-Crop Champion",
        "All attackers are chosen at once. You can’t attack with Ahn-Crop Champion, untap a tapped creature, and then attack with that creature."
    );
    supported("Ahn-Crop Champion");
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Ahn-Crop Champion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    to_combat(&mut t, P0);
    assert!(!legal_attack(
        &mut t,
        &[(champion, Entity::Player(P1)), (bears, Entity::Player(P1))]
    ));
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(champion, Entity::Player(P1))]);
    t.resolve_all();
    // The Bears untapped, but they aren't attacking.
    assert!(!tapped(&t, bears));
    assert!(!t.g.is_attacking(bears));
    assert!(t.obj_now(champion).exerted);
}

#[test]
fn copperhorn_scout_untaps_attacking_and_other_creatures() {
    cr!("506.4", "508.1f");
    ruling!(
        "Copperhorn Scout",
        "you’ll untap each other creature you control regardless of whether that creature was attacking."
    );
    ruling!(
        "Copperhorn Scout",
        "Untapping an attacking creature doesn’t cause it to stop attacking."
    );
    supported("Copperhorn Scout");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Copperhorn Scout");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Hill Giant");
    tap(&mut t, idle);
    attack_with(
        &mut t,
        &[(scout, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(!tapped(&t, bears) && !tapped(&t, idle));
    assert!(tapped(&t, scout), "not the Scout itself");
    assert!(t.g.is_attacking(bears));
    next_step(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sphinx_additional_beginning_phase_then_the_ending_phase() {
    cr!("500.8", "505.1a");
    ruling!(
        "Sphinx of the Second Sun",
        "After the additional beginning phase, the game proceeds to the ending phase"
    );
    ruling!(
        "Sphinx of the Second Sun",
        "Even if you don't attack with any creatures during combat, you'll still get a postcombat main phase and Sphinx of the Second Sun's ability will still trigger."
    );
    supported("Sphinx of the Second Sun");
    let mut t = TestGame::new(2);
    t.g.turn.number = 3;
    t.battlefield(P0, "Sphinx of the Second Sun");
    let land = t.battlefield(P0, "Island");
    tap(&mut t, land);
    let hand = t.hand_size(P0);
    // No attackers are declared.
    let steps = steps_of_this_turn(&mut t);
    let after: Vec<Step> = steps
        .iter()
        .copied()
        .skip_while(|s| *s != Step::PostcombatMain)
        .collect();
    assert_eq!(
        after,
        vec![
            Step::PostcombatMain,
            Step::Untap,
            Step::Upkeep,
            Step::Draw,
            Step::End,
            Step::Cleanup
        ]
    );
    assert_eq!(
        t.hand_size(P0),
        hand + 1,
        "drew in the additional draw step"
    );
}

/// Bond of Revival returns Grizzly Bears with haste "until your next turn" in P0's
/// precombat main phase; the turn goes through an additional beginning phase.
fn until_your_next_turn_survives(t: &mut TestGame) {
    let bears = t.graveyard(P0, "Grizzly Bears");
    cast_and_resolve(t, P0, "Bond of Revival", &[obj(bears)]);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    next_step(t, Step::PostcombatMain);
    t.resolve_all();
    next_step(t, Step::Upkeep);
    assert!(
        t.obj_now(bears).has_keyword(KeywordKind::Haste),
        "the effect ended in the additional beginning phase"
    );
    assert_eq!(t.g.turn.active, P0);
}

#[test]
fn sphinx_additional_beginning_phase_is_in_the_same_turn() {
    cr!("500.8", "611.2a");
    ruling!(
        "Sphinx of the Second Sun",
        "Any effects that last \"until your next turn\" or similar won't expire just because you'll go through an additional beginning phase."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphinx of the Second Sun");
    until_your_next_turn_survives(&mut t);
}

#[test]
fn shadow_of_the_second_sun_additional_beginning_phase_is_in_the_same_turn() {
    cr!("500.8", "611.2a", "303.4");
    ruling!(
        "Shadow of the Second Sun",
        "Any effects that last \"until your next turn\" or similar won't expire just because they'll go through an additional beginning phase."
    );
    supported("Shadow of the Second Sun");
    let mut t = TestGame::new(2);
    let shadow = t.battlefield(P0, "Shadow of the Second Sun");
    t.g.objects[shadow.0 as usize].attached_to = Some(Entity::Player(P0));
    t.g.recompute();
    until_your_next_turn_survives(&mut t);
}

#[test]
fn paradox_haze_echo_triggers_in_the_first_upkeep_only() {
    cr!("702.30a", "500.9");
    ruling!(
        "Paradox Haze",
        "If echo triggers during the first upkeep, it won't trigger again during the second upkeep"
    );
    supported("Paradox Haze");
    let mut t = TestGame::new(2);
    let haze = t.battlefield(P0, "Paradox Haze");
    t.g.objects[haze.0 as usize].attached_to = Some(Entity::Player(P0));
    t.lands(P0, "Mountain", 2);
    // Mogg War Marshal: "Echo {1}{R}".
    t.enter(P0, "Mogg War Marshal");
    t.resolve_all();
    untap_step_of(&mut t, P0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Echo"), 1, "{}", t.dump_log());
    t.resolve_all();
    nth_step(&mut t, Step::Upkeep, 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Echo"), 0);
}

#[test]
fn paradox_haze_targets_a_player_and_enters_attached_to_them() {
    cr!("303.4f", "303.4a");
    ruling!(
        "Paradox Haze",
        "Paradox Haze targets a player when it's cast, and it enters attached to that player."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let haze = t.hand(P0, "Paradox Haze");
    t.cast(P0, haze).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(haze).attached_to, Some(Entity::Player(P1)));
}

#[test]
fn natures_will_triggers_once_per_combat_damage_step() {
    cr!("510.2", "603.2c", "702.7b");
    ruling!(
        "Nature's Will",
        "The lands are tapped and untapped once each time combat damage is dealt"
    );
    supported("Nature's Will");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nature's Will");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let striker = t.battlefield(P0, "Fencing Ace");
    let mine = t.battlefield(P0, "Forest");
    tap(&mut t, mine);
    t.battlefield(P1, "Island");
    attack_with(
        &mut t,
        &[
            (bears, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
            (striker, Entity::Player(P1)),
        ],
    );
    // Fencing Ace (double strike) deals damage in both steps: one trigger each time, not
    // one per creature.
    next_step(&mut t, Step::FirstStrikeDamage);
    assert_eq!(
        triggers_on_stack(&t, "tap all lands"),
        1,
        "{:?} {}",
        stack_items(&t),
        t.dump_log()
    );
    t.resolve_all();
    assert!(!tapped(&t, mine));
    tap(&mut t, mine);
    next_step(&mut t, Step::CombatDamage);
    assert_eq!(
        triggers_on_stack(&t, "tap all lands"),
        1,
        "{:?} {}",
        stack_items(&t),
        t.dump_log()
    );
    t.resolve_all();
    assert!(!tapped(&t, mine));
}
