//! CR 508.1c–d, 508.1h, 509.1b–d audit: requirements are maximized without breaking
//! restrictions and without paying costs. A tax on attacking a player (Ghostly Prison,
//! Propaganda) doesn't tax attacking that player's planeswalkers, so a creature that must
//! attack attacks the planeswalker; a tax on both (Norn's Annex, Sphere of Safety) frees
//! it; a goaded creature attacks its goader rather than pay to attack another player;
//! "can't attack/block alone" makes another creature attack or block with a creature
//! that must; costs to block release Lure's requirement.

use crate::r506_audit_common::*;
use crate::r506_common::{block, declare, go_to};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's turn, beginning of combat.
fn combat_of_p0(t: &mut TestGame) {
    t.set_step(P0, Step::BeginningOfCombat);
}

#[test]
fn a_creature_that_must_attack_attacks_a_planeswalker_ghostly_prison_doesnt_tax() {
    cr!("508.1d", "508.1b");
    ruling!(
        "Ghostly Prison",
        "a creature that can't attack you can still attack a planeswalker you control"
    );
    supported("Ghostly Prison");
    supported("Juggernaut");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghostly Prison");
    let jace = t.battlefield(P1, "Jace Beleren");
    let jugg = t.battlefield(P0, "Juggernaut");
    combat_of_p0(&mut t);
    // No attack: illegal, an attack on Jace obeys the requirement for free.
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacks(&t, jugg), Some(Entity::Object(jace)));
}

#[test]
fn a_creature_that_must_attack_may_pay_to_attack_the_player_instead_of_the_planeswalker() {
    cr!("508.1d", "508.1h", "508.1j");
    supported("Ghostly Prison");
    supported("Juggernaut");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghostly Prison");
    t.battlefield(P1, "Jace Beleren");
    let jugg = t.battlefield(P0, "Juggernaut");
    let plains = t.lands(P0, "Plains", 2);
    combat_of_p0(&mut t);
    declare(&mut t, &[(jugg, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacks(&t, jugg), Some(Entity::Player(P1)));
    assert!(plains.iter().all(|p| t.obj_now(*p).tapped));
}

#[test]
fn a_creature_that_must_attack_neednt_pay_norns_annex_or_sphere_of_safety() {
    cr!("508.1d");
    ruling!(
        "Juggernaut",
        "If there's a cost associated with having it attack, you're not forced to pay that cost"
    );
    ruling!(
        "Sphere of Safety",
        "your opponents can choose not to attack with a creature with an ability that says it attacks if able"
    );
    for tax in ["Norn's Annex", "Sphere of Safety"] {
        supported(tax);
        let mut t = TestGame::new(2);
        t.battlefield(P1, tax);
        t.battlefield(P1, "Jace Beleren");
        let jugg = t.battlefield(P0, "Juggernaut");
        t.lands(P0, "Plains", 2);
        combat_of_p0(&mut t);
        go_to(&mut t, Step::DeclareAttackers);
        assert!(!t.g.is_attacking(jugg));
        // Attacking the player or Jace needs a cost: Juggernaut stays home, and the mana
        // stays untapped.
        assert!(attackers(&t).is_empty(), "{tax}");
        assert_eq!(t.life(P1), 20);
    }
}

/// P1 activates Gideon Jura's +2 targeting P0, then P0's turn reaches beginning of combat.
fn gideon_forces_attacks(tax: &str) -> (TestGame, ObjectId, ObjectId) {
    supported("Gideon Jura");
    supported(tax);
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let gideon = t.battlefield(P1, "Gideon Jura");
    t.battlefield(P1, tax);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P1, gideon, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    t.advance_to(P0, Step::BeginningOfCombat);
    (t, gideon, bears)
}

#[test]
fn gideon_juras_requirement_is_obeyed_under_ghostly_prison() {
    cr!("508.1d");
    ruling!(
        "Gideon Jura",
        "Gideon Jura's first ability causes creatures to attack him if able."
    );
    let (mut t, gideon, bears) = gideon_forces_attacks("Ghostly Prison");
    // Attacking P1 needs {2} (P0 has it): not a way to obey Gideon's requirement.
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacks(&t, bears), Some(Entity::Object(gideon)));
}

#[test]
fn gideon_juras_requirement_needs_no_payment_under_norns_annex() {
    cr!("508.1d");
    ruling!(
        "Gideon Jura",
        "If there's a cost associated with having a creature attack, the player isn't forced to pay that cost"
    );
    let (mut t, _, _) = gideon_forces_attacks("Norn's Annex");
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attackers(&t).is_empty());
}

