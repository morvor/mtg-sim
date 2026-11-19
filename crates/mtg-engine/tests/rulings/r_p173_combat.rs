//! Rulings batch P173 — tapping in combat: tapping an attacking creature doesn't remove it
//! from combat (CR 506.4, 508.1f), so attack triggers that tap creatures as they resolve
//! can tap attackers with vigilance or that became untapped, the source included (CR
//! 608.2d); a creature that became blocked stays blocked (CR 509.1h, 506.4); "becomes
//! blocked by a creature" triggers once per blocker (CR 509.3d); and a creature that
//! attacks each combat if able attacks whichever player, planeswalker or battle its
//! controller chooses (CR 508.1d).

use crate::r_p212_common::{back_face, grant};
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s24_common::tapped;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|i| obj(*i)).collect()
}

/// Finishes combat with no blocks.
fn no_blocks(t: &mut TestGame) {
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
}

// --- Attack triggers that tap attacking creatures --------------------------------------------

#[test]
fn attack_triggers_tap_attackers_with_vigilance_which_keep_attacking() {
    cr!("506.4", "508.1f", "608.2d", "702.20b");
    // The source has vigilance (granted), and a vigilant Human attacks with it.
    ruling!(
        "Ledev Champion",
        "Attacking creatures with vigilance may be tapped for Ledev Champion's first ability. If Ledev Champion itself has vigilance, it can be tapped this way, too. These creatures aren't removed from combat."
    );
    ruling!(
        "Orphans of the Wheat",
        "If Orphans of the Wheat is untapped as its ability is resolving (for example, if it has vigilance), it can be tapped for its own ability."
    );
    ruling!(
        "Siege Striker",
        "If Siege Striker gains vigilance and attacks or attacks and becomes untapped before its last ability resolves, it can be tapped for its own ability."
    );
    ruling!(
        "Siege Striker",
        "If an attacking creature with vigilance is tapped for Siege Striker's ability, that creature remains an attacking creature."
    );
    ruling!(
        "Minas Tirith Garrison",
        "If Minas Tirith Garrison is untapped as its last ability resolves, it can be tapped for its own last ability."
    );
    ruling!(
        "Minas Tirith Garrison",
        "If an attacking Human is tapped for Minas Tirith Garrison's ability (perhaps because it has vigilance or it was untapped after being declared as an attacker), that Human remains an attacking creature."
    );
    for name in [
        "Ledev Champion",
        "Orphans of the Wheat",
        "Siege Striker",
        "Minas Tirith Garrison",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let human = t.battlefield(P0, "Alaborn Trooper");
        grant(&mut t, src, KeywordKind::Vigilance);
        grant(&mut t, human, KeywordKind::Vigilance);
        t.lands(P0, "Island", 2);
        t.hand(P0, "Grizzly Bears");
        t.hand(P0, "Grizzly Bears");
        let before = t.pt(src);
        let hand = t.hand_size(P0);
        attack_with(&mut t, &[(src, Entity::Player(P1)), (human, Entity::Player(P1))]);
        assert!(!tapped(&t, src) && !tapped(&t, human), "{name}");
        t.answer_choose(P0, &objs(&[src, human]));
        t.resolve_all();
        assert!(tapped(&t, src) && tapped(&t, human), "{name}: {}", t.dump_log());
        assert!(t.g.is_attacking(src) && t.g.is_attacking(human), "{name}");
        let after = t.pt(src);
        if name == "Minas Tirith Garrison" {
            assert_eq!(t.hand_size(P0), hand + 2, "{name}");
        } else {
            assert_eq!((after.0 - before.0, after.1 - before.1), (2, 2), "{name}");
        }
        let power = t.pt(src).0 * if name == "Siege Striker" { 2 } else { 1 };
        let other = t.pt(human).0;
        no_blocks(&mut t);
        assert_eq!(t.life(P1), 20 - power - other, "{name}: both still dealt damage");
    }
    // Siege Striker without vigilance, untapped after attacking.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Siege Striker");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    assert!(tapped(&t, s));
    t.g.untap(s);
    t.answer_choose(P0, &[obj(s)]);
    t.resolve_all();
    assert!(tapped(&t, s) && t.g.is_attacking(s));
    assert_eq!(t.pt(s), (2, 2));
    no_blocks(&mut t);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn ledev_champion_taps_creatures_that_came_under_your_control_this_turn() {
    cr!("302.6", "608.2d");
    ruling!(
        "Ledev Champion",
        "Creatures that have come under your control this turn can be tapped for Ledev Champion's first ability."
    );
    ruling!(
        "Ledev Champion",
        "For Ledev Champion's first ability, the creatures to tap are chosen as the ability resolves. No player may take actions between the time you choose the creatures to tap and the time Ledev Champion's power and toughness are boosted."
    );
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Ledev Champion");
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Grizzly Bears");
    attack_with(&mut t, &[(l, Entity::Player(P1))]);
    // Nothing's chosen before the ability resolves.
    let from = t.asked().len();
    t.answer_choose(P0, &objs(&[a, b]));
    t.g.resolve_top();
    // Chosen and boosted within the resolution: no one was asked anything else between.
    let asked = crate::r_s01_common::asked_since(&t, from);
    assert!(asked.iter().all(|(p, _)| *p == P0), "{asked:?}");
    assert!(tapped(&t, a) && tapped(&t, b));
    assert_eq!(t.pt(l), (4, 4));
}

#[test]
fn redcap_raiders_taps_a_vigilant_attacker_with_no_window_before_the_bonus() {
    cr!("506.4", "608.2d", "608.2c");
    ruling!(
        "Redcap Raiders",
        "Tapping an attacking creature with vigilance to pay the cost of Redcap Raiders's ability won't cause that creature to leave combat or to stop attacking."
    );
    ruling!(
        "Redcap Raiders",
        "Players can't take any actions between the time you choose whether to tap a creature and the time Redcap Raiders gets +1/+1 and gains trample."
    );
    supported("Redcap Raiders");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Redcap Raiders");
    let angel = t.battlefield(P0, "Serra Angel");
    attack_with(&mut t, &[(r, Entity::Player(P1)), (angel, Entity::Player(P1))]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(angel)]);
    t.g.resolve_top();
    assert!(crate::r_s01_common::asked_since(&t, from)
        .iter()
        .all(|(p, _)| *p == P0));
    assert!(tapped(&t, angel) && t.g.is_attacking(angel));
    assert_eq!(t.pt(r), (4, 3));
    no_blocks(&mut t);
    assert_eq!(t.life(P1), 20 - 4 - 4);
}

