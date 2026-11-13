//! Structure-coverage tests (round 2): spell-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{next_turn, target_choices};
use super::structure_seed_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn vines_of_the_recluse_pumps_grants_reach_and_untaps() {
    cr!("608.2", "611.2a", "701.26b");
    // "Target creature gets +1/+2 and gains reach until end of turn. Untap it."
    supported("Vines of the Recluse");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.g.tap(bears);
    t.g.tap(other);
    t.lands(P0, "Forest", 1);
    let s = t.hand(P0, "Vines of the Recluse");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Reach));
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(other).tapped);
    assert_eq!(t.pt(other), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Reach));
}

#[test]
fn savage_surge_pumps_and_untaps_that_creature() {
    cr!("608.2", "611.2a", "701.26b");
    // "Target creature gets +2/+2 until end of turn. Untap that creature."
    supported("Savage Surge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Savage Surge");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!t.obj_now(bears).tapped);
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn demolish_destroys_target_artifact_or_land() {
    cr!("608.2", "701.8a", "115.1");
    // "Destroy target artifact or land."
    supported("Demolish");
    for land in [true, false] {
        let mut t = TestGame::new(2);
        let target = if land {
            t.battlefield(P1, "Island")
        } else {
            t.battlefield(P1, "Marble Chalice")
        };
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Mountain", 4);
        let s = t.hand(P0, "Demolish");
        t.cast(P0, s).target(target).go();
        t.resolve_all();
        assert!(!t.on_battlefield(target));
        assert!(t.on_battlefield(bears));
    }
    // A creature isn't a legal target: with no artifact or land to target, it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::R, 4);
    let s = t.hand(P0, "Demolish");
    assert!(t.cast(P0, s).try_go().is_err());
}

#[test]
fn tempest_of_light_destroys_all_enchantments() {
    cr!("608.2", "701.8a");
    // "Destroy all enchantments."
    supported("Tempest of Light");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Zephyr Charge");
    let theirs = t.battlefield(P1, "Treasure Trove");
    let art = t.battlefield(P1, "Marble Chalice");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Tempest of Light");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(art));
    assert!(t.on_battlefield(bears));
}

#[test]
fn essence_extraction_damages_a_creature_and_gains_life() {
    cr!("608.2", "120.3e", "119.3");
    // "Essence Extraction deals 3 damage to target creature and you gain 3 life."
    supported("Essence Extraction");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Essence Extraction");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn rain_of_embers_damages_each_creature_and_each_player() {
    cr!("608.2", "120.3a", "120.3e");
    // "Rain of Embers deals 1 damage to each creature and each player."
    supported("Rain of Embers");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let chalice = t.battlefield(P1, "Marble Chalice");
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Rain of Embers");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.obj_now(chalice).damage, 0, "not a creature");
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn tremor_damages_each_creature_without_flying() {
    cr!("608.2", "120.3e");
    // "Tremor deals 1 damage to each creature without flying."
    supported("Tremor");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bat = t.battlefield(P1, "Blighted Bat");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Tremor");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    assert_eq!(t.obj_now(mine).damage, 1);
    assert!(
        t.on_battlefield(bat),
        "flying creatures aren't dealt damage"
    );
    assert_eq!(t.obj_now(bat).damage, 0);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn defeat_destroys_only_a_creature_with_power_two_or_less() {
    cr!("608.2", "701.8a", "115.1", "608.2b");
    // "Destroy target creature with power 2 or less."
    supported("Defeat");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Defeat");
    t.cast(P0, s).target(bears).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)));
    assert!(
        !c.contains(&Entity::Object(giant)),
        "Hill Giant (power 3) isn't a legal target"
    );
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    // A target whose power became 3 by resolution is illegal: the spell does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Defeat");
    t.cast(P0, s).target(bears).go();
    let pump = t.battlefield(P0, "Nantuko Disciple");
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 1);
    super::structure_activated_2::act(&mut t, P0, pump, "+2/+2", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Defeat"));
}

