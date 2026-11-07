//! Rulings batch P215 — flying (CR 702.9) in combat: attack triggers that count or pump
//! creatures with flying, flying gained after blocks, and attack restrictions.

use crate::r_p215_fight::grant;
use crate::r_p215_flash::lose;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::abilities_from;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const FLYING: KeywordKind = KeywordKind::Flying;

fn flies(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(FLYING)
}

fn give_flying(t: &mut TestGame, id: ObjectId) {
    let c = t.obj_now(id).controller;
    grant(t, c, id, Keyword::new(FLYING));
}

/// P0 declares `attackers` attacking P1; the attack triggers are on the stack.
fn declare(t: &mut TestGame, attackers: &[ObjectId]) {
    let a: Vec<_> = attackers
        .iter()
        .map(|id| (*id, Entity::Player(P1)))
        .collect();
    attack_with(t, &a);
}

#[test]
fn windreader_sphinx_controller_draws_for_any_attacking_flyer() {
    cr!("508.1m", "603.2", "702.9a");
    ruling!(
        "Windreader Sphinx",
        "It doesn't matter who controls the creature with flying or which player or planeswalker that creature is attacking. Windreader Sphinx's controller is the player who may draw a card."
    );
    ruling!(
        "Windreader Sphinx",
        "The creature must have flying immediately after being declared as an attacking creature in order for Windreader Sphinx's ability to trigger. For example, attacking with Kitesail Corsair will cause the ability to trigger, and attacking with Pegasus Courser and a creature without flying will cause the ability to trigger only once."
    );
    supported("Windreader Sphinx");
    // P1's Sphinx: P0's Storm Crow attacks, P1 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Windreader Sphinx");
    let crow = t.battlefield(P0, "Storm Crow");
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    declare(&mut t, &[crow]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    // Pegasus Courser gives the Bears flying after attackers are declared: one trigger.
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Windreader Sphinx");
    let courser = t.battlefield(P0, "Pegasus Courser");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    declare(&mut t, &[courser, bears]);
    assert_eq!(abilities_from(&t, sphinx).len(), 1);
    t.resolve_all();
    assert!(flies(&mut t, bears));
}

#[test]
fn tide_skimmer_triggers_once_and_draws_regardless() {
    cr!("508.1m", "603.2c", "608.2");
    ruling!(
        "Tide Skimmer",
        "Tide Skimmer's ability triggers just once, no matter how many creatures with flying attack beyond the second."
    );
    ruling!(
        "Tide Skimmer",
        "You draw a card even if some or all of the attacking creatures lose flying or leave combat before Tide Skimmer's ability resolves."
    );
    supported("Tide Skimmer");
    let mut t = TestGame::new(2);
    let skimmer = t.battlefield(P0, "Tide Skimmer");
    let a = t.battlefield(P0, "Storm Crow");
    let b = t.battlefield(P0, "Storm Crow");
    declare(&mut t, &[skimmer, a, b]);
    assert_eq!(abilities_from(&t, skimmer).len(), 1);
    lose(&mut t, a, FLYING);
    destroy(&mut t, b);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn kangee_bonuses_go_to_creatures_with_flying_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Kangee's Lieutenant",
        "Only attacking creatures that have flying as Kangee's Lieutenant's second ability resolves will get +1/+1. Giving a creature flying or making it lose flying after that point won't change that bonus, if any."
    );
    ruling!(
        "Kangee, Sky Warden",
        "Only attacking creatures that have flying as the first triggered ability resolves will get the +2/+0. Similarly, only blocking creatures that have flying as second triggered ability resolves will get +0/+2. Giving a creature flying or making it lose flying after that point won't change the bonus the creature has, if any."
    );
    for (card, bonus) in [
        ("Kangee's Lieutenant", (1, 1)),
        ("Kangee, Sky Warden", (2, 0)),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, card);
        let crow = t.battlefield(P0, "Storm Crow");
        let bears = t.battlefield(P0, "Grizzly Bears");
        declare(&mut t, &[k, crow, bears]);
        // Before it resolves: the Crow loses flying, the Bears gains it.
        lose(&mut t, crow, FLYING);
        give_flying(&mut t, bears);
        t.resolve_all();
        assert_eq!(t.pt(crow), (1, 2), "{card}");
        assert_eq!(t.pt(bears), (2 + bonus.0, 2 + bonus.1), "{card}");
        // Later changes don't matter.
        lose(&mut t, bears, FLYING);
        give_flying(&mut t, crow);
        assert_eq!(t.pt(crow), (1, 2), "{card}");
        assert_eq!(t.pt(bears), (2 + bonus.0, 2 + bonus.1), "{card}");
    }
    // Kangee blocking: blocking creatures with flying as it resolves get +0/+2.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let kangee = t.battlefield(P1, "Kangee, Sky Warden");
    let crow = t.battlefield(P1, "Storm Crow");
    declare(&mut t, &[attacker]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(kangee, attacker), (crow, attacker)]),
    );
    t.g.run_until(1000, |g| {
        g.turn.step == Step::DeclareBlockers && !g.stack.is_empty()
    });
    lose(&mut t, crow, FLYING);
    t.resolve_all();
    assert_eq!(t.pt(kangee), (3, 5));
    assert_eq!(t.pt(crow), (1, 2));
}

