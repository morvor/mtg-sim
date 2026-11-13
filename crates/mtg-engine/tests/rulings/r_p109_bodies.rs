//! Rulings batch P109 — several bodies and several copies: "creatures you control get"
//! affects only creatures there as it resolves (CR 611.2c); an Aura spell with an illegal
//! target (CR 608.3b); excess damage with deathtouch (CR 120.4a, 702.2c); batch token
//! creation triggers (CR 603.2); Masterful Replication's copy effect (CR 707.2, 707.3,
//! 301.5c, 111.1); Drana and Linvala's gained abilities (CR 113.10a, 201.5).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_p017_common::{animate, enter_copying};
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s02_common::{can_activate, create_token};
use crate::r_s13_common::add;
use crate::r_s25_common::lands_for_cost;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{AbilityKind, Modification};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn hasty(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Haste)
}

// --- Creatures you control at the time ---------------------------------------------------

#[test]
fn creatures_you_control_get_affects_only_creatures_there_then() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Goblin War Party",
        "Goblin War Party’s second mode affects only creatures you control at the time the spell resolves. Creatures you begin to control later in the turn won’t get +1/+1 or gain haste."
    );
    supported("Goblin War Party");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Goblin War Party");
    let gwp = t.hand(P0, "Goblin War Party");
    t.cast(P0, gwp).modes(&[1]).go();
    t.resolve_all();
    let new = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(old), (3, 3));
    assert!(hasty(&t, old));
    assert_eq!(t.pt(new), (2, 2));
    assert!(!hasty(&t, new));

    ruling!(
        "Imodane's Recruiter // Train Troops",
        "Imodane's Recruiter's triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+0 or gain haste."
    );
    supported("Imodane's Recruiter // Train Troops");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Grizzly Bears");
    let r = t.enter(P0, "Imodane's Recruiter // Train Troops");
    // One that arrives while the trigger is on the stack is there as it resolves.
    let mid = t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    let new = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(old), (3, 2));
    assert_eq!(t.pt(mid), (3, 2));
    assert!(hasty(&t, r) && hasty(&t, mid));
    assert_eq!(t.pt(new), (2, 2));
    assert!(!hasty(&t, new));

    ruling!(
        "Heroic Reinforcements",
        "The second part of Heroic Reinforcements's effect affects only creatures you control at the time it's applied, including the tokens created by the first part of its effect. Creatures you begin to control later in the turn won't get +1/+1 or gain haste."
    );
    supported("Heroic Reinforcements");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Heroic Reinforcements");
    let hr = t.hand(P0, "Heroic Reinforcements");
    t.cast(P0, hr).go();
    t.resolve_all();
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    t.g.recompute();
    for s in &soldiers {
        assert_eq!(t.pt(*s), (2, 2), "the new tokens are included");
        assert!(hasty(&t, *s));
    }
    let new = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(new), (2, 2));
    assert!(!hasty(&t, new));
}

// --- Beck // Call and Join the Ranks ---------------------------------------------------

#[test]
fn beck_triggers_on_any_creature_entering() {
    cr!("603.2", "709.3");
    ruling!(
        "Beck // Call",
        "Beck triggers when any creature enters the battlefield, no matter who controls it."
    );
    supported("Beck // Call");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let bc = t.hand(P0, "Beck // Call");
    t.cast(P0, bc).method(CastMethod::Half(0)).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "an opponent's creature entering");
}

#[test]
fn join_the_ranks_triggers_ally_abilities_twice() {
    cr!("603.2", "603.3");
    ruling!(
        "Join the Ranks",
        "When a Join the Ranks you control resolves, each ability that triggers whenever an Ally enters under your control will trigger twice."
    );
    supported("Join the Ranks");
    supported("Kalastria Healer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalastria Healer");
    lands_for_cost(&mut t, P0, "Join the Ranks");
    let jr = t.hand(P0, "Join the Ranks");
    t.cast(P0, jr).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 2, "one trigger per Ally");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

// --- Fists of Ironwood ------------------------------------------------------------------

#[test]
fn fists_of_ironwood_with_an_illegal_target_creates_nothing() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Fists of Ironwood",
        "If the target creature is an illegal target when Fists of Ironwood tries to resolve, Fists of Ironwood won't resolve and won't enter the battlefield. You won't create Saproling tokens."
    );
    supported("Fists of Ironwood");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Fists of Ironwood");
    let f = t.hand(P0, "Fists of Ironwood");
    t.cast(P0, f).target(bears).go();
    kill(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fists of Ironwood"));
    assert!(with_subtype(&t, P0, "Saproling").is_empty());
}