#[test]
fn siege_striker_keeps_its_bonus_when_the_tapped_creatures_leave_or_untap() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Siege Striker",
        "Once Siege Striker's ability has resolved, it keeps the power-and-toughness bonus even if some of the tapped creatures leave the battlefield or become untapped."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Siege Striker");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.answer_choose(P0, &objs(&[a, b]));
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 3));
    crate::r_s02_common::destroy(&mut t, a);
    t.g.untap(b);
    t.g.recompute();
    assert_eq!(t.pt(s), (3, 3));
}

// --- Tapping attackers with activated abilities ----------------------------------------------

#[test]
fn aryel_and_frondland_felidar_tap_while_attacking_and_stay_in_combat() {
    cr!("506.4", "702.20b", "602.2");
    ruling!(
        "Aryel, Knight of Windgrace",
        "Tapping Aryel to activate either of its abilities while it's attacking doesn't remove it from combat."
    );
    supported("Aryel, Knight of Windgrace");
    // The first ability.
    let mut t = TestGame::new(2);
    let aryel = t.battlefield(P0, "Aryel, Knight of Windgrace");
    t.lands(P0, "Plains", 3);
    attack_with(&mut t, &[(aryel, Entity::Player(P1))]);
    assert!(!tapped(&t, aryel));
    activate_containing(&mut t, P0, aryel, "Create a 2/2").expect("activates");
    t.resolve_all();
    assert!(tapped(&t, aryel) && t.g.is_attacking(aryel));
    no_blocks(&mut t);
    assert_eq!(t.life(P1), 16);
    // The second (X = 0, destroying a 0-power creature).
    let mut t = TestGame::new(2);
    let aryel = t.battlefield(P0, "Aryel, Knight of Windgrace");
    t.lands(P0, "Swamp", 1);
    let thopter = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(aryel, Entity::Player(P1))]);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer_targets(P0, &[obj(thopter)]);
    activate_containing(&mut t, P0, aryel, "Destroy target").expect("activates");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(tapped(&t, aryel) && t.g.is_attacking(aryel));
    no_blocks(&mut t);
    assert_eq!(t.life(P1), 16);

    ruling!(
        "Frondland Felidar",
        "Tapping an attacking creature doesn’t remove it from combat."
    );
    supported("Frondland Felidar");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Frondland Felidar");
    let angel = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(f, Entity::Player(P1)), (angel, Entity::Player(P1))]);
    // The Felidar taps itself for its granted ability; the Angel's taps the Bears.
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, f, "Tap target creature").expect("Felidar");
    t.answer_targets(P0, &[obj(f)]);
    activate_containing(&mut t, P0, angel, "Tap target creature").expect("Angel");
    t.resolve_all();
    assert!(tapped(&t, f) && tapped(&t, angel) && tapped(&t, bears));
    assert!(t.g.is_attacking(f) && t.g.is_attacking(angel));
    no_blocks(&mut t);
    assert_eq!(t.life(P1), 20 - 3 - 4);
}

