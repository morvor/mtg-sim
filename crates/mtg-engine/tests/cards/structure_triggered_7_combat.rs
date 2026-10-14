//! Structure-coverage tests, round 4 (item 7): combat triggered-ability structures shared
//! by several cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{act, target_choices};
use super::structure_seed_common::*;
use super::structure_triggered_5::{next_turn, obj};
use super::structure_triggered_7::tokens;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, Color};
use mtg_engine::*;

/// Runs a combat on P0's turn: `attackers` attack P1, who blocks with `blocks`.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let a: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&a, blocks);
    t.resolve_all();
}

// ---------------------------------------------------------------------------
// "Whenever ~ deals combat damage to a player, ..."
// ---------------------------------------------------------------------------

#[test]
fn glint_eye_nephilim_draws_that_many_cards() {
    cr!("510.3a", "603.2");
    // "Whenever this creature deals combat damage to a player, draw that many cards."
    supported("Glint-Eye Nephilim");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Glint-Eye Nephilim");
    fight(&mut t, &[n], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.library_size(P0), 28);
    assert_eq!(t.library_size(P1), 30);
    // Pumped, it draws more; blocked, it draws nothing.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Glint-Eye Nephilim");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(n).go();
    t.resolve_all();
    fight(&mut t, &[n], &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.hand_size(P0), 5);
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Glint-Eye Nephilim");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[n], &[(wall, n)]);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn living_hive_creates_that_many_insects() {
    cr!("510.3a", "603.2", "111.1", "702.19b");
    // "Trample / Whenever this creature deals combat damage to a player, create that many
    // 1/1 green Insect creature tokens."
    supported("Living Hive");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Living Hive");
    fight(&mut t, &[h], &[]);
    let ins = tokens(&t, P0, "Insect");
    assert_eq!(ins.len(), 6);
    assert_eq!(t.pt(ins[0]), (1, 1));
    assert!(t.obj_now(ins[0]).chars.colors.contains(Color::Green));
    assert_eq!(tokens(&t, P1, "Insect").len(), 0);
    // Trampling over a 2/2 blocker: 4 damage to the player, 4 Insects.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Living Hive");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[h], &[(bears, h)]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(tokens(&t, P0, "Insect").len(), 4);
}

#[test]
fn precinct_captain_creates_a_soldier() {
    cr!("510.3a", "603.2", "111.1");
    // "First strike / Whenever this creature deals combat damage to a player, create a 1/1
    // white Soldier creature token."
    supported("Precinct Captain");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Precinct Captain");
    fight(&mut t, &[c], &[]);
    let s = tokens(&t, P0, "Soldier");
    assert_eq!(s.len(), 1, "one token, not one per damage");
    assert_eq!(t.pt(s[0]), (1, 1));
    assert!(t.obj_now(s[0]).chars.colors.contains(Color::White));
    // Blocked: no token.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Precinct Captain");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[c], &[(wall, c)]);
    assert!(tokens(&t, P0, "Soldier").is_empty());
}

#[test]
fn snooping_page_draws_and_loses_1_life() {
    cr!("510.3a", "603.2", "119.3");
    // "Whenever this creature deals combat damage to a player, you draw a card and lose 1
    // life."
    supported("Snooping Page");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Snooping Page");
    fight(&mut t, &[p], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Snooping Page");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[p], &[(wall, p)]);
    assert_eq!((t.life(P0), t.hand_size(P0)), (20, 0));
}

#[test]
fn aurora_shifter_gets_that_much_energy() {
    cr!("510.3a", "603.2", "122.1");
    // "Whenever this creature deals combat damage to a player, you get that many {E}."
    supported("Aurora Shifter");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Aurora Shifter");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(s).go();
    t.resolve_all();
    fight(&mut t, &[s], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 4);
    assert_eq!(t.g.player(P1).counter(counters::ENERGY), 0);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Aurora Shifter");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[s], &[(wall, s)]);
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 0);
}

#[test]
fn invasion_tactics_draws_once_when_allies_deal_combat_damage() {
    cr!("510.3a", "603.2", "603.2c");
    // "Whenever one or more Allies you control deal combat damage to a player, draw a
    // card."
    supported("Invasion Tactics");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Invasion Tactics");
    let a = t.battlefield(P0, "Makindi Patrol");
    let b = t.battlefield(P0, "Makindi Patrol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[a, b, bears], &[]);
    assert_eq!(t.life(P1), 20 - 2 - 2 - 2);
    assert_eq!(t.hand_size(P0), 1, "one card for both Allies");
    // A non-Ally alone doesn't draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Invasion Tactics");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Makindi Patrol");
    fight(&mut t, &[bears], &[]);
    assert_eq!(t.hand_size(P0), 0);
    // An opponent's Ally doesn't either.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Invasion Tactics");
    let theirs = t.battlefield(P1, "Makindi Patrol");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(theirs, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0) + t.hand_size(P1), 0);
}