#[test]
fn fists_of_ironwood_tokens_come_after_it_is_attached() {
    cr!("303.4a", "603.6a", "111.2");
    ruling!(
        "Fists of Ironwood",
        "The Saproling tokens aren't created until after Fists of Ironwood is on the battlefield, so you can't cast Fists of Ironwood on one of those tokens."
    );
    ruling!(
        "Fists of Ironwood",
        "You create the Saproling tokens, even if Fists of Ironwood enters the battlefield attached to another player's creature."
    );
    supported("Fists of Ironwood");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Fists of Ironwood");
    let f = t.hand(P0, "Fists of Ironwood");
    let before = t.asked().len();
    t.cast(P0, f).target(bears).go();
    // No Saproling among the candidates: they don't exist yet.
    assert!(with_subtype(&t, P0, "Saproling").is_empty());
    assert!(t.asked()[before..].iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { candidates, .. }
            if candidates.contains(&obj(bears))
    )));
    t.resolve_all();
    let fists = t.named_on_battlefield("Fists of Ironwood")[0];
    assert_eq!(t.obj(fists).attached_to, Some(obj(bears)));
    let saps = with_subtype(&t, P0, "Saproling");
    assert_eq!(saps.len(), 2);
    assert!(saps.iter().all(|s| t.obj(*s).controller == P0));
}

// --- Goblin Negotiation -----------------------------------------------------------------

#[test]
fn goblin_negotiation_with_deathtouch_counts_excess_beyond_one() {
    cr!("120.4a", "702.2c");
    ruling!(
        "Goblin Negotiation",
        "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage, so any amount greater than that will cause excess damage to be dealt, even if the total amount of damage isn't greater than the creature's toughness"
    );
    supported("Goblin Negotiation");
    supported("Pestilent Spirit");
    // X = 2 to a Hill Giant (3/3): with deathtouch (Pestilent Spirit), 1 is excess.
    for (spirit, goblins) in [(true, 1usize), (false, 0)] {
        let mut t = TestGame::new(2);
        if spirit {
            t.battlefield(P0, "Pestilent Spirit");
        }
        let giant = t.battlefield(P1, "Hill Giant");
        lands_for_cost(&mut t, P0, "Goblin Negotiation");
        t.lands(P0, "Mountain", 2);
        let gn = t.hand(P0, "Goblin Negotiation");
        t.cast(P0, gn).x(2).target(giant).go();
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Goblin").len(), goblins, "spirit {spirit}");
        assert_eq!(!t.on_battlefield(giant), spirit);
    }
}

// --- Masterful Replication --------------------------------------------------------------

/// Casts Masterful Replication's second mode targeting `target`.
fn replicate(t: &mut TestGame, target: ObjectId) {
    supported("Masterful Replication");
    lands_for_cost(t, P0, "Masterful Replication");
    let mr = t.hand(P0, "Masterful Replication");
    let target = t.g.current(target);
    t.cast(P0, mr).modes(&[1]).target(target).go();
    t.resolve_all();
    t.g.recompute();
}

fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

