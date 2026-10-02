//! Rulings batch P189 — the wordings the batch's new condition patterns newly compile:
//! "you've committed a crime this turn" (CR 700.13) and "you've cast a [kind of] spell
//! this turn", in static, cost, trigger, activation and effect conditions.

use crate::r_p076_common::mana;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::can_attack;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Shock targeting P1 (a crime, CR 700.13) and it resolves.
fn commit_crime(t: &mut TestGame) {
    mana(t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
}

/// P0 casts Shock targeting P0's own `target` (not a crime) and it resolves.
fn shock_own(t: &mut TestGame, target: ObjectId) {
    mana(t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(target).go();
    t.resolve_all();
}

/// Whether `id` could attack in P0's beginning of combat step (moved to, keeping the
/// turn's history).
fn could_attack(t: &mut TestGame, id: ObjectId) -> bool {
    t.set_step(P0, Step::BeginningOfCombat);
    let r = can_attack(t, id);
    t.set_step(P0, Step::PrecombatMain);
    r
}

/// P0 casts Opt (an instant spell) and it resolves.
fn cast_instant(t: &mut TestGame) {
    mana(t, P0, ManaType::U, 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
}

// ---------------------------------------------------------------------------
// "you've committed a crime this turn"
// ---------------------------------------------------------------------------

#[test]
fn slickshot_vault_buster_gets_bonus_after_a_crime() {
    cr!("700.13", "611.3a");
    supported("Slickshot Vault-Buster");
    let mut t = TestGame::new(2);
    let sv = t.battlefield(P0, "Slickshot Vault-Buster");
    let own = t.battlefield(P0, "Craw Wurm");
    // Targeting your own creature isn't a crime.
    shock_own(&mut t, own);
    assert_eq!(t.pt(sv), (1, 4));
    commit_crime(&mut t);
    assert_eq!(t.pt(sv), (3, 4));
    // An opponent's crime doesn't count.
    let mut t = TestGame::new(2);
    let sv = t.battlefield(P0, "Slickshot Vault-Buster");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(sv), (1, 4));
}

#[test]
fn omenport_vigilante_double_strike_after_a_crime() {
    cr!("700.13", "613.1f");
    supported("Omenport Vigilante");
    let mut t = TestGame::new(2);
    let ov = t.battlefield(P0, "Omenport Vigilante");
    t.g.recompute();
    assert!(!t.obj(ov).has_keyword(KeywordKind::DoubleStrike));
    commit_crime(&mut t);
    assert!(t.obj(ov).has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn nimble_brigand_unblockable_after_a_crime() {
    cr!("700.13", "509.1b");
    supported("Nimble Brigand");
    let mut t = TestGame::new(2);
    let nb = t.battlefield(P0, "Nimble Brigand");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(nb, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(bears, nb)]));
    let mut t = TestGame::new(2);
    let nb = t.battlefield(P0, "Nimble Brigand");
    let bears = t.battlefield(P1, "Grizzly Bears");
    commit_crime(&mut t);
    attack_with(&mut t, &[(nb, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, nb)]));
}

#[test]
fn seize_the_secrets_costs_less_after_a_crime() {
    cr!("700.13", "601.2f");
    supported("Seize the Secrets");
    let mut t = TestGame::new(2);
    let seize = t.hand(P0, "Seize the Secrets");
    mana(&mut t, P0, ManaType::U, 2);
    assert!(t.cast(P0, seize).try_go().is_err(), "{{2}}{{U}} with two mana");
    t.clear_answers();
    commit_crime(&mut t);
    let left = t.g.player(P0).mana_pool.total() as u32;
    mana(&mut t, P0, ManaType::U, 2 - left);
    let hand = t.hand_size(P0);
    t.cast(P0, seize).try_go().expect("{1}{U} after a crime");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn oko_the_ringleader_discards_one_after_a_crime() {
    cr!("700.13", "608.2c");
    supported("Oko, the Ringleader");
    for crime in [false, true] {
        let mut t = TestGame::new(2);
        let oko = t.battlefield(P0, "Oko, the Ringleader");
        let bears = t.battlefield(P0, "Craw Wurm");
        for _ in 0..3 {
            t.hand(P0, "Island");
        }
        if crime {
            commit_crime(&mut t);
        } else {
            shock_own(&mut t, bears);
        }
        let hand = t.hand_size(P0);
        let gy = t.g.player(P0).graveyard.len();
        t.activate(P0, oko, 0, &[]).expect("+1");
        t.resolve_all();
        let discarded = if crime { 1 } else { 2 };
        assert_eq!(t.hand_size(P0), hand + 2 - discarded, "crime: {crime}");
        assert_eq!(t.g.player(P0).graveyard.len(), gy + discarded);
    }
}

#[test]
fn servant_of_the_stinger_intervening_crime_check() {
    cr!("700.13", "603.4");
    supported("Servant of the Stinger");
    for crime in [false, true] {
        let mut t = TestGame::new(2);
        let sv = t.battlefield(P0, "Servant of the Stinger");
        if crime {
            commit_crime(&mut t);
        }
        t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
        t.attack(&[(sv, Entity::Player(P1))], &[]);
        t.resolve_all();
        // Sacrificed (and a card searched for) only after a crime.
        assert_eq!(t.on_battlefield(sv), !crime, "crime: {crime}");
        assert_eq!(t.in_graveyard(P0, "Servant of the Stinger"), crime);
    }
}

// ---------------------------------------------------------------------------
// "you've cast a [kind of] spell this turn"
// ---------------------------------------------------------------------------

#[test]
fn leapfrog_flies_after_an_instant() {
    cr!("611.3a", "613.1f");
    supported("Leapfrog");
    let mut t = TestGame::new(2);
    let lf = t.battlefield(P0, "Leapfrog");
    t.g.recompute();
    assert!(!t.obj(lf).has_keyword(KeywordKind::Flying));
    // A creature spell doesn't count.
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    assert!(!t.obj(lf).has_keyword(KeywordKind::Flying));
    cast_instant(&mut t);
    assert!(t.obj(lf).has_keyword(KeywordKind::Flying));
}

#[test]
fn haunting_figment_unblockable_after_an_instant() {
    cr!("509.1b");
    supported("Haunting Figment");
    let mut t = TestGame::new(2);
    let hf = t.battlefield(P0, "Haunting Figment");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_instant(&mut t);
    attack_with(&mut t, &[(hf, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, hf)]));
    // An opponent's instant doesn't count.
    let mut t = TestGame::new(2);
    let hf = t.battlefield(P0, "Haunting Figment");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P1, ManaType::U, 1);
    let opt = t.hand(P1, "Opt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, opt).go();
    t.resolve_all();
    attack_with(&mut t, &[(hf, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(bears, hf)]));
}

#[test]
fn piston_fist_cyclops_attacks_after_an_instant() {
    cr!("702.3b", "508.1c");
    supported("Piston-Fist Cyclops");
    let mut t = TestGame::new(2);
    let pc = t.battlefield(P0, "Piston-Fist Cyclops");
    assert!(!could_attack(&mut t, pc));
    cast_instant(&mut t);
    assert!(could_attack(&mut t, pc));
}

#[test]
fn goblin_cohort_attacks_after_a_creature_spell() {
    cr!("508.1c");
    supported("Goblin Cohort");
    supported("Mogg Conscripts");
    for name in ["Goblin Cohort", "Mogg Conscripts"] {
        let mut t = TestGame::new(2);
        let gc = t.battlefield(P0, name);
        cast_instant(&mut t);
        assert!(!could_attack(&mut t, gc), "{name}");
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
        assert!(could_attack(&mut t, gc), "{name}");
    }
}

#[test]
fn rhino_draws_after_a_spell_with_mana_value_four() {
    cr!("603.4", "202.3");
    supported("Rhino, Barreling Brute");
    for (spell, draws) in [("Grizzly Bears", 0), ("Hill Giant", 1)] {
        let mut t = TestGame::new(2);
        let rhino = t.battlefield(P0, "Rhino, Barreling Brute");
        mana(&mut t, P0, ManaType::C, 3);
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::R, 1);
        let c = t.hand(P0, spell);
        t.cast(P0, c).go();
        t.resolve_all();
        let hand = t.hand_size(P0);
        attack_with(&mut t, &[(rhino, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{spell}");
    }
}

#[test]
fn potioners_trove_activates_only_after_an_instant() {
    cr!("602.5");
    supported("Potioner's Trove");
    let mut t = TestGame::new(2);
    let trove = t.battlefield(P0, "Potioner's Trove");
    assert!(t.activate(P0, trove, 1, &[]).is_err());
    cast_instant(&mut t);
    t.activate(P0, trove, 1, &[]).expect("after an instant spell");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn fortune_tellers_talent_plays_from_the_top_after_a_spell() {
    cr!("716.2a", "305.1");
    supported("Fortune Teller's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Fortune Teller's Talent");
    t.g.objects[talent.0 as usize].class_level = 2;
    t.g.recompute();
    let island = t.library_top(P0, "Island");
    assert!(t.play_land(P0, island).is_err(), "no spell cast yet");
    // Any spell: Shock.
    commit_crime(&mut t);
    t.play_land(P0, island).expect("after casting a spell");
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
}
