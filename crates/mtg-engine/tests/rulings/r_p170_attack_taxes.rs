//! Rulings batch P170 — "creatures can't attack you or planeswalkers you control unless
//! their controller pays ..." (CR 508.1d, 508.1h): a creature that attacks if able isn't
//! forced to pay; in Two-Headed Giant attacks on the teammate aren't taxed (CR 805.10b,
//! 810.7); effects that don't change characteristics apply to creatures arriving later
//! and outlast their source (CR 611.2c); and attacks already declared stay (CR 506.4).

use crate::r_p061_common::declare_from;
use crate::r_p170_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 sets up the tax `tax` (Sivitri's +1, Baird or Forbidding Spirit's trigger).
fn p1_taxes(t: &mut TestGame, tax: &str) {
    supported(tax);
    t.set_step(P1, Step::PrecombatMain);
    match tax {
        "Sivitri, Dragon Master" => {
            let s = t.battlefield(P1, tax);
            t.activate(P1, s, 0, &[]).unwrap();
        }
        "Forbidding Spirit" => {
            t.enter(P1, tax);
        }
        _ => {
            t.battlefield(P1, tax);
        }
    }
    t.resolve_all();
}

#[test]
fn a_creature_that_attacks_if_able_isnt_forced_to_pay_the_tax() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Sivitri, Dragon Master",
        "A player who controls a creature that “attacks if able” can decline to pay additional costs to have that creature attack. If they do and there’s no other player or planeswalker to attack, that creature simply doesn’t attack."
    );
    ruling!(
        "Baird, Steward of Argive",
        "If you control Baird, your opponents can choose not to pay to attack with a creature that attacks \"if able.\" If there's no other player, planeswalker, or battle to attack, that creature simply doesn't attack."
    );
    ruling!(
        "Forbidding Spirit",
        "While Forbidding Spirit’s effect is in effect, your opponents can choose not to pay to attack with a creature that attacks “if able.” If there’s no other player or planeswalker to attack, that creature simply doesn’t attack."
    );
    for tax in [
        "",
        "Sivitri, Dragon Master",
        "Baird, Steward of Argive",
        "Forbidding Spirit",
    ] {
        let mut t = TestGame::new(2);
        if !tax.is_empty() {
            p1_taxes(&mut t, tax);
        }
        let jugg = t.battlefield(P0, "Juggernaut");
        let lands = t.lands(P0, "Wastes", 2);
        t.set_step(P0, Step::BeginningOfCombat);
        let attacks = declare_from(&mut t, P0, &[]);
        if tax.is_empty() {
            assert_eq!(attacks, vec![(jugg, pl(P1))], "it attacks when it's free");
        } else {
            assert!(attacks.is_empty(), "{tax}: {attacks:?}");
            assert_eq!(t.life(P0), 20);
            assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
        }
    }
}

