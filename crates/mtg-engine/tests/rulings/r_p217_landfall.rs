//! Rulings batch P217 — landfall ("Whenever a land you control enters, ..."; an ability
//! word, CR 207.2c).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::*;

const LANDFALL: &str = "a land you control enters";

#[test]
fn bloodghast_triggers_only_if_already_in_the_graveyard_as_the_land_enters() {
    cr!("603.6a", "603.10", "603.2");
    ruling!(
        "Bloodghast",
        "Bloodghast's landfall ability triggers only if it's already in your graveyard at the time a land enters under your control."
    );
    supported("Bloodghast");
    // On the battlefield as the land enters: it doesn't trigger (its landfall ability
    // works only from the graveyard), even if it dies right after.
    let mut t = TestGame::new(2);
    let ghast = t.battlefield(P0, "Bloodghast");
    enter(&mut t, P0, "Swamp");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 0);
    destroy(&mut t, ghast);
    assert!(t.in_graveyard(P0, "Bloodghast"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bloodghast"));
    // Already in the graveyard: it triggers and may return.
    enter(&mut t, P0, "Swamp");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bloodghast").len(), 1);
    // In hand: no trigger.
    let mut t = TestGame::new(2);
    t.hand(P0, "Bloodghast");
    enter(&mut t, P0, "Swamp");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 0);
}

#[test]
fn adventuring_gear_gone_gives_the_bonus_to_the_creature_it_was_attached_to() {
    cr!("608.2h", "113.7a", "301.5");
    ruling!(
        "Adventuring Gear",
        "If Adventuring Gear leaves the battlefield before its landfall ability resolves, the creature it was attached to at the time it left the battlefield gets +2/+2. If it wasn't attached to a creature at that time, nothing gets the bonus."
    );
    supported("Adventuring Gear");
    // Attached to the Bears as it leaves: the Bears get +2/+2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gear = attach_new(&mut t, P0, "Adventuring Gear", bears);
    enter(&mut t, P0, "Forest");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.g.sacrifice(gear, P0);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Adventuring Gear"));
    assert_eq!(t.pt(bears), (4, 4));
    // Unattached as it leaves (it was moved off the Bears in response): nothing gets it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gear = attach_new(&mut t, P0, "Adventuring Gear", bears);
    enter(&mut t, P0, "Forest");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.g.unattach(gear);
    t.g.recompute();
    t.g.sacrifice(gear, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

/// P0's Turntimber Basilisk's landfall ability ("you may have target creature block this
/// creature this turn if able") triggers once per creature in `targets` (one land
/// entering for each) and resolves, P0 saying yes each time. Then P0 attacks P1 with the
/// Basilisk.
fn basilisk_lures(t: &mut TestGame, basilisk: ObjectId, targets: &[ObjectId]) {
    use crate::r_s01_common::attack_with;
    use crate::r_s20_common::to_beginning_of_combat;
    for target in targets {
        t.answer_targets(P0, &[Entity::Object(*target)]);
        enter(t, P0, "Forest");
        assert_eq!(triggers_on_stack(t, LANDFALL), 1);
        t.answer_yes(P0, true);
        t.resolve_all();
    }
    to_beginning_of_combat(t, P0);
    attack_with(t, &[(basilisk, Entity::Player(P1))]);
}

#[test]
fn turntimber_basilisk_triggering_twice_makes_two_creatures_block_it() {
    use crate::r_s21_common::legal_blocks;
    cr!("509.1c", "603.2");
    ruling!(
        "Turntimber Basilisk",
        "If Turntimber Basilisk's landfall ability triggers multiple times during the same turn, you can have multiple creatures block it that turn if able."
    );
    supported("Turntimber Basilisk");
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Turntimber Basilisk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    basilisk_lures(&mut t, basilisk, &[bears, ogre]);
    // Both targeted creatures must block it: blocking with only one (or none) violates a
    // requirement that could have been obeyed.
    assert!(legal_blocks(&mut t, P1, &[(bears, basilisk), (ogre, basilisk)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, basilisk)]));
    assert!(!legal_blocks(&mut t, P1, &[(ogre, basilisk)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    // With one trigger, only that creature must block.
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Turntimber Basilisk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    basilisk_lures(&mut t, basilisk, &[bears]);
    assert!(legal_blocks(&mut t, P1, &[(bears, basilisk)]));
    assert!(!legal_blocks(&mut t, P1, &[(ogre, basilisk)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
}

#[test]
fn turntimber_basilisk_can_target_creatures_that_cant_block_and_the_requirement_does_nothing() {
    use crate::r_s21_common::legal_blocks;
    cr!("509.1c", "115.1");
    ruling!(
        "Turntimber Basilisk",
        "Such creatures can be targeted by Turntimber Basilisk's landfall ability, but the requirement to block does nothing."
    );
    supported("Turntimber Basilisk");
    // A tapped creature: a legal target; it can't block, so not blocking is legal.
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Turntimber Basilisk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    basilisk_lures(&mut t, basilisk, &[bears]);
    assert!(t.obj(bears).tapped);
    assert!(legal_blocks(&mut t, P1, &[]));
    // A creature that can't block ("This creature can't block."): likewise.
    supported("Hulking Goblin");
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Turntimber Basilisk");
    let goblin = t.battlefield(P1, "Hulking Goblin");
    basilisk_lures(&mut t, basilisk, &[goblin]);
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(goblin, basilisk)]));
    // A creature the defending player doesn't control (P0's own): no requirement on P1.
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Turntimber Basilisk");
    let own = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(own);
    let bears = t.battlefield(P1, "Grizzly Bears");
    basilisk_lures(&mut t, basilisk, &[own]);
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, basilisk)]));
}
