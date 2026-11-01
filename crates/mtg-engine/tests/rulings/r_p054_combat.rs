//! Rulings batch P054 — attack triggers of pump abilities (one trigger per attacking
//! creature, not for creatures put onto the battlefield attacking: CR 508.1m, 508.3a,
//! 508.4), "attacks alone" (CR 506.5), triggers that resolve in the declare attackers
//! step before blockers are declared, and blocking requirements with tapped creatures and
//! blocking costs (CR 509.1c-d).

use crate::r_p054_common::*;
use crate::r_s01_common::{attack_with, with_subtype};
use crate::r_s14_common::triggers_from;
use crate::r_s06_common::attach_new;
use crate::r_s21_common::legal_blocks;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn is_offer(d: &mtg_engine::decision::Decision) -> bool {
    use mtg_engine::decision::Decision;
    matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. })
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

// ---------------------------------------------------------------------------------------
// Arahbo, Roar of the World
// ---------------------------------------------------------------------------------------

#[test]
fn arahbo_triggers_for_each_attacking_cat_but_not_one_entering_attacking() {
    cr!("508.1m", "508.3a", "508.4");
    ruling!(
        "Arahbo, Roar of the World",
        "If a Cat you control enters the battlefield attacking, it won't cause Arahbo's last ability to trigger."
    );
    ruling!(
        "Arahbo, Roar of the World",
        "If multiple Cats attack, Arahbo's last ability triggers that many times. You may pay {1}{G}{W} each time one resolves."
    );
    ruling!(
        "Arahbo, Roar of the World",
        "While resolving Arahbo's last ability, you can't pay {1}{G}{W} multiple times to multiply the effect for one Cat."
    );
    // Brimaz attacks and creates a Cat token that's attacking: one Arahbo trigger.
    let mut t = TestGame::new(2);
    let arahbo = t.battlefield(P0, "Arahbo, Roar of the World");
    let brimaz = t.battlefield(P0, "Brimaz, King of Oreskos");
    attack_with(&mut t, &[(brimaz, pl(P1))]);
    assert_eq!(triggers_from(&t, arahbo), 1);
    pool(&mut t, P0);
    t.resolve_all();
    let cats = with_subtype(&t, P0, "Cat");
    assert_eq!(cats.len(), 3, "Arahbo, Brimaz and the token");
    // Arahbo triggered once (for Brimaz), not for the token: one payment offered.
    let offers = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && is_offer(d))
        .count();
    assert_eq!(offers, 1);
    let _ = arahbo;

    // Two Cats attack: two triggers, each paid once.
    let mut t = TestGame::new(2);
    let arahbo = t.battlefield(P0, "Arahbo, Roar of the World");
    let a = t.battlefield(P0, "Savannah Lions");
    let b = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    assert_eq!(triggers_from(&t, arahbo), 2);
    pool(&mut t, P0);
    let mana_before = t.g.players[0].mana_pool.total();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 3));
    assert_eq!(t.pt(b), (4, 3));
    assert!(t.obj_now(a).has_keyword(KeywordKind::Trample));
    assert_eq!(
        mana_before - t.g.players[0].mana_pool.total(),
        6,
        "paid once each"
    );
}

#[test]
fn arahbos_x_is_the_cats_power_as_it_resolves_and_never_negative() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Arahbo, Roar of the World",
        "The value of X is determined only as Arahbo's last ability resolves. If the attacking Cat's power is negative at that time, X is 0."
    );
    // Pumped in response: X counts the pump.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arahbo, Roar of the World");
    let lion = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(lion, pl(P1))]);
    cast_at(&mut t, P0, "Giant Growth", lion);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(lion), (10, 9));
    // Negative power: X is 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arahbo, Roar of the World");
    let lion = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(lion, pl(P1))]);
    crate::r_p017_common::modify_no_settle(
        &mut t,
        lion,
        vec![Modification::ModifyPT(Value::c(-4), Value::c(0))],
    );
    pool(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(lion), (-2, 1));
    assert!(t.obj_now(lion).has_keyword(KeywordKind::Trample));
}

