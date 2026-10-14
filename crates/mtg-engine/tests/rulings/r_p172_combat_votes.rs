//! Rulings batch P172 — trample and unblocked damage assignment (CR 510.1), attack
//! triggers that count on resolution, creatures with no abilities (CR 113), Vehicles and
//! crewing (CR 301.7), cost reductions to {0} (CR 601.2f), and additional votes
//! (CR 701.38d).

use crate::r_p021_common::animate_land;
use crate::r_p076_common::mana;
use crate::r_p117_common::modify;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::can_attack;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::{crew, is_creature};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, attach_new, damage, give_control};
use crate::r_s10_common::attacking;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::kwa::vote::VOTED;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn proud_wildbonder_assigns_to_the_player_despite_protection() {
    cr!("510.1c");
    ruling!(
        "Proud Wildbonder",
        "You can decide to assign combat damage to the defending player or planeswalker even if the blocking creature has protection from green or other damage prevention effects applying to it."
    );
    supported("Proud Wildbonder");
    supported("Mirran Crusader");
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Proud Wildbonder");
    let crusader = t.battlefield(P1, "Mirran Crusader");
    to_blockers(&mut t, &[(wb, Entity::Player(P1))], &[(crusader, wb)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(crusader));
}

#[test]
fn knotvine_paladin_counts_untapped_creatures_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Knotvine Paladin",
        "The number of untapped creatures you control is counted as the ability resolves, and the bonus is locked in at that time. By this point, other attacking creatures will already be tapped (unless they have vigilance)."
    );
    supported("Knotvine Paladin");
    let mut t = TestGame::new(2);
    let kp = t.battlefield(P0, "Knotvine Paladin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Serra Angel");
    attack_with(
        &mut t,
        &[(kp, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    // A creature tapped before it resolves isn't counted.
    t.g.tap(giant);
    t.resolve_all();
    assert_eq!(t.pt(kp), (3, 3));
    // Locked in: untapping the giant changes nothing.
    t.g.untap(giant);
    t.g.recompute();
    assert_eq!(t.pt(kp), (3, 3));
}

#[test]
fn muraganda_petroglyphs_counts_any_text_as_an_ability() {
    cr!("113.1", "113.3");
    ruling!(
        "Muraganda Petroglyphs",
        "Any text on a creature is an ability, even if that text isn't currently applicable (such as a cycling ability of a creature)."
    );
    ruling!(
        "Muraganda Petroglyphs",
        "If a land with a basic land type becomes a creature, the intrinsic mana ability it has stops Muraganda Petroglyphs from applying to it."
    );
    supported("Muraganda Petroglyphs");
    supported("Hundroog");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muraganda Petroglyphs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hundroog = t.battlefield(P0, "Hundroog");
    let forest = t.battlefield(P0, "Forest");
    animate_land(&mut t, forest);
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(hundroog), (4, 7));
    assert_eq!(t.pt(forest), (2, 2));
}

#[test]
fn damage_on_a_vanilla_creature_can_become_lethal_later() {
    cr!("120.6", "704.5g");
    ruling!(
        "Muraganda Petroglyphs",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature with no abilities may become lethal if that creature gains an ability or Muraganda Petroglyphs leaves the battlefield during that turn."
    );
    // It leaves the battlefield.
    let mut t = TestGame::new(2);
    let mp = t.battlefield(P0, "Muraganda Petroglyphs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, 3, bears);
    assert!(t.on_battlefield(bears));
    move_to(&mut t, mp, Zone::Graveyard(P0));
    assert!(!t.on_battlefield(bears));
    // It gains an ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muraganda Petroglyphs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, 3, bears);
    modify(
        &mut t,
        bears,
        vec![Modification::AddKeyword(
            mtg_engine::keywords::KeywordKind::Flying.into(),
        )],
    );
    t.settle();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn a_hasty_vehicle_must_still_be_crewed_to_attack() {
    cr!("301.7", "508.1a", "702.122a");
    ruling!(
        "Greasefang, Okiba Boss",
        "A Vehicle with haste must still be crewed (or become a creature some other way) before it can attack."
    );
    supported("Greasefang, Okiba Boss");
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Greasefang, Okiba Boss");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copter = t.graveyard(P0, "Smuggler's Copter");
    t.answer_targets(P0, &[obj(copter)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let copter = t.g.current(copter);
    assert!(t.on_battlefield(copter));
    assert!(!can_attack(&mut t, copter));
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(can_attack(&mut t, copter));
}

#[test]
fn collision_course_counts_a_creature_vehicle_once() {
    cr!("608.2h");
    ruling!(
        "Collision Course",
        "A permanent that is both a creature and a Vehicle is counted only once for this spell’s first mode."
    );
    supported("Collision Course");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.battlefield(P0, "Smuggler's Copter");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    let wurm = t.battlefield(P1, "Craw Wurm");
    let cc = t.hand(P0, "Collision Course");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, cc).modes(&[0]).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);
}

#[test]
fn boom_scholar_can_reduce_an_exhaust_cost_to_zero() {
    cr!("601.2f", "602.2b", "702.177a");
    ruling!(
        "Boom Scholar",
        "Boom Scholar’s first ability can cause the mana component of the cost to activate an ability to be {0}."
    );
    supported("Boom Scholar");
    supported("Jeong Jeong, the Deserter");
    // Two Boom Scholars reduce Jeong Jeong's "Exhaust — {3}" to {0}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Boom Scholar");
    t.battlefield(P0, "Boom Scholar");
    let jj = t.battlefield(P0, "Jeong Jeong, the Deserter");
    activate_containing(&mut t, P0, jj, "Exhaust").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(jj, "+1/+1"), 1);
    // With one, it costs {1}: no mana, no activation.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Boom Scholar");
    let jj = t.battlefield(P0, "Jeong Jeong, the Deserter");
    assert!(activate_containing(&mut t, P0, jj, "Exhaust").is_err());
}

#[test]
fn crewing_doesnt_change_the_crews_power_or_toughness() {
    cr!("702.122a");
    ruling!(
        "Giant Ox",
        "Crewing a Vehicle doesn't cause Giant Ox's power or toughness to change."
    );
    ruling!(
        "Dynamite Diver",
        "Saddling a Mount or crewing a Vehicle doesn’t cause this creature’s power to change."
    );
    supported("Giant Ox");
    supported("Dynamite Diver");
    supported("Weatherlight");
    let mut t = TestGame::new(2);
    let ox = t.battlefield(P0, "Giant Ox");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    assert!(crew(&mut t, P0, copter, &[ox]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(ox), (0, 6));
    // Dynamite Diver crews a crew 3 Vehicle alone.
    let mut t = TestGame::new(2);
    let diver = t.battlefield(P0, "Dynamite Diver");
    let wl = t.battlefield(P0, "Weatherlight");
    assert!(crew(&mut t, P0, wl, &[diver]));
    t.resolve_all();
    assert!(is_creature(&t, wl));
    assert_eq!(t.pt(diver), (1, 1));
}

#[test]
fn a_vehicle_losing_aerial_modification_is_removed_from_combat() {
    cr!("506.4", "301.7b");
    ruling!(
        "Aerial Modification",
        "If Aerial Modification becomes unattached from a Vehicle that’s attacking or blocking, that Vehicle will be removed from combat unless another effect (such as its crew ability) is also making it a creature."
    );
    supported("Aerial Modification");
    for crewed in [false, true] {
        let mut t = TestGame::new(2);
        let copter = t.battlefield(P0, "Smuggler's Copter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let aura = attach_new(&mut t, P0, "Aerial Modification", copter);
        assert!(is_creature(&t, copter));
        t.set_step(P0, Step::BeginningOfCombat);
        if crewed {
            assert!(crew(&mut t, P0, copter, &[bears]));
            t.resolve_all();
        }
        attack_with(&mut t, &[(copter, Entity::Player(P1))]);
        t.resolve_all();
        assert!(attacking(&t, copter));
        move_to(&mut t, aura, Zone::Graveyard(P0));
        assert_eq!(attacking(&t, copter), crewed, "crewed: {crewed}");
        assert_eq!(is_creature(&t, copter), crewed);
    }
}

#[test]
fn start_your_engines_affects_what_you_control_as_it_resolves() {
    cr!("611.2c", "613.4c");
    ruling!(
        "Start Your Engines",
        "The set of Vehicles and creatures affected by Start Your Engines is determined as the spell resolves."
    );
    ruling!(
        "Start Your Engines",
        "Vehicles you control become artifact creatures and then also get +2/+0."
    );
    supported("Start Your Engines");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their_copter = t.battlefield(P1, "Smuggler's Copter");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    let sye = t.hand(P0, "Start Your Engines");
    mana(&mut t, P0, ManaType::R, 4);
    t.cast(P0, sye).go();
    t.resolve_all();
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (5, 3));
    assert_eq!(t.pt(bears), (4, 2));
    give_control(&mut t, their_copter, P0);
    give_control(&mut t, their_bears, P0);
    assert!(!is_creature(&t, their_copter));
    assert_eq!(t.pt(their_bears), (2, 2));
}

fn voters(t: &TestGame) -> Vec<PlayerId> {
    t.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Custom { name, player, .. } if name == VOTED => *player,
            _ => None,
        })
        .collect()
}

/// P0 casts Plea for Power (each player votes for time or knowledge).
fn plea(t: &mut TestGame) {
    supported("Plea for Power");
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Plea for Power");
    t.cast(P0, spell).go();
    t.resolve_all();
}

#[test]
fn additional_votes_are_cumulative() {
    cr!("701.38d");
    ruling!(
        "Brago's Representative",
        "The ability is cumulative. For example, if you control two Brago’s Representatives, you’ll vote three times per vote."
    );
    ruling!(
        "Ballot Broker",
        "Ballot Broker’s ability is cumulative. If you control two of them, you can vote up to three times."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Brago's Representative");
    t.battlefield(P1, "Brago's Representative");
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P1, P1]);
    let mut t = TestGame::new(2);
    supported("Ballot Broker");
    t.battlefield(P1, "Ballot Broker");
    t.battlefield(P1, "Ballot Broker");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P1, P1]);
    // Up to: declining one.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ballot Broker");
    t.battlefield(P1, "Ballot Broker");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P1]);
}

