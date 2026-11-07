//! Attack and block restrictions, requirements and taxes on the declaration (CR 508.1c-d,
//! 508.6, 509.1b-d, 609.4): Scarred Puma, Okk, Tromokratis, Bloodthirster, Hipparion,
//! Heat Wave, Awesome Presence, Seeker of Slaanesh, Trove of Temptation, Invasion Plans,
//! Dragon Hunter, Street Savvy, Aether Web.

use crate::combat2_common::*;
use mtg_engine::combat::block_costs_by_player;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn multi(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

fn blocks_now(t: &TestGame) -> Vec<(ObjectId, ObjectId)> {
    t.g.combat
        .as_ref()
        .map(|c| {
            c.blockers
                .iter()
                .flat_map(|b| b.blocking.iter().map(move |a| (b.id, *a)))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn scarred_puma_attacks_only_with_a_black_or_green_creature() {
    cr!("508.1c");
    ruling!(
        "Scarred Puma",
        "It means “a black or green creature other than itself attacks”."
    );
    supported("Scarred Puma");
    let mut t = TestGame::new(2);
    let puma = t.battlefield(P0, "Scarred Puma");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let knight = t.battlefield(P0, "Savannah Lions");
    to_combat(&mut t, P0);
    let p1 = Entity::Player(P1);
    assert!(!legal_attack(&mut t, &[(puma, p1)]));
    assert!(!legal_attack(&mut t, &[(puma, p1), (knight, p1)]));
    assert!(legal_attack(&mut t, &[(puma, p1), (bears, p1)]));
    assert!(legal_attack(&mut t, &[(bears, p1)]));
}

#[test]
fn okk_needs_a_creature_with_greater_power_to_attack_or_block() {
    cr!("508.1c", "509.1b");
    supported("Okk");
    let mut t = TestGame::new(2);
    let okk = t.battlefield(P0, "Okk");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    to_combat(&mut t, P0);
    let p1 = Entity::Player(P1);
    assert!(!legal_attack(&mut t, &[(okk, p1)]));
    assert!(!legal_attack(&mut t, &[(okk, p1), (giant, p1)]));
    assert!(legal_attack(&mut t, &[(okk, p1), (wurm, p1)]));
    // Blocking: P1 attacks with two creatures.
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let a2 = t.battlefield(P1, "Grizzly Bears");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    declare(
        &mut t,
        P1,
        &[(a1, Entity::Player(P0)), (a2, Entity::Player(P0))],
    );
    assert!(!legal_blocks(&mut t, &[(okk, a1)]));
    assert!(!legal_blocks(&mut t, &[(okk, a1), (giant, a2)]));
    assert!(legal_blocks(&mut t, &[(okk, a1), (wurm, a2)]));
}

#[test]
fn tromokratis_must_be_blocked_by_all_creatures_defending_player_controls() {
    cr!("509.1b");
    ruling!(
        "Tromokratis",
        "The defending player may block Tromokratis with each creature they control. If that player can't, perhaps because one of those creatures is tapped, or chooses not to, Tromokratis can't be blocked."
    );
    supported("Tromokratis");
    let mut t = TestGame::new(2);
    let tromo = t.battlefield(P0, "Tromokratis");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(tromo, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, &[(b1, tromo)]));
    assert!(legal_blocks(&mut t, &[(b1, tromo), (b2, tromo)]));
    // One of them is tapped: it can't be blocked at all.
    t.g.tap(b2);
    assert!(!legal_blocks(&mut t, &[(b1, tromo)]));
}

#[test]
fn bloodthirster_cant_attack_the_same_player_twice_in_a_turn() {
    cr!("508.1c", "508.6", "500.8");
    supported("Bloodthirster");
    let mut t = multi(3);
    let bt = t.battlefield(P0, "Bloodthirster");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(bt, Entity::Player(P1))]);
    // Combat damage: untap it, and an additional combat phase.
    t.advance_to(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.life(P1), 14);
    assert!(!t.obj_now(bt).tapped);
    assert!(!legal_attack(&mut t, &[(bt, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[(bt, Entity::Player(P2))]));
}

#[test]
fn hipparion_pays_to_block_big_creatures() {
    cr!("509.1c", "509.1d");
    supported("Hipparion");
    let mut t = TestGame::new(2);
    let hipp = t.battlefield(P1, "Hipparion");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert!(block_costs_by_player(&t.g, &[(hipp, bears)]).is_empty());
    let costs = block_costs_by_player(&t.g, &[(hipp, giant)]);
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].0, P1);
    assert_eq!(costs[0].1.mana.as_ref().map(|m| m.mana_value()), Some(1));
    // With a land, P1 pays and blocks the Giant.
    let land = t.lands(P1, "Plains", 1)[0];
    declare_blocks(&mut t, P1, &[(hipp, giant)]);
    assert_eq!(blocks_now(&t), vec![(hipp, giant)]);
    assert!(t.obj_now(land).tapped);
}

#[test]
fn hipparion_cant_block_a_big_creature_without_paying() {
    cr!("509.1d");
    let mut t = TestGame::new(2);
    let hipp = t.battlefield(P1, "Hipparion");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    // The block is legal but its cost can't be paid: it doesn't happen.
    let costs = block_costs_by_player(&t.g, &[(hipp, giant)]);
    assert!(!t
        .g
        .can_pay_cost(P1, &costs[0].1, None, &mtg_engine::eval::Ctx::new(None, P1)));
    declare_blocks(&mut t, P1, &[(hipp, giant)]);
    assert!(blocks_now(&t).is_empty());
}

#[test]
fn heat_wave_charges_life_for_each_nonblue_blocker() {
    cr!("509.1c", "509.1d", "119.4");
    supported("Heat Wave");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heat Wave");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    let blue = t.battlefield(P1, "Coral Merfolk");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    // Blue creatures can't block creatures P0 controls.
    assert!(!legal_blocks(&mut t, &[(blue, a)]));
    declare_blocks(&mut t, P1, &[(w1, a), (w2, b)]);
    assert_eq!(blocks_now(&t).len(), 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn awesome_presence_charges_for_each_creature_blocking_it() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Awesome Presence",
        "In the Two-Headed Giant format, you still only have to pay once per creature."
    );
    supported("Awesome Presence");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Awesome Presence");
    assert!(t.g.attach(aura, Entity::Object(a)));
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    let costs = block_costs_by_player(&t.g, &[(w1, a), (w2, a)]);
    assert_eq!(costs[0].1.mana.as_ref().map(|m| m.mana_value()), Some(6));
    // Blocking the other attacker is free.
    assert!(block_costs_by_player(&t.g, &[(w1, other)]).is_empty());
    let lands = t.lands(P1, "Island", 3);
    declare_blocks(&mut t, P1, &[(w1, a)]);
    assert_eq!(blocks_now(&t), vec![(w1, a)]);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn seeker_of_slaanesh_makes_each_opponent_attack_with_a_creature() {
    cr!("508.1d");
    ruling!(
        "Seeker of Slaanesh",
        "If no creatures a player controls are able to attack, Seeker of Slaanesh's requirement has no effect during that combat."
    );
    supported("Seeker of Slaanesh");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Seeker of Slaanesh");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(a, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(b, Entity::Player(P0))]));
    // None able to attack: nothing required.
    t.g.tap(a);
    t.g.tap(b);
    assert!(legal_attack(&mut t, &[]));
    // Seeker's controller isn't affected.
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn trove_of_temptation_requires_one_attack_on_its_controller() {
    cr!("508.1d");
    ruling!(
        "Trove of Temptation",
        "Each opponent only has to attack you or a planeswalker you control with one creature total"
    );
    supported("Trove of Temptation");
    let mut t = multi(3);
    t.battlefield(P0, "Trove of Temptation");
    let pw = t.battlefield(P0, "Jace Beleren");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(a, Entity::Player(P2))]));
    assert!(legal_attack(
        &mut t,
        &[(a, Entity::Player(P0)), (b, Entity::Player(P2))]
    ));
    assert!(legal_attack(&mut t, &[(b, Entity::Object(pw))]));
}