#[test]
fn masterful_replication_equipment_copies() {
    cr!("707.2", "301.5c", "704.5n");
    ruling!(
        "Masterful Replication",
        "An Equipment that becomes a copy of a creature becomes unattached if it was attached to a creature. An Equipment that becomes a copy of another Equipment remains attached if it was attached to a creature."
    );
    // A copy of Ornithopter (an artifact creature): unattached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P0, "Ornithopter");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.g.attach(eq, obj(bears));
    replicate(&mut t, thopter);
    assert_eq!(name(&t, eq), "Ornithopter");
    assert!(t.obj_now(eq).is(CardType::Creature));
    assert_eq!(t.obj_now(eq).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
    // A copy of Short Sword (another Equipment): still attached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Short Sword");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.g.attach(eq, obj(bears));
    replicate(&mut t, sword);
    assert_eq!(name(&t, eq), "Short Sword");
    assert_eq!(t.obj_now(eq).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn masterful_replication_copies_keep_token_status_and_copiable_values() {
    cr!("707.2", "707.3", "111.1", "613.2");
    ruling!(
        "Masterful Replication",
        "Becoming a copy won't change whether an artifact is a token."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let clue = create_token(&mut t, P0, "Clue");
    let card = t.battlefield(P0, "Short Sword");
    replicate(&mut t, thopter);
    assert_eq!(name(&t, clue), "Ornithopter");
    assert!(t.obj_now(clue).is_token());
    assert_eq!(name(&t, card), "Ornithopter");
    assert!(!t.obj_now(card).is_token());

    ruling!(
        "Masterful Replication",
        "If the target artifact is copying something else, then your artifacts become copies of whatever the target artifact copied."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let steel = enter_copying(&mut t, P0, "Sculpting Steel", Some(thopter));
    assert_eq!(name(&t, steel), "Ornithopter");
    let sword = t.battlefield(P0, "Short Sword");
    replicate(&mut t, steel);
    assert_eq!(name(&t, sword), "Ornithopter");
    assert_eq!(t.pt(sword), (0, 2));

    ruling!(
        "Masterful Replication",
        "Masterful Replication copies the printed values of the target artifact. It won't copy counters on that artifact or effects that have changed its power, toughness, types, color, or so on. Notably, it won't copy effects that made the target artifact become an artifact creature if it isn't normally an artifact creature."
    );
    // Counters and a pump on the target aren't copied.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    add(&mut t, thopter, "+1/+1", 2);
    modify_until_eot(
        &mut t,
        thopter,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Trample))],
    );
    let sword = t.battlefield(P0, "Short Sword");
    replicate(&mut t, thopter);
    assert_eq!(t.pt(thopter), (2, 4));
    assert_eq!(t.pt(sword), (0, 2));
    assert_eq!(t.counters(sword, "+1/+1"), 0);
    assert!(!t.obj_now(sword).has_keyword(KeywordKind::Trample));
    // An animated Short Sword: the copies aren't creatures.
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    animate(&mut t, sword, 3);
    let other = t.battlefield(P0, "Bonesplitter");
    replicate(&mut t, sword);
    assert!(t.obj_now(sword).is(CardType::Creature));
    assert_eq!(name(&t, other), "Short Sword");
    assert!(!t.obj_now(other).is(CardType::Creature));
}

// --- Drana and Linvala ------------------------------------------------------------------

fn activated_with(t: &mut TestGame, id: ObjectId, needle: &str) -> Vec<usize> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .enumerate()
        .filter(|(_, a)| a.text.contains(needle))
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn drana_and_linvala_gained_abilities_refer_to_itself() {
    cr!("113.10a", "201.5");
    ruling!(
        "Drana and Linvala",
        "If an activated ability references the creature that has it by name, treat Drana and Linvala’s version of that ability as though it referenced Drana and Linvala by name instead."
    );
    supported("Drana and Linvala");
    supported("Onakke Javelineer");
    // P1's Onakke Javelineer: "{T}: This creature deals 2 damage to target player or
    // battle." D&L (with lifelink here) deals the damage and taps itself.
    let mut t = TestGame::new(2);
    let dl = t.battlefield(P0, "Drana and Linvala");
    let jav = t.battlefield(P1, "Onakke Javelineer");
    modify_until_eot(
        &mut t,
        dl,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Lifelink))],
    );
    let i = activated_with(&mut t, dl, "2 damage")[0];
    t.activate(P0, dl, i, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22, "Drana and Linvala dealt the damage");
    assert!(t.obj_now(dl).tapped);
    assert!(!t.obj_now(jav).tapped);
}

#[test]
fn two_dranas_under_different_players_activate_nothing() {
    cr!("113.10a", "602.5");
    ruling!(
        "Drana and Linvala",
        "If multiple players control a Drana and Linvala, exactly what abilities each of them has can get complex, based on what order they entered the battlefield. However, as none of those abilities can be activated"
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Drana and Linvala");
    let b = t.battlefield(P1, "Drana and Linvala");
    t.battlefield(P1, "Onakke Javelineer");
    t.battlefield(P0, "Onakke Javelineer");
    t.lands(P0, "Mountain", 3);
    t.lands(P1, "Mountain", 3);
    assert!(!activated_with(&mut t, a, "2 damage").is_empty());
    assert!(!can_activate(&mut t, P0, a));
    assert!(!can_activate(&mut t, P1, b));
}