/// A three-player game where all opponents are defending players: P1 goads P0's Bears
/// with Disrupt Decorum; P2 controls Propaganda. Returns the game at P0's beginning of
/// combat and the Bears.
fn goaded_bears_and_propaganda() -> (TestGame, ObjectId) {
    supported("Disrupt Decorum");
    supported("Propaganda");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P2, "Propaganda");
    t.lands(P0, "Island", 2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 4);
    let decorum = t.hand(P1, "Disrupt Decorum");
    t.cast(P1, decorum).go();
    t.resolve();
    assert_eq!(t.g.goaders(bears), vec![P1]);
    t.advance_to(P0, Step::BeginningOfCombat);
    (t, bears)
}

#[test]
fn a_goaded_creature_attacks_its_goader_rather_than_pay_to_attack_another_player() {
    cr!("508.1d", "701.15b", "802.3a");
    ruling!(
        "Disrupt Decorum",
        "If there's a cost associated with having a creature attack a player, its controller isn't forced to pay that cost, so it doesn't have to attack that player."
    );
    let (mut t, bears) = goaded_bears_and_propaganda();
    // Not attacking at all disobeys "attacks each combat if able", which attacking P1
    // obeys for free.
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacks(&t, bears), Some(Entity::Player(P1)));
    assert_eq!(t.life(P2), 20);
}

#[test]
fn a_goaded_creature_may_pay_to_attack_a_player_other_than_its_goader() {
    cr!("508.1d", "508.1h", "701.15b");
    let (mut t, bears) = goaded_bears_and_propaganda();
    declare(&mut t, &[(bears, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    // Paying obeys both goad requirements: legal.
    assert_eq!(attacks(&t, bears), Some(Entity::Player(P2)));
    assert!(t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.name == "Island")
        .all(|o| t.g.obj(*o).tapped));
}

#[test]
fn mogg_flunkies_that_must_attack_attacks_with_another_creature() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Mogg Flunkies",
        "If an effect says that Mogg Flunkies attacks or blocks if able and you control another creature able to attack or block, you must attack or block with Mogg Flunkies and that creature."
    );
    supported("Mogg Flunkies");
    supported("Basandra, Battle Seraph");
    for decl in [0usize, 1, 2] {
        let mut t = TestGame::new(2);
        let flunkies = t.battlefield(P0, "Mogg Flunkies");
        let bears = t.battlefield(P0, "Grizzly Bears");
        // P1's Basandra: "{R}: Target creature attacks this turn if able."
        let basandra = t.battlefield(P1, "Basandra, Battle Seraph");
        t.lands(P1, "Mountain", 1);
        t.activate(P1, basandra, 0, &[Entity::Object(flunkies)])
            .unwrap();
        t.resolve();
        combat_of_p0(&mut t);
        // Nothing, Flunkies alone (a restriction) or the Bears alone (no requirement
        // obeyed): each is illegal.
        let d: Vec<(ObjectId, Entity)> = match decl {
            0 => vec![],
            1 => vec![(flunkies, Entity::Player(P1))],
            _ => vec![(bears, Entity::Player(P1))],
        };
        declare(&mut t, &d);
        go_to(&mut t, Step::DeclareAttackers);
        let mut a = attackers(&t);
        a.sort();
        assert_eq!(a, vec![flunkies, bears], "declaration {decl}");
    }
}