#[test]
fn attack_taxes_dont_cover_the_teammate_in_two_headed_giant() {
    cr!("810.7", "805.10b", "508.1h");
    ruling!(
        "Sivitri, Dragon Master",
        "In a Two-Headed Giant game, creatures can attack your teammate and planeswalkers your teammate controls without requiring a life payment."
    );
    ruling!(
        "Archon of Absolution",
        "In a Two-Headed Giant game, creatures can attack your teammate and planeswalkers your teammate controls without requiring a mana payment."
    );
    ruling!(
        "Forbidding Spirit",
        "In a Two-Headed Giant game, creatures can attack your teammate and planeswalkers your teammate controls without requiring a mana payment."
    );
    ruling!(
        "Baird, Steward of Argive",
        "In a Two-Headed Giant game, creatures can attack your teammate and planeswalkers your teammate controls without requiring a mana payment. This is a change from previous rules."
    );
    // P0 and P1 are teammates; P0 has the tax. P2 (no mana) attacks P1 and P1's Jace.
    for tax in [
        "Sivitri, Dragon Master",
        "Archon of Absolution",
        "Forbidding Spirit",
        "Baird, Steward of Argive",
    ] {
        supported(tax);
        let mut t = crate::r_p108_common::two_headed_giant();
        match tax {
            "Sivitri, Dragon Master" => {
                let s = t.battlefield(P0, tax);
                t.activate(P0, s, 0, &[]).unwrap();
            }
            "Forbidding Spirit" => {
                t.enter(P0, tax);
            }
            _ => {
                t.battlefield(P0, tax);
            }
        }
        t.resolve_all();
        let jace = t.battlefield(P1, "Jace Beleren");
        let a = t.battlefield(P2, "Hill Giant");
        let b = t.battlefield(P2, "Grizzly Bears");
        let life = t.life(P2);
        t.set_step(P2, Step::BeginningOfCombat);
        let attacks = declare_from(&mut t, P2, &[(a, pl(P1)), (b, obj(jace))]);
        assert_eq!(attacks.len(), 2, "{tax}: {attacks:?}");
        assert_eq!(t.life(P2), life, "{tax}: no life paid");
        // The control: attacking P0 itself costs something P2 can't pay (no mana), or
        // life for Sivitri.
        let mut t2 = crate::r_p108_common::two_headed_giant();
        match tax {
            "Sivitri, Dragon Master" => {
                let s = t2.battlefield(P0, tax);
                t2.activate(P0, s, 0, &[]).unwrap();
            }
            "Forbidding Spirit" => {
                t2.enter(P0, tax);
            }
            _ => {
                t2.battlefield(P0, tax);
            }
        }
        t2.resolve_all();
        let c2 = t2.battlefield(P2, "Craw Wurm");
        let life = t2.life(P2);
        t2.set_step(P2, Step::BeginningOfCombat);
        let attacks = declare_from(&mut t2, P2, &[(c2, pl(P0))]);
        if tax.starts_with("Sivitri") {
            assert_eq!(attacks.len(), 1);
            assert_eq!(t2.life(P2), life - 2, "two life paid");
        } else {
            assert!(attacks.is_empty(), "{tax}: P2 can't pay");
        }
    }
}

#[test]
fn forbidding_spirits_effect_covers_new_creatures_and_outlasts_it() {
    cr!("611.2c", "611.2a", "508.1h");
    ruling!(
        "Forbidding Spirit",
        "Because it doesn’t modify the characteristics of creatures, Forbidding Spirit’s triggered ability can affect creatures that enter the battlefield after that ability resolves."
    );
    ruling!(
        "Forbidding Spirit",
        "Forbidding Spirit’s effect continues until your next turn even if Forbidding Spirit leaves the battlefield before then."
    );
    supported("Forbidding Spirit");
    for mana in [0, 2] {
        let mut t = TestGame::new(2);
        let spirit = t.enter(P0, "Forbidding Spirit");
        t.resolve_all();
        crate::r_s02_common::destroy(&mut t, spirit);
        // P1's creature arrives afterwards.
        let giant = t.battlefield(P1, "Hill Giant");
        let lands = t.lands(P1, "Wastes", mana);
        t.set_step(P1, Step::BeginningOfCombat);
        let attacks = declare_from(&mut t, P1, &[(giant, pl(P0))]);
        if mana == 0 {
            assert!(attacks.is_empty(), "it can't attack without paying");
        } else {
            assert_eq!(attacks, vec![(giant, pl(P0))]);
            assert!(lands.iter().all(|l| t.obj_now(*l).tapped), "{{2}} paid");
        }
    }
}

/// P0's Orzhov Advokist: at P0's next upkeep P1 puts the counters on `creature`; then it's
/// P1's turn (the same one), at the beginning of combat.
fn advokist_accepted(t: &mut TestGame, creature: ObjectId) -> ObjectId {
    supported("Orzhov Advokist");
    let adv = t.battlefield(P0, "Orzhov Advokist");
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(creature)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(creature, "+1/+1"), 2);
    adv
}