#[test]
fn invasion_plans_lets_the_attacking_player_choose_blocks() {
    cr!("509.1", "509.1c");
    supported("Invasion Plans");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Invasion Plans");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let w = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    // P0, the attacking player, declares P1's blocks.
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(w, b)]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(w, a)]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocks_now(&t), vec![(w, b)]);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::DeclareBlockers { .. })));
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::DeclareBlockers { .. })));
}

#[test]
fn dragon_hunter_blocks_flying_dragons_only() {
    cr!("509.1b", "702.9b", "609.4");
    supported("Dragon Hunter");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P1, "Dragon Hunter");
    let dragon = t.battlefield(P0, "Shivan Dragon");
    let angel = t.battlefield(P0, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(dragon, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    assert!(legal_blocks(&mut t, &[(hunter, dragon)]));
    assert!(!legal_blocks(&mut t, &[(hunter, angel)]));
}

#[test]
fn street_savvy_blocks_landwalkers() {
    cr!("509.1b", "702.14c", "609.4");
    ruling!(
        "Street Savvy",
        "“Landwalk abilities” are abilities such as “forestwalk” or “legendary landwalk” that have “walk” in the name."
    );
    supported("Street Savvy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Mutavault");
    let aura = t.battlefield(P1, "Street Savvy");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let walker = t.battlefield(P0, "Dryad Sophisticate");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(walker, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, &[(bears, walker)]));
    assert!(!legal_blocks(&mut t, &[(other, walker)]));
    assert_eq!(t.pt(bears), (2, 4));
}

#[test]
fn aether_web_blocks_creatures_with_shadow() {
    cr!("509.1b", "702.28b", "609.4");
    ruling!(
        "Aether Web",
        "The enchanted creature can block creatures with shadow and creatures without shadow."
    );
    supported("Aether Web");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Aether Web");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let lancer = t.battlefield(P0, "Soltari Lancer");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(lancer, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert!(legal_blocks(&mut t, &[(bears, lancer)]));
    assert!(legal_blocks(&mut t, &[(bears, giant)]));
    assert!(!legal_blocks(&mut t, &[(other, lancer)]));
}

#[test]
fn aether_web_on_a_creature_with_shadow_blocks_nothing() {
    cr!("702.28b", "609.4");
    ruling!(
        "Aether Web",
        "If the enchanted creature has shadow, it won’t be able to block any creatures (not even those with shadow)."
    );
    let mut t = TestGame::new(2);
    let shadow = t.battlefield(P1, "Soltari Lancer");
    let aura = t.battlefield(P1, "Aether Web");
    assert!(t.g.attach(aura, Entity::Object(shadow)));
    let lancer = t.battlefield(P0, "Soltari Lancer");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(lancer, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert!(!legal_blocks(&mut t, &[(shadow, lancer)]));
    assert!(!legal_blocks(&mut t, &[(shadow, giant)]));
}
