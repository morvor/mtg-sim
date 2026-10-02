//! Rulings batch P223 — scry from cast triggers and counterspells, and Case of the
//! Shifting Visage's solved abilities (CR 603.3, 701.22, 702.169, 719).

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s17_common::{become_copy, token_copy};
use mtg_engine::cases;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn giants(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.library_top(p, "Hill Giant");
    }
}

#[test]
fn cast_triggered_scry_resolves_before_the_spell() {
    cr!("603.3", "405.5", "701.22a");
    ruling!("Jace's Sanctum", "Jace’s Sanctum’s scry ability will resolve before the instant or sorcery spell that caused it to trigger.");
    ruling!("Prescient Chimera", "The triggered ability will resolve and you'll scry before the instant or sorcery spell resolves.");
    for name in ["Jace's Sanctum", "Prescient Chimera"] {
        supported(name);
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        t.battlefield(P0, name);
        let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}");
        let from = t.asked().len();
        t.resolve();
        // Scried, with the Bolt still on the stack.
        assert_eq!(scry_sizes(&t, P0, from), vec![1], "{name}");
        assert_eq!(t.stack_len(), 1);
        assert_eq!(t.life(P1), 20);
        t.resolve_all();
        assert_eq!(t.life(P1), 17);
    }
}

#[test]
fn eyes_of_the_watcher_scries_at_the_end_of_the_resolution_if_you_pay() {
    cr!("608.2c", "118.12", "701.22a");
    ruling!("Eyes of the Watcher", "You do the scrying at the end of the ability’s resolution (assuming you pay {1}).");
    supported("Eyes of the Watcher");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        t.battlefield(P0, "Eyes of the Watcher");
        let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
        t.lands(P0, "Wastes", 1);
        t.cast(P0, bolt).target(P1).go();
        t.answer_yes(P0, pay);
        let from = t.asked().len();
        t.resolve();
        let asked = t.asked();
        let ask = asked[from..]
            .iter()
            .position(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .expect("asked whether to pay");
        let scry = first_scry(&t, from);
        if pay {
            assert!(scry.expect("scried") > from + ask);
            assert_eq!(scry_sizes(&t, P0, from), vec![2]);
        } else {
            assert!(scry.is_none());
        }
    }
}