#[test]
fn silent_arbiter_lets_only_the_creature_that_must_attack_attack() {
    cr!("508.1c", "508.1d");
    supported("Silent Arbiter");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Silent Arbiter");
    let jugg = t.battlefield(P0, "Juggernaut");
    let bears = t.battlefield(P0, "Grizzly Bears");
    combat_of_p0(&mut t);
    // The Bears alone obey no requirement; both together break the restriction.
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attackers(&t), vec![jugg]);
}

/// P0 attacks with a creature enchanted by Lure; returns (game, attacker).
fn lured_attack(t: &mut TestGame, attacker: &str) -> ObjectId {
    supported("Lure");
    let a = t.battlefield(P0, attacker);
    let lure = t.battlefield(P0, "Lure");
    assert!(t.g.attach(lure, Entity::Object(a)));
    combat_of_p0(t);
    declare(t, &[(a, Entity::Player(P1))]);
    go_to(t, Step::DeclareAttackers);
    a
}

#[test]
fn lure_on_a_creature_with_menace_needs_two_able_blockers() {
    cr!("509.1b", "509.1c", "702.111b");
    supported("Wayward Giant");
    // One creature able to block: it can't block a creature with menace alone, so it
    // doesn't block, and a declared block by it alone is illegal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = lured_attack(&mut t, "Wayward Giant");
    block(&mut t, P1, &[(bears, giant)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blockers_of(&t, giant).is_empty());
    // Two: both must block it (CR 509.1c's example).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let giant = lured_attack(&mut t, "Wayward Giant");
    block(&mut t, P1, &[]);
    go_to(&mut t, Step::DeclareBlockers);
    let mut bs = blockers_of(&t, giant);
    bs.sort();
    assert_eq!(bs, vec![a, b]);
}

#[test]
fn mogg_flunkies_blocks_a_lured_attacker_only_with_another_blocker() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Mogg Flunkies",
        "If an effect says that Mogg Flunkies attacks or blocks if able and you control another creature able to attack or block, you must attack or block with Mogg Flunkies and that creature."
    );
    let mut t = TestGame::new(2);
    let flunkies = t.battlefield(P1, "Mogg Flunkies");
    let bears = lured_attack(&mut t, "Grizzly Bears");
    go_to(&mut t, Step::DeclareBlockers);
    // It can't block alone, so Lure's requirement can't be obeyed.
    assert!(blocking(&t, flunkies).is_empty());
    assert!(!blocked(&t, bears));
    let mut t = TestGame::new(2);
    let flunkies = t.battlefield(P1, "Mogg Flunkies");
    let other = t.battlefield(P1, "Hill Giant");
    let bears = lured_attack(&mut t, "Grizzly Bears");
    go_to(&mut t, Step::DeclareBlockers);
    let mut bs = blockers_of(&t, bears);
    bs.sort();
    assert_eq!(bs, vec![flunkies, other]);
}

