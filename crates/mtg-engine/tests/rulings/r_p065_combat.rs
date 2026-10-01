//! Rulings batch P065 — combat with creatures that gain or lose abilities: attack
//! requirements (CR 508.1d), costs to attack (508.1h), block restrictions checked only as
//! blockers are declared (509.1b), intimidate and current colors (702.13), hexproof the
//! turn a creature enters, first strike gained between the combat damage steps (510.4),
//! and redundant instances of keywords (702.2f etc.).

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s09_common::{declare, legal_attack, to_combat};
use crate::r_s10_common::{attacking, blocking};
use crate::r_s21_common::legal_blocks;
use crate::r_s26_common::modify_until_eot;
use crate::r_s30_common::damage_events;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// "Attacks each combat if able"
// ---------------------------------------------------------------------------------------

/// A creature that attacks each combat if able need not attack when it's tapped, or when
/// attacking would cost something (Ghostly Prison); otherwise not attacking is illegal.
fn attacks_if_able_but_not_when_tapped_or_costly(name: &str) {
    supported(name);
    supported("Ghostly Prison");
    // Untapped, no cost: it must attack.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]), "{name}: must attack");
    assert!(legal_attack(&mut t, &[(c, Entity::Player(P1))]));
    // Tapped: it doesn't attack.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    t.g.tap(c);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]), "{name}: tapped, needn't attack");
    let attacks = declare(&mut t, P0, &[]);
    assert!(attacks.is_empty());
    // A cost to attack: its controller isn't forced to pay it.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Plains", 4);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]), "{name}: a cost, needn't attack");
    let attacks = declare(&mut t, P0, &[]);
    assert!(attacks.is_empty());
    assert!(!attacking(&t, c));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn mishras_juggernaut_doesnt_attack_when_it_cant_or_it_costs_something() {
    cr!("508.1d", "508.1h", "508.1a");
    ruling!(
        "Mishra's Juggernaut",
        "If Mishra's Juggernaut can't attack for any reason (such as being tapped), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost"
    );
    attacks_if_able_but_not_when_tapped_or_costly("Mishra's Juggernaut");
}

#[test]
fn sprinting_warbrute_doesnt_attack_when_it_cant_or_it_costs_something() {
    cr!("508.1d", "508.1h", "302.6");
    ruling!(
        "Sprinting Warbrute",
        "If, during your declare attackers step, Sprinting Warbrute is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under your control continuously since the turn began (and doesn’t have haste), then it doesn’t attack."
    );
    attacks_if_able_but_not_when_tapped_or_costly("Sprinting Warbrute");
    // Summoning sick without haste: it can't attack, so it doesn't.
    let mut t = TestGame::new(2);
    let c = t.battlefield_sick(P0, "Sprinting Warbrute");
    to_combat(&mut t, P0);
    assert!(!can_attack(&mut t, c));
    assert!(legal_attack(&mut t, &[]));
    // An effect that says it can't attack (Pacifism).
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Sprinting Warbrute");
    crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", c);
    to_combat(&mut t, P0);
    assert!(!can_attack(&mut t, c));
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn sprinting_warbrute_chooses_which_player_or_planeswalker_it_attacks() {
    cr!("508.1d", "508.1b");
    ruling!(
        "Sprinting Warbrute",
        "You still choose which player or planeswalker Sprinting Warbrute attacks."
    );
    let mut t = TestGame::new(3);
    let c = t.battlefield(P0, "Sprinting Warbrute");
    let jace = t.battlefield(P1, "Jace Beleren");
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(c, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[(c, Entity::Player(P2))]));
    assert!(legal_attack(&mut t, &[(c, Entity::Object(jace))]));
    let attacks = declare(&mut t, P0, &[(c, Entity::Object(jace))]);
    assert_eq!(attacks, vec![(c, Entity::Object(jace))]);
}

// ---------------------------------------------------------------------------------------
// Blocking
// ---------------------------------------------------------------------------------------

#[test]
fn zurgo_keeps_blocking_when_the_attackers_power_rises() {
    cr!("509.1b", "506.4");
    ruling!(
        "Zurgo Bellstriker",
        "Once Zurgo Bellstriker has legally blocked a creature, raising that creature's power to 2 or greater won't undo the block."
    );
    supported("Zurgo Bellstriker");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let zurgo = t.battlefield(P0, "Zurgo Bellstriker");
    to_combat(&mut t, P1);
    declare(&mut t, P1, &[(bears, Entity::Player(P0)), (elves, Entity::Player(P0))]);
    // Zurgo can't block the 2-power Bears, but can block the 1-power Elves.
    assert!(!legal_blocks(&mut t, P0, &[(zurgo, bears)]));
    assert!(legal_blocks(&mut t, P0, &[(zurgo, elves)]));
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let zurgo = t.battlefield(P0, "Zurgo Bellstriker");
    to_combat(&mut t, P1);
    to_blockers(&mut t, &[(elves, Entity::Player(P0))], &[(zurgo, elves)]);
    assert!(blocking(&t, zurgo));
    modify_until_eot(
        &mut t,
        elves,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    assert_eq!(t.pt(elves).0, 3);
    assert!(blocking(&t, zurgo), "the block isn't undone");
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20, "the Elves stayed blocked");
    assert!(!t.on_battlefield(zurgo), "Zurgo took 3 damage");
}

#[test]
fn guul_draz_vampire_intimidate_uses_its_current_colors() {
    cr!("702.13b", "105.2");
    ruling!(
        "Guul Draz Vampire",
        "Intimidate looks at the current colors of a creature that has it."
    );
    supported("Guul Draz Vampire");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 10;
    let vamp = t.battlefield(P0, "Guul Draz Vampire");
    let black = t.battlefield(P1, "Walking Corpse");
    let white = t.battlefield(P1, "Savannah Lions");
    let artifact = t.battlefield(P1, "Ornithopter");
    t.g.recompute();
    assert!(has_kw(&t, vamp, KeywordKind::Intimidate));
    declare(&mut t, P0, &[(vamp, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(black, vamp)]));
    assert!(legal_blocks(&mut t, P1, &[(artifact, vamp)]));
    assert!(!legal_blocks(&mut t, P1, &[(white, vamp)]));
    // Turned white: white creatures can block it, black ones can't.
    modify_until_eot(
        &mut t,
        vamp,
        vec![Modification::SetColors(ColorSet::single(Color::White))],
    );
    assert!(legal_blocks(&mut t, P1, &[(white, vamp)]));
    assert!(legal_blocks(&mut t, P1, &[(artifact, vamp)]));
    assert!(!legal_blocks(&mut t, P1, &[(black, vamp)]));
}

// ---------------------------------------------------------------------------------------
// Hexproof the turn it enters
// ---------------------------------------------------------------------------------------

#[test]
fn drownyard_behemoth_has_hexproof_all_turn_it_enters() {
    cr!("702.11b", "115.4");
    ruling!(
        "Drownyard Behemoth",
        "There is no moment during the turn Drownyard Behemoth enters the battlefield in which it doesn't have hexproof."
    );
    supported("Drownyard Behemoth");
    let mut t = TestGame::new(2);
    let b = t.enter(P0, "Drownyard Behemoth");
    t.settle();
    assert!(has_kw(&t, b, KeywordKind::Hexproof));
    // An opponent can't target it this turn, even in the end step.
    t.advance_to(P0, Step::End);
    assert!(has_kw(&t, b, KeywordKind::Hexproof));
    let targets = crate::r_s04_common::spell_targets(&mut t, P1, "Lightning Bolt");
    assert!(!targets.contains(&Entity::Object(b)));
    // In the next turn's upkeep, it no longer has hexproof.
    t.advance_to(P1, Step::Upkeep);
    assert!(!has_kw(&t, b, KeywordKind::Hexproof));
    let targets = crate::r_s04_common::spell_targets(&mut t, P1, "Lightning Bolt");
    assert!(targets.contains(&Entity::Object(b)));
}

// ---------------------------------------------------------------------------------------
// Altac Bloodseeker: first strike gained between the combat damage steps
// ---------------------------------------------------------------------------------------

#[test]
fn altac_bloodseeker_gaining_first_strike_in_the_first_step_deals_damage_in_the_second() {
    cr!("510.4", "702.7c");
    ruling!(
        "Altac Bloodseeker",
        "Altac Bloodseeker may gain first strike during the first one. If this happens, it will assign combat damage in the second combat damage step"
    );
    supported("Altac Bloodseeker");
    let mut t = TestGame::new(2);
    let altac = t.battlefield(P0, "Altac Bloodseeker");
    let knight = t.battlefield(P0, "White Knight");
    let elves = t.battlefield(P1, "Llanowar Elves");
    to_combat(&mut t, P0);
    to_blockers(
        &mut t,
        &[(altac, Entity::Player(P1)), (knight, Entity::Player(P1))],
        &[(elves, knight)],
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(elves));
    assert!(has_kw(&t, altac, KeywordKind::FirstStrike));
    assert_eq!(t.pt(altac).0, 4);
    // Altac dealt its damage once, as a 4-power creature, in the second step.
    let altac_hits: Vec<_> = damage_events(&t)
        .into_iter()
        .filter(|e| e.0 == altac)
        .collect();
    assert_eq!(altac_hits, vec![(altac, Entity::Player(P1), 4, true)]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn altac_bloodseeker_bonuses_are_cumulative_but_first_strike_is_redundant() {
    cr!("702.7d", "702.10d", "613.4c");
    ruling!(
        "Altac Bloodseeker",
        "If the ability triggers multiple times in a single turn, the power bonuses are cumulative, but the additional instances of first strike and haste are redundant."
    );
    let mut t = TestGame::new(2);
    let altac = t.battlefield(P0, "Altac Bloodseeker");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    destroy(&mut t, a);
    t.resolve_all();
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.pt(altac), (6, 1));
    assert!(has_kw(&t, altac, KeywordKind::FirstStrike));
    // It deals first-strike damage once (not twice, as double strike would).
    let blocker = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    to_blockers(&mut t, &[(altac, Entity::Player(P1))], &[(blocker, altac)]);
    t.advance_to(P0, Step::EndOfCombat);
    let hits = damage_events(&t)
        .into_iter()
        .filter(|e| e.0 == altac)
        .count();
    assert_eq!(hits, 1);
    assert!(!t.on_battlefield(blocker));
    assert!(t.on_battlefield(altac), "first strike killed the blocker first");
}

// ---------------------------------------------------------------------------------------
// Endling: negative power
// ---------------------------------------------------------------------------------------

#[test]
fn endling_can_have_negative_power_which_is_used_when_modified() {
    cr!("107.1b", "613.4c");
    ruling!(
        "Endling",
        "If Endling's last ability gives it -1/+1 enough times, it may have negative power. Use that negative value if its power is further modified."
    );
    supported("Endling");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Endling");
    t.lands(P0, "Swamp", 7);
    for _ in 0..5 {
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        activate_containing(&mut t, P0, e, "{1}:").unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(e), (-2, 8));
    for _ in 0..2 {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        activate_containing(&mut t, P0, e, "{1}:").unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(e), (0, 6));
}

// ---------------------------------------------------------------------------------------
// Redundant keywords
// ---------------------------------------------------------------------------------------

/// `c` attacks P1 unblocked; returns P0's life gained.
fn attack_unblocked_life_gain(t: &mut TestGame, c: ObjectId) -> i32 {
    let before = t.life(P0);
    to_combat(t, P0);
    declare(t, P0, &[(c, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.life(P0) - before
}

#[test]
fn shifting_ceratops_multiple_instances_of_trample_are_redundant() {
    cr!("702.19b", "702.17c", "702.10d");
    ruling!(
        "Shifting Ceratops",
        "Multiple instances of reach, trample, and/or haste on the same creature are redundant."
    );
    supported("Shifting Ceratops");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Shifting Ceratops");
    t.lands(P0, "Forest", 6);
    for choice in [1, 1, 0, 0, 2, 2] {
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        activate_containing(&mut t, P0, c, "{G}").unwrap();
        t.resolve_all();
    }
    assert!(has_kw(&t, c, KeywordKind::Trample));
    assert!(has_kw(&t, c, KeywordKind::Reach));
    assert!(has_kw(&t, c, KeywordKind::Haste));
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    to_blockers(&mut t, &[(c, Entity::Player(P1))], &[(bears, c)]);
    t.advance_to(P0, Step::EndOfCombat);
    // Two instances of trample assign excess damage just as one: 5 - 2 = 3.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn resolute_rider_multiple_instances_of_lifelink_are_redundant() {
    cr!("702.15f", "702.12c");
    ruling!(
        "Resolute Rider",
        "Multiple instances of lifelink or indestructible on the same creature are redundant."
    );
    supported("Resolute Rider");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Resolute Rider");
    t.lands(P0, "Plains", 10);
    for _ in 0..2 {
        activate_containing(&mut t, P0, c, "lifelink").unwrap();
        t.resolve_all();
        activate_containing(&mut t, P0, c, "indestructible").unwrap();
        t.resolve_all();
    }
    assert!(has_kw(&t, c, KeywordKind::Indestructible));
    destroy(&mut t, c);
    assert!(t.on_battlefield(c));
    assert_eq!(attack_unblocked_life_gain(&mut t, c), 4);
}

#[test]
fn shambling_vent_activated_twice_still_gains_life_once() {
    cr!("702.15f", "613.1d");
    ruling!(
        "Shambling Vent",
        "Activating the last ability more than once won't cause you to gain additional life if Shambling Vent deals damage."
    );
    supported("Shambling Vent");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Shambling Vent");
    t.lands(P0, "Scrubland", 6);
    for _ in 0..2 {
        activate_containing(&mut t, P0, v, "becomes").unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(v), (2, 3));
    assert!(has_kw(&t, v, KeywordKind::Lifelink));
    assert_eq!(attack_unblocked_life_gain(&mut t, v), 2);
}

#[test]
fn stonehorn_chanter_activated_twice_still_gains_life_once() {
    cr!("702.15f", "702.20c");
    ruling!(
        "Stonehorn Chanter",
        "Multiple instances of vigilance and lifelink are redundant."
    );
    supported("Stonehorn Chanter");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Stonehorn Chanter");
    t.lands(P0, "Plains", 12);
    for _ in 0..2 {
        activate_containing(&mut t, P0, c, "vigilance").unwrap();
        t.resolve_all();
    }
    assert_eq!(attack_unblocked_life_gain(&mut t, c), 4);
    assert!(!t.obj_now(c).tapped, "vigilance");
}

// ---------------------------------------------------------------------------------------
// "Tap it" abilities can be activated while tapped
// ---------------------------------------------------------------------------------------

/// Activates the indestructible ability of the tapped creature (paying its cost), and
/// checks that it gained indestructible and survives "destroy".
fn tapped_activation(name: &str, needle: &str, land: &str, setup: fn(&mut TestGame)) {
    supported(name);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    t.lands(P0, land, 8);
    setup(&mut t);
    t.g.tap(c);
    activate_containing(&mut t, P0, c, needle).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    t.resolve_all();
    assert!(has_kw(&t, c, KeywordKind::Indestructible), "{name}");
    assert!(t.obj_now(c).tapped);
    destroy(&mut t, c);
    assert!(t.on_battlefield(c), "{name}");
    // Again, with indestructible already, and nothing about to destroy it.
    setup(&mut t);
    activate_containing(&mut t, P0, c, needle).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    t.resolve_all();
    assert!(has_kw(&t, c, KeywordKind::Indestructible));
}

fn a_card_in_hand(t: &mut TestGame) {
    t.hand(P0, "Grizzly Bears");
}

fn another_creature(t: &mut TestGame) {
    t.battlefield(P0, "Grizzly Bears");
}

#[test]
fn drudge_sentinel_can_be_activated_while_tapped() {
    cr!("602.5", "701.26a");
    ruling!(
        "Drudge Sentinel",
        "You can activate Drudge Sentinel’s ability even if it’s already tapped. It will still gain indestructible."
    );
    tapped_activation("Drudge Sentinel", "{3}", "Swamp", |_| {});
}

#[test]
fn iron_shield_elf_can_be_activated_while_tapped() {
    cr!("602.5", "701.26a");
    ruling!(
        "Iron-Shield Elf",
        "You can activate Iron-Shield Elf's ability even if Iron-Shield Elf is already tapped."
    );
    tapped_activation("Iron-Shield Elf", "Discard", "Swamp", a_card_in_hand);
}

#[test]
fn seasoned_hallowblade_can_be_activated_while_tapped_or_indestructible() {
    cr!("602.5", "701.26a");
    ruling!(
        "Seasoned Hallowblade",
        "You can activate Seasoned Hallowblade's ability even if nothing's about to destroy it, even if it's already tapped, and even if it already has indestructible."
    );
    tapped_activation("Seasoned Hallowblade", "Discard", "Plains", a_card_in_hand);
}

#[test]
fn unrooted_ancestor_can_be_activated_while_tapped() {
    cr!("602.5", "701.26a");
    ruling!(
        "Unrooted Ancestor",
        "You can activate Unrooted Ancestor’s last ability even if it is already tapped."
    );
    tapped_activation("Unrooted Ancestor", "Sacrifice", "Swamp", another_creature);
}

#[test]
fn necromancers_familiar_can_be_activated_while_tapped_or_indestructible() {
    cr!("602.5", "701.26a");
    ruling!(
        "Necromancer's Familiar",
        "You may activate the last ability even if Necromancer's Familiar is already tapped or it has indestructible."
    );
    tapped_activation("Necromancer's Familiar", "Discard", "Swamp", a_card_in_hand);
}

#[test]
fn butcher_of_the_horde_chooses_the_ability_on_resolution() {
    cr!("608.2d");
    ruling!(
        "Butcher of the Horde",
        "You choose which ability Butcher of the Horde gains when the activated ability resolves."
    );
    supported("Butcher of the Horde");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Butcher of the Horde");
    t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    activate_containing(&mut t, P0, b, "Sacrifice").unwrap();
    let option_asked = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
    };
    assert!(!option_asked(&t, from), "no choice as it's activated");
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert!(option_asked(&t, from), "the choice is made on resolution");
    assert!(has_kw(&t, b, KeywordKind::Lifelink));
    assert!(!has_kw(&t, b, KeywordKind::Vigilance));
}