#[test]
fn psychic_impetus_the_auras_controller_scries() {
    cr!("603.3a", "701.22a");
    ruling!("Psychic Impetus", "Psychic Impetus causes you to scry 2, not the attacking creature's controller.");
    supported("Psychic Impetus");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    giants(&mut t, P1, 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = in_hand_with_mana(&mut t, P0, "Psychic Impetus");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    t.set_step(P1, Step::BeginningOfCombat);
    let from = t.asked().len();
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
    assert!(scry_sizes(&t, P1, from).is_empty());
}

/// P1 casts Lightning Bolt at P0 with three extra lands available; P0 responds with
/// `counter` (X = `x` for Condescend) with `in_gy` instant cards in their graveyard. P1
/// pays if `pays`. Returns (the game, P0's scry sizes).
fn counter_bolt(counter: &str, x: Option<i64>, in_gy: usize, pays: bool) -> (TestGame, Vec<usize>) {
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    for _ in 0..in_gy {
        t.graveyard(P0, "Shock");
    }
    t.set_step(P1, Step::PrecombatMain);
    let bolt = in_hand_with_mana(&mut t, P1, "Lightning Bolt");
    t.lands(P1, "Wastes", 3);
    let spell = t.cast(P1, bolt).target(P0).go();
    let c = in_hand_with_mana(&mut t, P0, counter);
    if let Some(x) = x {
        t.lands(P0, "Wastes", x as usize);
    }
    let mut b = t.cast(P0, c).target(spell);
    if let Some(x) = x {
        b = b.x(x);
    }
    b.go();
    t.answer_yes(P1, pays);
    let from = t.asked().len();
    t.resolve_all();
    let s = scry_sizes(&t, P0, from);
    (t, s)
}

#[test]
fn calculated_dismissal_scries_with_spell_mastery_even_if_the_controller_pays() {
    cr!("608.2c", "701.22a");
    ruling!("Calculated Dismissal", "If the spell mastery ability applies, you’ll scry 2 even if the controller of the spell pays {3}.");
    supported("Calculated Dismissal");
    // Paid: not countered, but scry 2.
    let (t, s) = counter_bolt("Calculated Dismissal", None, 2, true);
    assert_eq!(t.life(P0), 17);
    assert_eq!(s, vec![2]);
    // Not paid: countered, scry 2.
    let (t, s) = counter_bolt("Calculated Dismissal", None, 2, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(s, vec![2]);
    // No spell mastery: no scry.
    let (_, s) = counter_bolt("Calculated Dismissal", None, 1, true);
    assert!(s.is_empty());
}

#[test]
fn calculated_dismissal_countering_your_own_spell_can_turn_on_spell_mastery() {
    cr!("608.2c", "608.2h", "701.6a");
    ruling!("Calculated Dismissal", "In one unusual situation, you can cast Calculated Dismissal targeting an instant or sorcery spell you control while there is one instant or sorcery card in your graveyard. In this situation, if you decline to pay {3}, the spell will be countered and put into your graveyard. The spell mastery ability will then apply and you’ll scry 2.");
    supported("Calculated Dismissal");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    t.graveyard(P0, "Shock");
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    let cd = in_hand_with_mana(&mut t, P0, "Calculated Dismissal");
    t.cast(P0, cd).target(spell).go();
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
}

#[test]
fn condescend_scries_even_if_the_controller_pays() {
    cr!("608.2c", "701.22a");
    ruling!("Condescend", "You scry 2 even if the spell’s controller pays {X}.");
    supported("Condescend");
    let (t, s) = counter_bolt("Condescend", Some(2), 0, true);
    assert_eq!(t.life(P0), 17);
    assert_eq!(s, vec![2]);
    let (t, s) = counter_bolt("Condescend", Some(2), 0, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(s, vec![2]);
}

// --- Case of the Shifting Visage ----------------------------------------------------
// "At the beginning of your upkeep, surveil 1. / To solve — There are fifteen or more
// cards in your graveyard. / Solved — Whenever you cast a nonlegendary creature spell,
// copy that spell. (The copy becomes a token.)"

fn visage(t: &mut TestGame) -> ObjectId {
    supported("Case of the Shifting Visage");
    t.battlefield(P0, "Case of the Shifting Visage")
}

fn fill_graveyard(t: &mut TestGame, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(P0, "Shock")).collect()
}

#[test]
fn to_solve_checks_its_condition_at_the_end_step_and_on_resolution() {
    cr!("719.3a", "603.4");
    ruling!("Case of the Shifting Visage", "“To Solve — [condition]” means “At the beginning of your end step, if [condition] and this Case is not solved, it becomes solved.”");
    ruling!("Case of the Shifting Visage", "“To solve” abilities will check for their condition twice: once when the ability would trigger, and once when it resolves. If the condition isn’t true at the beginning of your end step, the ability won’t trigger at all. If the condition isn’t true when the ability resolves, the Case won’t become solved.");
    // Fifteen cards: solved at the beginning of the end step.
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    fill_graveyard(&mut t, 15);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(cases::is_solved(&t.g, case));
    // Not in the opponent's end step, even with fifteen cards.
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    fill_graveyard(&mut t, 15);
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(!cases::is_solved(&t.g, case));
    // Fourteen cards: it doesn't trigger.
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    fill_graveyard(&mut t, 14);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(!cases::is_solved(&t.g, case));
    // Fifteen as it triggers, fourteen as it resolves: not solved.
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    let cards = fill_graveyard(&mut t, 15);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, cards[0], mtg_engine::object::Zone::Exile);
    t.resolve_all();
    assert!(!cases::is_solved(&t.g, case));
    // Already solved: it doesn't trigger again.
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    cases::solve(&mut t.g, case);
    fill_graveyard(&mut t, 15);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_solved_triggered_ability_triggers_only_while_solved_and_resolves_first() {
    cr!("702.169c", "707.10", "603.3");
    ruling!("Case of the Shifting Visage", "“Solved — [Triggered ability]” means “[Triggered ability]. This ability triggers only if this Case is solved.”");
    ruling!("Case of the Shifting Visage", "The triggered ability Case of the Shifting Visage has when it’s solved and the copy it creates will resolve before the spell that caused the ability to trigger.");
    // Unsolved: no copy.
    let mut t = TestGame::new(2);
    visage(&mut t);
    let bears = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Solved: the trigger, then the copy, resolve before the original.
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    cases::solve(&mut t.g, case);
    t.g.recompute();
    let bears = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The copy is on the stack above the original.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(*t.g.stack.first().unwrap(), spell);
    t.resolve();
    let on_bf = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on_bf.len(), 1);
    assert!(t.obj_now(on_bf[0]).is_token());
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn cases_keep_their_other_abilities_when_solved() {
    cr!("719.3c");
    ruling!("Case of the Shifting Visage", "Cases don’t lose their other abilities when they become solved.");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let case = visage(&mut t);
    cases::solve(&mut t.g, case);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let from = t.asked().len();
    t.resolve_all();
    let surveils = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Surveil { .. }))
        .count();
    assert_eq!(surveils, 1);
}

#[test]
fn being_solved_isnt_copiable() {
    cr!("719.3b", "707.2");
    ruling!("Case of the Shifting Visage", "Being solved is not part of a permanent’s copiable values. A permanent that becomes a copy of a solved Case is not solved. A solved Case that somehow becomes a copy of a different Case stays solved.");
    let mut t = TestGame::new(2);
    let case = visage(&mut t);
    cases::solve(&mut t.g, case);
    // A token copy of the solved Case isn't solved.
    let copies = token_copy(&mut t, P0, case);
    assert_eq!(copies.len(), 1);
    assert!(!cases::is_solved(&t.g, copies[0]));
    // The solved Case becomes a copy of another Case: still solved, and that Case's
    // solved ability works.
    let other = t.battlefield(P1, "Case of the Gateway Express");
    become_copy(&mut t, case, other);
    assert_eq!(t.obj_now(case).chars.name, "Case of the Gateway Express");
    assert!(cases::is_solved(&t.g, case));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn solved_static_and_activated_abilities_work_only_while_solved() {
    cr!("702.169b", "702.169d");
    ruling!("Case of the Shifting Visage", "“Solved — [static ability]” means “As long as this Case is solved, [static ability].”");
    ruling!("Case of the Shifting Visage", "The meaning of “solved” differs based on what type of ability follows it. “Solved — [activated ability]” means “[Activated ability]. Activate only if this Case is solved.”");
    // Static: Case of the Gateway Express, "Solved — Creatures you control get +1/+0."
    supported("Case of the Gateway Express");
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Gateway Express");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    cases::solve(&mut t.g, case);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 2));
    // Activated: Case of the Stashed Skeleton, "Solved — {1}{B}, Sacrifice this Case:
    // Search your library for a card, put it into your hand, then shuffle. Activate only
    // as a sorcery."
    supported("Case of the Stashed Skeleton");
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Stashed Skeleton");
    t.lands(P0, "Swamp", 2);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, case));
    cases::solve(&mut t.g, case);
    t.g.recompute();
    assert!(crate::r_s02_common::can_activate(&mut t, P0, case));
}