#[test]
fn arahbos_bonuses_stay_after_arahbo_leaves() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Arahbo, Roar of the World",
        "Once either of Arahbo's triggered abilities has resolved, the affected Cat keeps its bonus even if Arahbo changes zones later in the turn."
    );
    let mut t = TestGame::new(2);
    let arahbo = t.battlefield(P0, "Arahbo, Roar of the World");
    let lion = t.battlefield(P0, "Savannah Lions");
    // Eminence at the beginning of combat: +3/+3.
    t.answer_targets(P0, &[o(lion)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(lion), (5, 4));
    attack_with(&mut t, &[(lion, pl(P1))]);
    pool(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(lion), (10, 9));
    move_to(&mut t, arahbo, Zone::Exile);
    assert_eq!(t.pt(lion), (10, 9));
    assert!(t.obj_now(lion).has_keyword(KeywordKind::Trample));
}

// ---------------------------------------------------------------------------------------
// One trigger per attacking creature
// ---------------------------------------------------------------------------------------

#[test]
fn kestia_draws_once_per_qualifying_attacker_you_control() {
    cr!("508.1m", "603.2");
    ruling!(
        "Kestia, the Cultivator",
        "If an enchanted enchantment creature attacks, you draw only one card for it."
    );
    ruling!(
        "Kestia, the Cultivator",
        "If multiple enchanted or enchantment creatures you control attack, you draw that many cards."
    );
    ruling!(
        "Kestia, the Cultivator",
        "The phrase \"you control\" applies to both \"enchanted creature\" and \"enchantment creature.\" That is, Kestia's ability triggers when an enchanted creature you control attacks, not any player's enchanted creature."
    );
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.battlefield(P0, "Kestia, the Cultivator");
    // An enchanted enchantment creature, an enchantment creature, an enchanted creature,
    // and a plain creature.
    let both = t.battlefield(P0, "Nyxborn Marauder");
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", both);
    let ench = t.battlefield(P0, "Nyxborn Marauder");
    let enchanted = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", enchanted);
    let plain = t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &[
            (both, pl(P1)),
            (ench, pl(P1)),
            (enchanted, pl(P1)),
            (plain, pl(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // An opponent's enchanted creature attacking doesn't trigger P0's Kestia.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.battlefield(P0, "Kestia, the Cultivator");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", theirs);
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(theirs, pl(P0))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn beastmaster_ascension_triggers_for_each_attacker() {
    cr!("508.1m", "603.2");
    ruling!(
        "Beastmaster Ascension",
        "If you attack with multiple creatures, Beastmaster Ascension's first ability triggers multiple times."
    );
    let mut t = TestGame::new(2);
    let ba = t.battlefield(P0, "Beastmaster Ascension");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))]);
    assert_eq!(triggers_from(&t, ba), 3);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.resolve_all();
    assert_eq!(t.counters(ba, "quest"), 3);
}

// ---------------------------------------------------------------------------------------
// Resolving in the declare attackers step
// ---------------------------------------------------------------------------------------