/// Who was asked each "Vote" choice.
fn vote_choosers(t: &TestGame) -> Vec<PlayerId> {
    t.asked()
        .iter()
        .filter(|(_, d)| {
            matches!(d, mtg_engine::decision::Decision::ChooseOption { prompt, .. } if prompt == "Vote")
        })
        .map(|(p, _)| *p)
        .collect()
}

fn illusion(t: &mut TestGame, p: PlayerId) {
    let ic = t.hand(p, "Illusion of Choice");
    mana(t, p, ManaType::U, 1);
    t.cast(p, ic).go();
    t.resolve_all();
}

#[test]
fn illusion_of_choice_chooses_votes_but_not_whether_to_vote_again() {
    cr!("701.38a", "701.38d");
    ruling!(
        "Illusion of Choice",
        "If another player controls Ballot Broker, that player first takes their “normal” vote with you choosing the result, then that player decides whether they are taking the additional vote. If there is an additional vote, you again choose the result."
    );
    supported("Illusion of Choice");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ballot Broker");
    illusion(&mut t, P0);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    let from = t.asked().len();
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P1]);
    assert_eq!(vote_choosers(&t), vec![P0, P0, P0]);
    let extra: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(extra, vec![P1]);
    // Without it, each player chooses their own vote.
    let mut t = TestGame::new(2);
    plea(&mut t);
    assert_eq!(vote_choosers(&t), vec![P0, P1]);
}