#[test]
fn becoming_unblockable_after_blocks_doesnt_unblock() {
    cr!("509.1h", "506.4");
    ruling!(
        "Kumena, Tyrant of Orazca",
        "Activating Kumena's first ability after it has become blocked won't cause it to become unblocked."
    );
    ruling!(
        "Sure-Footed Infiltrator",
        "Once a creature has blocked Sure-Footed Infiltrator, activating its first ability won't cause it to become unblocked or remove that blocking creature from combat."
    );
    for (name, helper) in [
        ("Kumena, Tyrant of Orazca", "Merfolk of the Pearl Trident"),
        ("Sure-Footed Infiltrator", "Merfolk Looter"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let h = t.battlefield(P0, helper);
        let wall = t.battlefield(P1, "Wall of Wood");
        attack_with(&mut t, &[(src, Entity::Player(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(wall, src)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        t.settle();
        t.answer_choose(P0, &[obj(h)]);
        activate_containing(&mut t, P0, src, "can't be blocked").expect("activates");
        t.resolve_all();
        assert!(tapped(&t, h));
        assert!(t.g.combat.as_ref().unwrap().is_blocked(src), "{name}: still blocked");
        assert!(t.g.is_blocking(wall), "{name}: the blocker stays in combat");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}");
        assert_eq!(t.obj_now(wall).damage, 2, "{name}");
    }
}

// --- Blocked and forced attacks --------------------------------------------------------------

#[test]
fn vicious_battlerager_triggers_for_each_blocker() {
    cr!("509.3d", "603.2c");
    ruling!(
        "Vicious Battlerager",
        "If Vicious Battlerager becomes blocked by more than one creature, its triggered ability triggers once for each creature that blocked it."
    );
    supported("Vicious Battlerager");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vicious Battlerager");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(a, v), (b, v)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "loses 5 life"),
        2
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn incited_rabble_attacks_whom_its_controller_chooses() {
    cr!("508.1d", "508.1a");
    ruling!(
        "Town Gossipmonger",
        "Incited Rabble’s controller still chooses which player, planeswalker, or battle it attacks."
    );
    supported("Town Gossipmonger");
    // P1 has a planeswalker; P0's Incited Rabble must attack and attacks it.
    let mut t = TestGame::new(2);
    let r = back_face(&mut t, P0, "Town Gossipmonger");
    assert_eq!(t.obj_now(r).chars.name.as_str(), "Incited Rabble");
    let pw = t.battlefield(P1, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(r, obj(pw))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(pw, "loyalty"), 1, "{}", t.dump_log());
    // Left to itself, it still attacks.
    let mut t = TestGame::new(2);
    let r = back_face(&mut t, P0, "Town Gossipmonger");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    let _ = r;
}