#[test]
fn attack_triggers_resolve_before_blockers_are_declared() {
    cr!("508.2", "509.1");
    ruling!(
        "Argentum Armor",
        "The second ability triggers and resolves during the declare attackers step. The target permanent will be destroyed before blockers are declared."
    );
    ruling!(
        "Angelic Benediction",
        "The creature you target with the last ability will become tapped before blockers are declared."
    );
    // Argentum Armor destroys the would-be blocker.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Argentum Armor", bear);
    let wall = t.battlefield(P1, "Wall of Wood");
    t.answer_targets(P0, &[o(wall)]);
    attack_with(&mut t, &[(bear, pl(P1))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wall of Wood"));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 12);
    // Angelic Benediction taps it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Benediction");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.answer_targets(P0, &[o(wall)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bear, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(tapped(&t, wall));
    // The tapped Wall can't block.
    assert!(!legal_blocks(&mut t, P1, &[(wall, bear)]));
}

#[test]
fn argentum_armor_may_target_any_permanent_even_your_own() {
    cr!("115.1", "603.3d");
    ruling!(
        "Argentum Armor",
        "You may target any permanent with the triggered ability, not just one controlled by the defending player. If no other player controls a permanent, you must target one of your own."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Argentum Armor", bear);
    let mine = t.battlefield(P0, "Forest");
    let mark = t.asked().len();
    attack_with(&mut t, &[(bear, pl(P1))]);
    let cands = crate::r_s02_common::target_candidates(&t, P0, mark);
    assert!(!cands.is_empty() && cands[0].contains(&o(mine)));
    t.resolve_all();
    // One of P0's own permanents was destroyed (the only kind there is).
    let survivors = t.g.permanents().filter(|p| p.controller == P0).count();
    assert_eq!(
        survivors, 2,
        "of the bear, the Armor and the Forest, one is gone"
    );
}

// ---------------------------------------------------------------------------------------
// Attacks alone
// ---------------------------------------------------------------------------------------

#[test]
fn attacks_alone_means_the_only_creature_declared_as_an_attacker() {
    cr!("506.5", "508.1m");
    ruling!(
        "Team Avatar",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable). For example, Team Avatar's triggered ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    ruling!(
        "Asari Captain",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step. For example, Asari Captain's triggered ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    for name in ["Team Avatar", "Asari Captain"] {
        // Two declared, one removed from combat: no trigger, no bonus.
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let a = t.battlefield(P0, "Elvish Warrior");
        let b = t.battlefield(P0, "Elvish Warrior");
        attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
        assert_eq!(triggers_from(&t, src), 0, "{name}");
        move_to(&mut t, b, Zone::Exile);
        t.settle();
        assert_eq!(triggers_from(&t, src), 0, "{name}");
        t.resolve_all();
        assert_eq!(t.pt(a), (2, 3), "{name}");
        // Declared alone: it triggers.
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let a = t.battlefield(P0, "Elvish Warrior");
        attack_with(&mut t, &[(a, pl(P1))]);
        assert_eq!(triggers_from(&t, src), 1, "{name}");
    }
}

#[test]
fn asari_captain_counts_samurai_and_warriors_once_each_including_the_attacker() {
    cr!("506.5", "608.2h", "205.3m");
    ruling!(
        "Asari Captain",
        "Asari Captain's triggered ability gives the solitary attacker +1/+0 for each creature you control that is either a Samurai or a Warrior as it resolves, including the attacker itself, if applicable. A creature that is both a Samurai and a Warrior is only counted once."
    );
    let mut t = TestGame::new(2);
    // Asari Captain (Samurai), the attacker (Warrior), a changeling (both), a Bears.
    t.battlefield(P0, "Asari Captain");
    let w = t.battlefield(P0, "Elvish Warrior");
    t.battlefield(P0, "Woodland Changeling");
    t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(w, pl(P1))]);
    // Another Samurai arrives before it resolves: counted.
    t.battlefield(P0, "Devoted Retainer");
    t.resolve_all();
    assert_eq!(t.pt(w), (2 + 4, 3));
}

#[test]
fn team_avatar_x_is_locked_in_as_it_resolves() {
    cr!("608.2h", "506.5");
    ruling!(
        "Team Avatar",
        "The value of X is determined only as Team Avatar's triggered ability begins to resolve. It won't change later in the turn if the number of creatures you control changes."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Team Avatar");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bear, pl(P1))]);
    t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(bear), (5, 5));
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bear), (5, 5));
}

// ---------------------------------------------------------------------------------------
// Blocking requirements
// ---------------------------------------------------------------------------------------