#[test]
fn illusion_of_choice_most_recent_one_wins() {
    cr!("701.38a");
    ruling!(
        "Illusion of Choice",
        "If multiple players cast Illusion of Choice during the same turn, the controller of the one that resolved most recently will choose how each player votes that turn."
    );
    let mut t = TestGame::new(2);
    illusion(&mut t, P0);
    illusion(&mut t, P1);
    plea(&mut t);
    assert_eq!(vote_choosers(&t), vec![P1, P1]);
}

/// `p` casts Council's Judgment ("Starting with you, each player votes for a nonland
/// permanent you don't control. Exile each permanent with the most votes or tied for most
/// votes."); it isn't resolved yet.
fn judgment(t: &mut TestGame, p: PlayerId) {
    supported("Council's Judgment");
    let cj = t.hand(p, "Council's Judgment");
    mana(t, p, ManaType::W, 3);
    t.cast(p, cj).go();
}

#[test]
fn grudge_keeper_triggers_after_the_vote_even_if_it_leaves() {
    cr!("603.2", "603.10a", "701.38");
    ruling!(
        "Grudge Keeper",
        "The spell or ability that caused you to vote will finish resolving before Grudge Keeper’s ability is put on the stack. The ability will be put on the stack even if the results of the vote cause Grudge Keeper to leave the battlefield."
    );
    supported("Grudge Keeper");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let gk = t.battlefield(P0, "Grudge Keeper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P1, &[obj(gk)]);
    t.answer_choose(P0, &[obj(bears)]);
    judgment(&mut t, P1);
    t.resolve();
    // Both were exiled (tied); the spell finished; then the trigger.
    assert!(t.in_exile("Grudge Keeper") && t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Council's Judgment"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn grudge_keeper_considers_only_the_vote_that_triggered_it() {
    cr!("701.38", "603.2");
    ruling!(
        "Grudge Keeper",
        "Grudge Keeper’s ability considers only the vote that caused it to trigger, not any previous votes, even if they were caused by the same permanent."
    );
    // Options: 0 = time, 1 = knowledge.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grudge Keeper");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    plea(&mut t);
    assert_eq!(t.life(P1), 18);
    // The second vote: the same choices; P1 doesn't lose life again for the first vote.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    plea(&mut t);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn grudge_keeper_with_additional_votes() {
    cr!("701.38d");
    ruling!(
        "Grudge Keeper",
        "If one of your opponents has additional votes because of Brago’s Representative, and that player votes for more than one choice you didn’t vote for, that player will lose only 2 life."
    );
    ruling!(
        "Grudge Keeper",
        "If you have additional votes because of Brago’s Representative, and you vote for more than one choice, an opponent will lose 2 life only if they voted for none of those choices."
    );
    // P1 votes for two choices P0 didn't vote for: 2 life.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grudge Keeper");
    t.battlefield(P1, "Brago's Representative");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    let c = t.battlefield(P2, "Raging Goblin");
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P1, &[obj(b)]);
    t.answer_choose(P1, &[obj(c)]);
    t.answer_choose(P2, &[obj(a)]);
    judgment(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 20);
    // P0 votes twice: only an opponent who voted for neither loses life.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grudge Keeper");
    t.battlefield(P0, "Brago's Representative");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    let c = t.battlefield(P2, "Raging Goblin");
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    t.answer_choose(P1, &[obj(b)]);
    t.answer_choose(P2, &[obj(c)]);
    judgment(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 18);
}