#[test]
fn orzhov_advokists_counters_go_on_chosen_creatures_without_targeting() {
    cr!("115.1", "608.2c");
    ruling!(
        "Orzhov Advokist",
        "Orzhov Advokist's ability doesn't target the creatures that receive +1/+1 counters. Their controllers choose them as the ability resolves. Players can't take actions between making their choices and the +1/+1 counters being placed."
    );
    // P1's hexproof Gladecover Scout can get them.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let from = t.asked().len();
    advokist_accepted(&mut t, scout);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
}

#[test]
fn orzhov_advokists_gift_stops_every_attack_on_you_until_your_next_turn() {
    cr!("611.2c", "611.2a", "508.1c");
    ruling!(
        "Orzhov Advokist",
        "If a player chooses to accept Orzhov Advokist's gift, that player can't attack you or a planeswalker you control with any creatures during their next turn, even creatures that weren't on the battlefield as Orzhov Advokist's ability resolved."
    );
    ruling!(
        "Orzhov Advokist",
        "Orzhov Advokist's effect keeps creatures from attacking you even if Orzhov Advokist leaves the battlefield, if the creature that received counters is no longer on the battlefield, or if an effect has moved those counters off of the creature that received them and put them onto another creature."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let adv = advokist_accepted(&mut t, bears);
        let jace = t.battlefield(P0, "Jace Beleren");
        if leaves {
            crate::r_s02_common::destroy(&mut t, adv);
            crate::r_s02_common::destroy(&mut t, bears);
        }
        // A creature P1 gets afterwards can't attack P0 or Jace either.
        let giant = t.battlefield(P1, "Hill Giant");
        t.advance_to(P1, Step::BeginningOfCombat);
        let mut decl = vec![(giant, pl(P0))];
        if !leaves {
            decl.push((bears, obj(jace)));
        }
        let attacks = declare_from(&mut t, P1, &decl);
        assert!(attacks.is_empty(), "leaves {leaves}: {attacks:?}");
    }
    // Counters moved to another creature (the Bears die after giving them away): still
    // no attacks.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    advokist_accepted(&mut t, bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.remove_counters(Entity::Object(bears), "+1/+1", 2);
    add_counters(&mut t, giant, "+1/+1", 2);
    t.advance_to(P1, Step::BeginningOfCombat);
    let attacks = declare_from(&mut t, P1, &[(giant, pl(P0)), (bears, pl(P0))]);
    assert!(attacks.is_empty());
}

#[test]
fn sandwurm_convergence_doesnt_undo_attacks() {
    cr!("506.4a", "508.4", "508.1c");
    ruling!(
        "Sandwurm Convergence",
        "Once a creature has attacked you or a planeswalker you control, it will remain in combat even if it gains flying. Similarly, if a creature with flying enters the battlefield attacking, Sandwurm Convergence's first ability can't undo that attack."
    );
    supported("Sandwurm Convergence");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sandwurm Convergence");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    t.set_step(P0, Step::BeginningOfCombat);
    let attacks = declare_from(&mut t, P0, &[(bears, pl(P1)), (geist, pl(P1))]);
    assert_eq!(attacks.len(), 2);
    t.resolve_all();
    // Geist's Angel token with flying entered attacking P1.
    let angels = tokens_with_subtype(&t, P0, "Angel");
    assert_eq!(angels.len(), 1);
    assert_eq!(crate::r_p015_common::attacking(&t, angels[0]), Some(pl(P1)));
    // The Bears gain flying: still attacking.
    cast_card(&mut t, P0, "Jump", &[obj(bears)]);
    t.resolve_all();
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(crate::r_p015_common::attacking(&t, bears), Some(pl(P1)));
    t.advance_to(P0, Step::EndOfCombat);
    // 2 (Bears) + 2 (Geist) + 4 (Angel).
    assert_eq!(t.life(P1), 12);
}