#[test]
fn natural_end_destroys_an_artifact_or_enchantment_and_gains_life() {
    cr!("608.2", "701.8a", "119.3");
    // "Destroy target artifact or enchantment. You gain 3 life."
    supported("Natural End");
    for ench in [true, false] {
        let mut t = TestGame::new(2);
        let target = if ench {
            t.battlefield(P1, "Treasure Trove")
        } else {
            t.battlefield(P1, "Marble Chalice")
        };
        t.lands(P0, "Forest", 3);
        let s = t.hand(P0, "Natural End");
        t.cast(P0, s).target(target).go();
        t.resolve_all();
        assert!(!t.on_battlefield(target));
        assert_eq!(t.life(P0), 23);
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn into_the_void_returns_up_to_two_creatures_to_their_owners_hands() {
    cr!("608.2", "115.1a", "601.2c");
    // "Return up to two target creatures to their owners' hands."
    supported("Into the Void");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let stays = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Island", 4);
    let s = t.hand(P0, "Into the Void");
    t.cast(P0, s)
        .targets(&[Entity::Object(mine), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Hill Giant"), "to its owner's hand");
    assert!(t.on_battlefield(stays));
    // Zero targets is allowed ("up to").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let s = t.hand(P0, "Into the Void");
    t.cast(P0, s).targets(&[]).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Into the Void"));
}

#[test]
fn tenderize_has_your_creature_deal_damage_equal_to_its_power() {
    cr!("608.2", "120.3e", "115.1");
    // "Target creature you control deals damage equal to its power to target creature an
    // opponent controls."
    supported("Tenderize");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Tenderize");
    // Two target slots: the targets are answered one per slot.
    t.cast(P0, s).target(giant).target(big).go();
    let c = target_choices(&t, P0);
    let (yours, opps) = (&c[c.len() - 2], &c[c.len() - 1]);
    assert!(yours.contains(&Entity::Object(giant)) && !yours.contains(&Entity::Object(theirs)));
    assert!(opps.contains(&Entity::Object(big)) && !opps.contains(&Entity::Object(giant)));
    t.resolve_all();
    assert_eq!(t.obj_now(big).damage, 3, "Hill Giant's power");
    assert_eq!(t.obj_now(theirs).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 0, "one-sided: not a fight");
}

#[test]
fn soothing_balm_gains_target_player_life() {
    cr!("608.2", "119.3", "115.1");
    // "Target player gains 5 life."
    supported("Soothing Balm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Soothing Balm");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 25);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn hysterical_blindness_shrinks_only_opponents_creatures() {
    cr!("608.2", "611.2c", "613.4c");
    // "Creatures your opponents control get -4/-0 until end of turn."
    supported("Hysterical Blindness");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Island", 3);
    let s = t.hand(P0, "Hysterical Blindness");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(theirs), (2, 4));
    assert_eq!(t.pt(mine), (2, 2));
    // Locked in at resolution (CR 611.2c): a creature that enters later isn't affected.
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (6, 4));
}

#[test]
fn boulderfall_divides_damage_among_any_number_of_targets() {
    cr!("608.2", "601.2d", "115.1");
    // "Boulderfall deals 5 damage divided as you choose among any number of targets."
    supported("Boulderfall");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 8);
    let s = t.hand(P0, "Boulderfall");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 3]));
    t.cast(P0, s)
        .targets(&[Entity::Object(bears), Entity::Player(P1)])
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn unlikely_aid_pumps_power_and_grants_indestructible() {
    cr!("608.2", "611.2a", "702.12b");
    // "Target creature gets +2/+0 and gains indestructible until end of turn."
    supported("Unlikely Aid");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Unlikely Aid");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.on_battlefield(bears));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn shield_wall_raises_toughness_of_creatures_you_control() {
    cr!("608.2", "611.2c");
    // "Creatures you control get +0/+2 until end of turn."
    supported("Shield Wall");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Shield Wall");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(mine), (2, 4));
    assert_eq!(t.pt(theirs), (2, 2));
    // Locked in at resolution (CR 611.2c).
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(mine), (2, 2));
}

#[test]
fn pierce_the_sky_damages_only_a_creature_with_flying() {
    cr!("608.2", "120.3e", "115.1");
    // "Pierce the Sky deals 7 damage to target creature with flying."
    supported("Pierce the Sky");
    let mut t = TestGame::new(2);
    let bat = t.battlefield(P1, "Blighted Bat");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Pierce the Sky");
    t.cast(P0, s).target(bat).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bat)));
    assert!(
        !c.contains(&Entity::Object(bears)),
        "Grizzly Bears has no flying"
    );
    t.resolve_all();
    assert!(!t.on_battlefield(bat));
    assert_eq!(t.obj_now(bears).damage, 0);
    // With no flying creature, it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Pierce the Sky");
    assert!(t.cast(P0, s).try_go().is_err());
}

#[test]
fn turn_the_tide_lasts_until_cleanup() {
    cr!("514.2");
    // Turn the Tide: "Creatures your opponents control get -2/-0 until end of turn."
    supported("Turn the Tide");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Turn the Tide");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(theirs), (1, 3));
    t.advance_to_step(Step::End);
    assert_eq!(t.pt(theirs), (1, 3), "still this turn");
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (3, 3));
}