#[test]
fn cavalry_pegasus_humans_fly_before_blocks_and_keep_flying() {
    cr!("508.1m", "611.2c", "509.1b");
    ruling!(
        "Cavalry Pegasus",
        "The attacking Humans gain flying before blockers are declared."
    );
    ruling!(
        "Cavalry Pegasus",
        "Once the triggered ability resolves, the attacking Humans will continue to have flying even if Cavalry Pegasus leaves the battlefield."
    );
    supported("Cavalry Pegasus");
    let mut t = TestGame::new(2);
    let peg = t.battlefield(P0, "Cavalry Pegasus");
    let human = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[peg, human]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    destroy(&mut t, peg);
    assert!(flies(&mut t, human));
    assert!(!legal_blocks(&mut t, P1, &[(bears, human)]));
}

#[test]
fn victorys_herald_gains_flying_and_lifelink_too() {
    cr!("508.1m", "702.15b");
    ruling!(
        "Victory's Herald",
        "Victory’s Herald also gains flying (which will likely be redundant) and lifelink if it’s still attacking when the ability resolves."
    );
    supported("Victory's Herald");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Victory's Herald");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[herald, bears]);
    t.resolve_all();
    for id in [herald, bears] {
        assert!(t.obj_now(id).has_keyword(KeywordKind::Lifelink));
        assert!(flies(&mut t, id));
    }
}

/// The number of cards in the scry decisions asked since `from`.
fn scried(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards } => Some(cards.len()),
            _ => None,
        })
        .sum()
}

#[test]
fn siani_counts_attacking_flyers_as_it_resolves() {
    cr!("608.2h", "701.22a");
    ruling!(
        "Siani, Eye of the Storm",
        "Use the number of attacking creatures with flying as the ability resolves to determine the value of X."
    );
    supported("Siani, Eye of the Storm");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island"; 5]);
    let siani = t.battlefield(P0, "Siani, Eye of the Storm");
    let crow = t.battlefield(P0, "Storm Crow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[siani, crow, bears]);
    lose(&mut t, crow, FLYING);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(scried(&t, from), 1);
}

#[test]
fn lurking_green_dragon_keeps_attacking_if_the_flyer_leaves() {
    cr!("508.1c", "506.4");
    ruling!(
        "Lurking Green Dragon",
        "Lurking Green Dragon's second ability applies only while you're declaring attackers. It is not removed from combat if the creature with flying leaves the battlefield, changes controllers, or loses flying after Lurking Green Dragon is declared as an attacker."
    );
    supported("Lurking Green Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Lurking Green Dragon");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, dragon));
    let crow = t.battlefield(P1, "Storm Crow");
    assert!(can_attack(&mut t, dragon));
    declare(&mut t, &[dragon]);
    destroy(&mut t, crow);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn zenith_seeker_flying_after_blocks_doesnt_undo_the_block() {
    cr!("509.1h", "506.4");
    ruling!(
        "Zenith Seeker",
        "Giving a creature flying after it’s already been blocked won’t change or undo that block. If you want the flying to affect what can block the creature, you must cycle or discard a card during the declare attackers step at the latest."
    );
    supported("Zenith Seeker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zenith Seeker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[bears]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // Discard a card: the Bears gains flying.
    let card = t.hand(P0, "Island");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.discard(P0, card, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(flies(&mut t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(blocker));
}

#[test]
fn skyrider_patrol_reflexive_trigger_can_be_responded_to() {
    cr!("603.12", "117.3b");
    ruling!(
        "Skyrider Patrol",
        "Skyrider Patrol’s triggered ability goes on the stack without a target. While that ability is resolving, you may pay {G}{U}. If you do, a second ability triggers and you pick a target creature. This is different from abilities that say “If you do . . .” in that players may cast spells and activate abilities after mana is paid but before the target receives a counter and flying."
    );
    supported("Skyrider Patrol");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skyrider Patrol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // The reflexive trigger, with its target, waits on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // P1 responds by killing the Bears: no counter.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn kinscaer_harpoonist_may_target_a_creature_without_flying() {
    cr!("115.1", "603.3d");
    ruling!(
        "Kinscaer Harpoonist",
        "You may target a creature that doesn’t have flying."
    );
    supported("Kinscaer Harpoonist");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Kinscaer Harpoonist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    declare(&mut t, &[h]);
    let c = target_candidates(&t, P0, from);
    assert!(c[0].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(!flies(&mut t, bears));
}