#[test]
fn lure_doesnt_force_a_block_that_costs_archangel_of_tithes() {
    cr!("509.1c", "509.1d", "509.1f");
    ruling!(
        "Lure",
        "If there’s a cost associated with having the creature block, no player is forced to pay that cost"
    );
    supported("Archangel of Tithes");
    supported("Giant Spider");
    // Not paying: no block.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P1, "Giant Spider");
    t.lands(P1, "Forest", 1);
    let angel = lured_attack(&mut t, "Archangel of Tithes");
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blocking(&t, spider).is_empty());
    // Paying {1}: the block is legal.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P1, "Giant Spider");
    let forest = t.lands(P1, "Forest", 1)[0];
    let angel2 = lured_attack(&mut t, "Archangel of Tithes");
    block(&mut t, P1, &[(spider, angel2)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(blocking(&t, spider), vec![angel2]);
    assert!(t.obj_now(forest).tapped);
    let _ = angel;
}

#[test]
fn the_total_cost_to_block_is_one_per_blocker_and_must_be_paid_in_full() {
    cr!("509.1d", "509.1f");
    // Two blockers cost {2}; with one Forest, the declaration can't be paid for and is
    // undone: no creature blocks.
    let mut t = TestGame::new(2);
    let s1 = t.battlefield(P1, "Giant Spider");
    let s2 = t.battlefield(P1, "Giant Spider");
    let forest = t.lands(P1, "Forest", 1)[0];
    let angel = t.battlefield(P0, "Archangel of Tithes");
    combat_of_p0(&mut t);
    declare(&mut t, &[(angel, Entity::Player(P1))]);
    block(&mut t, P1, &[(s1, angel), (s2, angel)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blockers_of(&t, angel).is_empty());
    assert!(!t.obj_now(forest).tapped);
}

#[test]
fn norns_annex_costs_are_paid_one_way_each_and_two_life_needs_more_than_one_life() {
    cr!("508.1h", "508.1j", "107.4f");
    ruling!(
        "Norn's Annex",
        "if you attack with two creatures, you may pay {W} for one cost and 2 life for the other"
    );
    ruling!("Norn's Annex", "If you're at 1 life or less, you can't pay 2 life.");
    supported("Norn's Annex");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Norn's Annex");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let plains = t.lands(P0, "Plains", 1)[0];
    combat_of_p0(&mut t);
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attackers(&t).len(), 2);
    assert!(t.obj_now(plains).tapped);
    assert_eq!(t.life(P0), 18);
    // At 1 life with no white mana, the cost can't be paid: the attack is undone.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Norn's Annex");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 1;
    combat_of_p0(&mut t);
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attackers(&t).is_empty());
    assert_eq!(t.life(P0), 1);
}

#[test]
fn trove_of_temptation_requires_no_attack_its_controller_taxes() {
    cr!("508.1d");
    supported("Trove of Temptation");
    // Trove and Ghostly Prison: attacking P1 needs a cost, and P1 has no planeswalker.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trove of Temptation");
    t.battlefield(P1, "Ghostly Prison");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    combat_of_p0(&mut t);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attackers(&t).is_empty());
    // With a planeswalker, one creature must attack it (for free).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trove of Temptation");
    t.battlefield(P1, "Ghostly Prison");
    let jace = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    combat_of_p0(&mut t);
    go_to(&mut t, Step::DeclareAttackers);
    let att = attackers(&t);
    assert_eq!(att.len(), 1);
    assert!(att[0] == a || att[0] == b);
    assert_eq!(attacks(&t, att[0]), Some(Entity::Object(jace)));
}

#[test]
fn trove_of_temptation_in_two_headed_giant_needs_an_attack_on_its_controller_by_each_opponent() {
    cr!("508.1d", "805.10b");
    ruling!(
        "Trove of Temptation",
        "Attacking your teammate or a planeswalker your teammate controls doesn't satisfy Trove of Temptation's requirement."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P2, "Trove of Temptation");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let mates = t.battlefield(P1, "Grizzly Bears");
    combat_of_p0(&mut t);
    // Both attack P3 (Trove's controller's teammate): illegal.
    declare(
        &mut t,
        &[(mine, Entity::Player(P3)), (mates, Entity::Player(P3))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacks(&t, mine), Some(Entity::Player(P2)));
    assert_eq!(attacks(&t, mates), Some(Entity::Player(P2)));
}

#[test]
fn a_creature_goaded_by_each_opponent_attacks_an_opponent_rather_than_a_planeswalker() {
    cr!("508.1d", "701.15b", "701.15c");
    ruling!(
        "Disrupt Decorum",
        "If a creature you control has been goaded by each of your opponents, the creature must attack an opponent (rather than a planeswalker"
    );
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.g.objects[bears.0 as usize].goaded_by = vec![P1, P2];
    combat_of_p0(&mut t);
    declare(&mut t, &[(bears, Entity::Object(jace))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(matches!(attacks(&t, bears), Some(Entity::Player(p)) if p == P1 || p == P2));
}
