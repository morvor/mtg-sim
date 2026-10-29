//! Paper-scope combat grammar (cards4-paper-combat-statics): combat damage choices (Cloak
//! of Confusion, Farrel's Mantle), blocks and attack/block restrictions (Ydwen Efreet,
//! Blaze of Glory, Joven's Ferrets, Orcish Conscripts, Akron Legionnaire) and untap
//! restrictions (Spore Cloud, Tangle, Telekinesis, Reveka, Wizard Savant).

use crate::combat2_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p`'s creature attacks P1 unblocked; the game stops in the declare blockers step.
fn attack_unblocked(t: &mut TestGame, p: PlayerId, attacker: ObjectId, defender: PlayerId) {
    declare(t, p, &[(attacker, Entity::Player(defender))]);
    declare_blocks(t, defender, &[]);
}

/// Advances to the next untap step of `p`'s turn and returns whether `id` is tapped
/// once it has passed (in that turn's upkeep).
fn tapped_after_untap_step(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    next_upkeep(t, p);
    t.obj_now(id).tapped
}

// ---------------------------------------------------------------------------------------
// Combat damage choices
// ---------------------------------------------------------------------------------------

#[test]
fn cloak_of_confusion_may_trade_combat_damage_for_a_random_discard() {
    cr!("510.1a");
    ruling!(
        "Cloak of Confusion",
        "triggers in the declare blockers step after blockers are declared"
    );
    supported("Cloak of Confusion");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let cloak = t.battlefield(P0, "Cloak of Confusion");
        t.g.attach(cloak, Entity::Object(bears));
        t.hand(P1, "Shock");
        t.hand(P1, "Shock");
        attack_unblocked(&mut t, P0, bears, P1);
        t.answer_yes(P0, yes);
        t.resolve_all();
        to_end_of_combat(&mut t);
        if yes {
            assert_eq!(t.life(P1), 20, "the creature assigns no combat damage");
            assert_eq!(t.hand_size(P1), 1, "the defending player discards a card");
        } else {
            assert_eq!(t.life(P1), 18);
            assert_eq!(t.hand_size(P1), 2);
        }
    }
}

#[test]
fn farrels_mantle_lets_the_creatures_controller_trade_its_combat_damage() {
    cr!("510.1a", "115.1");
    ruling!(
        "Farrel's Mantle",
        "they decide whether or not to make their creature deal no damage to you"
    );
    ruling!(
        "Farrel's Mantle",
        "cannot choose to have the enchanted creature deal damage to itself"
    );
    supported("Farrel's Mantle");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        // The opponent's Aura on P0's creature: P0 decides.
        let mantle = t.battlefield(P1, "Farrel's Mantle");
        t.g.attach(mantle, Entity::Object(bears));
        let giant = t.battlefield(P1, "Hill Giant");
        t.battlefield(P0, "Llanowar Elves");
        t.answer_targets(P1, &[Entity::Object(giant)]);
        attack_unblocked(&mut t, P0, bears, P1);
        t.answer_yes(P0, yes);
        t.resolve_all();
        let offered: Vec<Entity> = t
            .asked()
            .into_iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseTargets { candidates, .. } if p == P1 => Some(candidates),
                _ => None,
            })
            .expect("the Aura's controller chose the target");
        assert!(!offered.contains(&Entity::Object(bears)));
        assert!(offered.contains(&Entity::Object(giant)));
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        to_end_of_combat(&mut t);
        if yes {
            assert!(!t.on_battlefield(giant), "2 power plus 2 damage");
            assert_eq!(t.life(P1), 20);
        } else {
            assert!(t.on_battlefield(giant));
            assert_eq!(t.life(P1), 18);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Blocks and attack/block restrictions
// ---------------------------------------------------------------------------------------

#[test]
fn ydwen_efreet_unblocks_its_attacker_only_when_it_loses_the_flip() {
    cr!("509.1h", "506.4", "705.2");
    ruling!("Ydwen Efreet", "The ability triggers any time Ydwen Efreet blocks.");
    supported("Ydwen Efreet");
    // Seeds whose first flip each way, as the harness's RNG flips them.
    let mut seen = [false, false];
    for seed in 0..40u64 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        let giant = t.battlefield(P0, "Hill Giant");
        let efreet = t.battlefield(P1, "Ydwen Efreet");
        declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
        declare_blocks(&mut t, P1, &[(efreet, giant)]);
        t.resolve_all();
        let blocking = t
            .g
            .combat
            .as_ref()
            .is_some_and(|c| c.blockers_of(giant).contains(&efreet));
        to_end_of_combat(&mut t);
        if blocking {
            seen[0] = true;
            assert_eq!(t.life(P1), 20, "it's still blocked");
        } else {
            seen[1] = true;
            assert_eq!(t.life(P1), 17, "it became unblocked");
        }
        if seen[0] && seen[1] {
            return;
        }
    }
    panic!("both outcomes of the flip weren't seen: {seen:?}");
}

#[test]
fn blaze_of_glory_makes_the_creature_block_each_attacker_it_can() {
    cr!("509.1c");
    ruling!(
        "Blaze of Glory",
        "Does not allow a tapped creature to block"
    );
    supported("Blaze of Glory");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let wall = t.battlefield(P1, "Wall of Wood");
    let other = t.battlefield(P1, "Grizzly Bears");
    let blaze = t.hand(P1, "Blaze of Glory");
    t.lands(P1, "Plains", 1);
    declare(
        &mut t,
        P0,
        &[
            (bears, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    t.cast(P1, blaze).target(wall).go();
    t.resolve();
    // The wall must block all three; the other creature needn't block.
    assert!(!legal_blocks(&mut t, &[]));
    assert!(!legal_blocks(&mut t, &[(wall, bears), (wall, giant)]));
    assert!(legal_blocks(
        &mut t,
        &[(wall, bears), (wall, giant), (wall, elves)]
    ));
    assert!(legal_blocks(
        &mut t,
        &[(wall, bears), (wall, giant), (wall, elves), (other, giant)]
    ));
}

#[test]
fn blaze_of_glory_doesnt_make_a_tapped_creature_block() {
    cr!("509.1c", "509.1a");
    ruling!(
        "Blaze of Glory",
        "Does not allow a tapped creature to block"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let blaze = t.hand(P1, "Blaze of Glory");
    t.lands(P1, "Plains", 1);
    let icy = t.battlefield(P0, "Icy Manipulator");
    t.lands(P0, "Mountain", 1);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.cast(P1, blaze).target(wall).go();
    t.resolve();
    t.activate(P0, icy, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve();
    assert!(t.obj_now(wall).tapped);
    assert!(legal_blocks(&mut t, &[]));
}

#[test]
fn joven_s_ferrets_taps_the_creatures_that_blocked_it_and_keeps_them_tapped() {
    cr!("502.3");
    supported("Joven's Ferrets");
    let mut t = TestGame::new(2);
    let ferrets = t.battlefield(P0, "Joven's Ferrets");
    let wall = t.battlefield(P1, "Wall of Wood");
    let bystander = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, P0, &[(ferrets, Entity::Player(P1))]);
    t.resolve_all();
    declare_blocks(&mut t, P1, &[(wall, ferrets)]);
    to_end_of_combat(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(ferrets));
    assert!(t.obj_now(wall).tapped);
    assert!(!t.obj_now(bystander).tapped);
    // It doesn't untap during its controller's next untap step, then does.
    assert!(tapped_after_untap_step(&mut t, P1, wall));
    assert!(!tapped_after_untap_step(&mut t, P0, ferrets));
    assert!(!tapped_after_untap_step(&mut t, P1, wall));
}

#[test]
fn orcish_conscripts_attacks_and_blocks_only_with_two_others() {
    cr!("508.1c", "509.1b");
    supported("Orcish Conscripts");
    let mut t = TestGame::new(2);
    let orc = t.battlefield(P0, "Orcish Conscripts");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    let p1 = Entity::Player(P1);
    assert!(!legal_attack(&mut t, &[(orc, p1)]));
    assert!(!legal_attack(&mut t, &[(orc, p1), (a, p1)]));
    assert!(legal_attack(&mut t, &[(orc, p1), (a, p1), (b, p1)]));
    assert!(legal_attack(&mut t, &[(a, p1)]));

    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, "Grizzly Bears");
    let orc = t.battlefield(P1, "Orcish Conscripts");
    let c = t.battlefield(P1, "Grizzly Bears");
    let d = t.battlefield(P1, "Llanowar Elves");
    declare(&mut t, P0, &[(x, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, &[(orc, x)]));
    assert!(!legal_blocks(&mut t, &[(orc, x), (c, x)]));
    assert!(legal_blocks(&mut t, &[(orc, x), (c, x), (d, x)]));
    assert!(legal_blocks(&mut t, &[(c, x)]));
}

#[test]
fn akron_legionnaire_lets_only_legionnaires_and_artifact_creatures_attack() {
    cr!("508.1c");
    ruling!(
        "Akron Legionnaire",
        "If you have two or more Legionnaires, they can all attack."
    );
    supported("Akron Legionnaire");
    let mut t = TestGame::new(2);
    let l1 = t.battlefield(P0, "Akron Legionnaire");
    let l2 = t.battlefield(P0, "Akron Legionnaire");
    let golem = t.battlefield(P0, "Bronze Sable");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    let p1 = Entity::Player(P1);
    assert!(legal_attack(&mut t, &[(l1, p1), (l2, p1), (golem, p1)]));
    assert!(!legal_attack(&mut t, &[(bears, p1)]));
    // The opponent's creatures aren't affected.
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[(theirs, Entity::Player(P0))]));
}

// ---------------------------------------------------------------------------------------
// Untap restrictions
// ---------------------------------------------------------------------------------------

#[test]
fn spore_cloud_keeps_the_declared_attackers_and_blockers_tapped() {
    cr!("502.3", "611.2c");
    ruling!(
        "Spore Cloud",
        "Only makes attackers or blockers that have been declared before the spell is cast"
    );
    supported("Spore Cloud");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let cloud = t.hand(P1, "Spore Cloud");
    t.lands(P1, "Forest", 3);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, giant)]);
    t.cast(P1, cloud).go();
    t.resolve();
    assert!(t.obj_now(wall).tapped);
    to_end_of_combat(&mut t);
    assert!(t.on_battlefield(wall), "combat damage was prevented");
    assert!(tapped_after_untap_step(&mut t, P1, wall));
    assert!(tapped_after_untap_step(&mut t, P0, giant));
    assert!(!tapped_after_untap_step(&mut t, P1, wall));
    assert!(!tapped_after_untap_step(&mut t, P0, giant));
}

#[test]
fn tangle_fixes_the_attackers_as_it_resolves() {
    cr!("502.3", "611.2c");
    ruling!(
        "Tangle",
        "The creatures that are not going to untap are determined at the time this spell resolves."
    );
    supported("Tangle");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let tangle = t.hand(P1, "Tangle");
    t.lands(P1, "Forest", 2);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    t.cast(P1, tangle).go();
    t.resolve();
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert!(tapped_after_untap_step(&mut t, P0, giant));
    assert!(!tapped_after_untap_step(&mut t, P0, giant));
}

#[test]
fn telekinesis_keeps_the_creature_tapped_through_two_of_its_controllers_untap_steps() {
    cr!("502.3");
    ruling!(
        "Telekinesis",
        "it won’t untap during its controller’s untap step until it has not done so twice"
    );
    supported("Telekinesis");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let tk = t.hand(P0, "Telekinesis");
    t.lands(P0, "Island", 2);
    t.cast(P0, tk).target(giant).go();
    t.resolve();
    assert!(t.obj_now(giant).tapped);
    assert!(tapped_after_untap_step(&mut t, P1, giant));
    assert!(tapped_after_untap_step(&mut t, P1, giant));
    assert!(!tapped_after_untap_step(&mut t, P1, giant));
}

#[test]
fn telekinesis_counts_untap_steps_of_whoever_controls_the_creature() {
    cr!("502.3");
    ruling!(
        "Telekinesis",
        "regardless of who its controller was during any of those untap steps"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let tk = t.hand(P0, "Telekinesis");
    let magic = t.hand(P0, "Control Magic");
    t.lands(P0, "Island", 4);
    t.cast(P0, tk).target(giant).go();
    t.resolve();
    // P1's untap step: the first of two.
    assert!(tapped_after_untap_step(&mut t, P1, giant));
    // P0 gains control of it: P0's next untap step is the second.
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, magic).target(giant).go();
    t.resolve();
    assert_eq!(controller(&t, giant), P0);
    assert!(tapped_after_untap_step(&mut t, P1, giant));
    assert!(tapped_after_untap_step(&mut t, P0, giant));
    assert!(!tapped_after_untap_step(&mut t, P0, giant));
}

#[test]
fn reveka_deals_damage_and_skips_your_next_untap_step() {
    cr!("502.3");
    ruling!(
        "Reveka, Wizard Savant",
        "If another player gains control of it, it will untap during that player’s next untap step."
    );
    supported("Reveka, Wizard Savant");
    let mut t = TestGame::new(2);
    let reveka = t.battlefield(P0, "Reveka, Wizard Savant");
    t.activate(P0, reveka, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(t.obj_now(reveka).tapped);
    assert!(tapped_after_untap_step(&mut t, P0, reveka));
    assert!(!tapped_after_untap_step(&mut t, P0, reveka));

    // Under another player's control it untaps during that player's untap step.
    let mut t = TestGame::new(2);
    let reveka = t.battlefield(P0, "Reveka, Wizard Savant");
    let magic = t.hand(P1, "Control Magic");
    t.lands(P1, "Island", 4);
    t.activate(P0, reveka, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.obj_now(reveka).tapped);
    t.cast(P1, magic).target(reveka).go();
    t.resolve();
    assert_eq!(controller(&t, reveka), P1);
    assert!(!tapped_after_untap_step(&mut t, P1, reveka));
}