/// Declare Dominance / Monstrous Step / Emergent Growth: creatures that can't block
/// (tapped) or whose block has a cost aren't required to block.
#[test]
fn blocking_requirements_skip_tapped_creatures_and_creatures_with_a_blocking_cost() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Declare Dominance",
        "If a creature the defending player controls can’t block the target creature for any reason (such as being tapped), then it doesn’t block that creature. If there’s a cost associated with having a creature block the attacking creature, the defending player isn’t forced to pay that cost, so it doesn’t have to block in that case either."
    );
    ruling!(
        "Monstrous Step",
        "If the second target creature can’t block the first target creature for any reason (such as being tapped), then it doesn’t block. If there’s a cost associated with having it block the creature, the player isn’t forced to pay that cost, so it doesn’t have to block in that case either. That creature is free to block another creature or to not block at all."
    );
    ruling!(
        "Emergent Growth",
        "If the target creature attacks, the defending player must assign at least one blocker to it during the declare blockers step if that player controls any creatures that could block it. Other creatures may also block it, block other creatures, or not block at all."
    );
    // Declare Dominance: every untapped potential blocker must block; a tapped one can't.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Savannah Lions");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    let w3 = t.battlefield(P1, "Wall of Wood");
    t.g.tap(w3);
    cast_at(&mut t, P0, "Declare Dominance", bear);
    t.resolve_all();
    attack_with(&mut t, &[(bear, pl(P1)), (lion, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(w1, bear)]));
    assert!(!legal_blocks(&mut t, P1, &[(w1, bear), (w2, lion)]));
    assert!(legal_blocks(&mut t, P1, &[(w1, bear), (w2, bear)]));
    assert!(!legal_blocks(
        &mut t,
        P1,
        &[(w1, bear), (w2, bear), (w3, bear)]
    ));

    // Monstrous Step: the second target must block the first; tapped, it's free.
    for tap in [false, true] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let lion = t.battlefield(P0, "Savannah Lions");
        let w1 = t.battlefield(P1, "Wall of Wood");
        cast_how(
            &mut t,
            P0,
            "Monstrous Step",
            CastMethod::Normal,
            &[&[o(bear)], &[o(w1)]],
            None,
            None,
        );
        t.resolve_all();
        attack_with(&mut t, &[(bear, pl(P1)), (lion, pl(P1))]);
        if tap {
            t.g.tap(w1);
        }
        assert_eq!(legal_blocks(&mut t, P1, &[]), tap, "tapped: {tap}");
        assert!(!legal_blocks(&mut t, P1, &[(w1, lion)]) || tap);
        assert_eq!(legal_blocks(&mut t, P1, &[(w1, bear)]), !tap);
    }

    // Emergent Growth: at least one blocker; others are free.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Savannah Lions");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    cast_at(&mut t, P0, "Emergent Growth", bear);
    t.resolve_all();
    attack_with(&mut t, &[(bear, pl(P1)), (lion, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(w1, lion)]));
    assert!(legal_blocks(&mut t, P1, &[(w1, bear)]));
    assert!(legal_blocks(&mut t, P1, &[(w1, bear), (w2, bear)]));
    assert!(legal_blocks(&mut t, P1, &[(w1, bear), (w2, lion)]));

    // A blocking cost (War Cadence with X = 0, still a cost): blocking is optional for both.
    interp!("combat-tax-of-zero-is-a-cost");
    for which in ["Declare Dominance", "Monstrous Step"] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let w1 = t.battlefield(P1, "Wall of Wood");
        let cadence = t.battlefield(P0, "War Cadence");
        let (tb, tw) = ([o(bear)], [o(w1)]);
        let targets: Vec<&[Entity]> = if which == "Monstrous Step" {
            vec![&tb, &tw]
        } else {
            vec![&tb]
        };
        cast_how(&mut t, P0, which, CastMethod::Normal, &targets, None, None);
        t.resolve_all();
        pool(&mut t, P0);
        t.answer(P0, DecisionKind::X, Answer::Number(0));
        t.activate(P0, cadence, 0, &[]).unwrap();
        t.resolve_all();
        attack_with(&mut t, &[(bear, pl(P1))]);
        assert!(legal_blocks(&mut t, P1, &[]), "{which}");
        assert!(legal_blocks(&mut t, P1, &[(w1, bear)]), "{which}");
    }
}

// ---------------------------------------------------------------------------------------
// Redundant keyword counters
// ---------------------------------------------------------------------------------------

#[test]
fn multiple_menace_trample_or_lifelink_counters_are_redundant() {
    cr!("702.111c", "702.15f", "702.19g");
    ruling!(
        "T-45 Power Armor",
        "Multiple instances of menace are redundant. The same is true for trample and lifelink."
    );
    // T-45 Power Armor (+3/+3) puts a keyword counter on the equipped creature at each of
    // P0's upkeeps; the other counters are added directly so each kind is doubled.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "T-45 Power Armor", bear);
    t.g.add_counters(pl(P0), "energy", 2, None);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let by_t45: u32 = ["menace", "lifelink", "trample"]
        .iter()
        .map(|k| t.counters(bear, k))
        .sum();
    assert_eq!(by_t45, 1, "one keyword counter from T-45 Power Armor");
    for kind in ["menace", "lifelink", "trample"] {
        let have = t.counters(bear, kind);
        t.g.add_counters(o(bear), kind, 2 - have, None);
    }
    t.g.recompute();
    assert_eq!(t.pt(bear), (5, 5));
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bear, pl(P1))]);
    // Menace twice: still two blockers are enough, one isn't.
    assert!(!legal_blocks(&mut t, P1, &[(w1, bear)]));
    assert!(legal_blocks(&mut t, P1, &[(w1, bear), (w2, bear)]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    // Lifelink twice: 5 damage, 5 life.
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 25);
}