#[test]
fn reconnaissance_mission_may_draw_for_each_creature() {
    cr!("510.3a", "603.2", "603.5");
    // "Whenever a creature you control deals combat damage to a player, you may draw a
    // card."
    supported("Reconnaissance Mission");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reconnaissance Mission");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    fight(&mut t, &[a, b], &[]);
    assert_eq!(t.hand_size(P0), 1, "one yes, one no");
    // An opponent's creature: no trigger, so the queued "yes" stays unused.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reconnaissance Mission");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(theirs, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0), 0);
}

// ---------------------------------------------------------------------------
// "Whenever ~ deals combat damage to a creature, ..."
// ---------------------------------------------------------------------------

#[test]
fn ohran_viper_destroys_the_creature_at_end_of_combat() {
    cr!("510.3a", "603.2", "603.7a", "511.1", "701.8a");
    // "Whenever this creature deals combat damage to a creature, destroy that creature at
    // end of combat."
    supported("Ohran Viper");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Ohran Viper");
    let wall = t.battlefield(P1, "Wall of Wood");
    let other = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(v, Entity::Player(P1))]);
    block(&mut t, P1, &[(wall, v)]);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert!(t.on_battlefield(wall), "not until end of combat");
    assert_eq!(t.obj_now(wall).damage, 1);
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(wall));
    assert!(t.on_battlefield(other) && t.on_battlefield(v));
    // Blocking: it destroys the attacker it damaged.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P1, "Ohran Viper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[(v, bears)]);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(v));
}

#[test]
fn dripping_dead_destroys_the_creature_it_damages_without_regeneration() {
    cr!("510.3a", "603.2", "701.8a", "701.19c", "608.2c");
    // "Whenever this creature deals combat damage to a creature, destroy that creature. It
    // can't be regenerated."
    supported("Dripping Dead");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dripping Dead");
    let wall = t.battlefield(P1, "Wall of Stone");
    let other = t.battlefield(P1, "Wall of Stone");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(d, Entity::Player(P1))]);
    block(&mut t, P1, &[(wall, d)]);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert!(!t.on_battlefield(wall), "4 damage to a 0/8, then destroyed");
    assert!(t.on_battlefield(other) && t.on_battlefield(d));
    // A regeneration shield doesn't save it.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dripping Dead");
    let sk = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(d, Entity::Player(P1))]);
    block(&mut t, P1, &[(sk, d)]);
    go_to(&mut t, Step::DeclareBlockers);
    for _ in 0..2 {
        act(&mut t, P1, sk, "regenerate", &[]).unwrap();
        t.resolve_all();
    }
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert!(!t.on_battlefield(sk), "regenerated from lethal damage, then destroyed");
    assert_eq!(t.zone(sk), Zone::Graveyard(P1));
}

// ---------------------------------------------------------------------------
// "Whenever ~ attacks, ..."
// ---------------------------------------------------------------------------

#[test]
fn pegasus_courser_gives_another_attacker_flying() {
    cr!("508.1m", "115.1", "611.2a", "514.2");
    // "Whenever this creature attacks, another target attacking creature gains flying until
    // end of turn."
    supported("Pegasus Courser");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pegasus Courser");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(p, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(home).has_keyword(KeywordKind::Flying));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(p)), "another");
    assert!(!offered.contains(&obj(home)), "attacking");
    assert!(!offered.contains(&obj(theirs)));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn territorial_hammerskull_taps_an_opponents_creature_when_it_attacks() {
    cr!("508.1m", "115.1", "701.26a");
    // "Whenever this creature attacks, tap target creature an opponent controls."
    supported("Territorial Hammerskull");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Territorial Hammerskull");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(a)]);
    fight(&mut t, &[h], &[]);
    assert!(t.obj_now(a).tapped && !t.obj_now(b).tapped);
    assert_eq!(t.life(P1), 18, "the tapped creature couldn't block");
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(b)));
    assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(h)));
    assert!(!offered.contains(&obj(forest)));
    // Another attacker doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Territorial Hammerskull");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a)]);
    fight(&mut t, &[mine], &[]);
    assert!(!t.obj_now(a).tapped);
}
