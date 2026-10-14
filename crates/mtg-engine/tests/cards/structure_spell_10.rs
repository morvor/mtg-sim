//! Structure-coverage tests (round 10): spell-ability structures shared by several cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks that a
//! real card's spell does what its text says.

use super::structure_activated_2::{next_turn, target_choices};
use super::structure_activated_6::entity_choices;
use super::structure_seed_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::library::top_cards;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `name` in P1's hand and casts it during P1's main phase (P1 gets the mana it
/// needs from `lands` of `land`); returns the spell.
fn p1_casts(t: &mut TestGame, name: &str, land: &str, lands: usize, targets: &[Entity]) -> ObjectId {
    t.lands(P1, land, lands);
    let c = t.hand(P1, name);
    t.cast(P1, c).targets(targets).go()
}

#[test]
fn reprieve_returns_a_spell_to_its_owners_hand_and_draws() {
    cr!("608.2", "115.1", "121.1");
    // "Return target spell to its owner's hand." / "Draw a card."
    supported("Reprieve");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = p1_casts(&mut t, "Grizzly Bears", "Forest", 2, &[]);
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Reprieve");
    t.cast(P0, r).target(bears).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(land)), "a permanent isn't a spell");
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.hand_size(P0), 1, "Reprieve's caster draws");
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_graveyard(P0, "Reprieve"));
}

#[test]
fn thirst_for_identity_discards_a_creature_or_two_cards() {
    cr!("608.2c", "121.1", "701.9a");
    // "Draw three cards. Then discard two cards unless you discard a creature card."
    supported("Thirst for Identity");
    // Discarding a creature card: one card.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Island", 3);
    let s = t.hand(P0, "Thirst for Identity");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 4, "2 + 3 drawn - 1 creature card");
    assert_eq!(
        entity_choices(&t, P0),
        vec![vec![Entity::Object(bears)]],
        "only creature cards can be the alternative"
    );
    // Declining: two cards.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Island", 3);
    let s = t.hand(P0, "Thirst for Identity");
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(bears)]);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3, "2 + 3 drawn - 2");
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    let c = entity_choices(&t, P0).pop().unwrap();
    assert_eq!(c.len(), 5, "any two cards of the five in hand");
    assert_eq!(t.graveyard_size(P0), 3, "two discarded cards and the spell");
    // No creature card in hand: two cards.
    let mut t = TestGame::new(2);
    t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Island", 3);
    let s = t.hand(P0, "Thirst for Identity");
    t.answer_yes(P0, true);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2, "1 + 3 drawn - 2");
}

#[test]
fn taste_of_blood_drains_a_player_or_planeswalker() {
    cr!("608.2", "115.1", "120.3a", "120.3c", "119.3");
    // "Taste of Blood deals 1 damage to target player or planeswalker and you gain 1 life."
    supported("Taste of Blood");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Swamp", 1);
    let s = t.hand(P0, "Taste of Blood");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Object(garruk)));
    assert!(!c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Swamp", 1);
    let s = t.hand(P0, "Taste of Blood");
    t.cast(P0, s).target(garruk).go();
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
    assert_eq!((t.life(P0), t.life(P1)), (21, 20));
}

#[test]
fn traitorous_instinct_steals_untaps_and_pumps_until_end_of_turn() {
    cr!("608.2", "613.1b", "701.26b", "611.2a", "514.2", "702.10b");
    // "Gain control of target creature until end of turn. Untap that creature. Until end
    // of turn, it gets +2/+0 and gains haste."
    supported("Traitorous Instinct");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.g.tap(bears);
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Traitorous Instinct");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    let o = t.obj_now(bears);
    assert_eq!(o.controller, P0);
    assert!(!o.tapped, "untapped");
    assert!(o.has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.obj_now(other).controller, P1);
    assert_eq!(t.pt(other), (3, 3));
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16, "it attacked its owner");
    next_turn(&mut t);
    let o = t.obj_now(bears);
    assert_eq!(o.controller, P1);
    assert!(!o.has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn turn_aside_counters_a_spell_that_targets_your_permanent() {
    cr!("608.2", "115.1", "701.6a");
    // "Counter target spell that targets a permanent you control."
    supported("Turn Aside");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let growth = p1_casts(&mut t, "Giant Growth", "Forest", 1, &[Entity::Object(theirs)]);
    let shock = p1_casts(&mut t, "Shock", "Mountain", 1, &[Entity::Player(P0)]);
    t.lands(P0, "Island", 2);
    let ta = t.hand(P0, "Turn Aside");
    assert!(
        t.cast(P0, ta).try_go().is_err(),
        "neither spell targets a permanent you control"
    );
    t.clear_answers();
    let bolt = p1_casts(&mut t, "Lightning Bolt", "Mountain", 1, &[Entity::Object(mine)]);
    t.cast(P0, ta).target(bolt).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert_eq!(c, vec![Entity::Object(bolt)]);
    assert!(!c.contains(&Entity::Object(growth)) && !c.contains(&Entity::Object(shock)));
    t.resolve_all();
    assert!(t.on_battlefield(mine), "the Bolt was countered");
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 18, "the Shock still resolved");
    assert_eq!(t.pt(theirs), (5, 5));
}

#[test]
fn vanishment_puts_a_nonland_permanent_on_top_of_its_owners_library() {
    cr!("608.2", "115.1");
    // "Put target nonland permanent on top of its owner's library."
    supported("Vanishment");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Island", 5);
    let s = t.hand(P0, "Vanishment");
    t.cast(P0, s).target(bears).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(anthem)));
    assert!(!c.contains(&Entity::Object(land)), "a land");
    t.resolve_all();
    let top = top_cards(&t.g, P1, 1);
    assert_eq!(top, vec![t.g.current(bears)]);
    assert_eq!(t.zone(bears), Zone::Library(P1));
    assert_eq!(t.library_size(P1), 31);
    assert_eq!(t.library_size(P0), 30);
    assert!(t.on_battlefield(anthem));
}

#[test]
fn whisk_away_tucks_an_attacking_or_blocking_creature() {
    cr!("608.2", "115.1");
    // "Put target attacking or blocking creature on top of its owner's library."
    supported("Whisk Away");
    for blocker_target in [false, true] {
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, "Grizzly Bears");
        let idle = t.battlefield(P0, "Hill Giant");
        let blocker = t.battlefield(P1, "Savannah Lions");
        let idle2 = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        declare(&mut t, &[(attacker, Entity::Player(P1))]);
        block(&mut t, P1, &[(blocker, attacker)]);
        go_to(&mut t, Step::DeclareBlockers);
        let target = if blocker_target { blocker } else { attacker };
        t.lands(P0, "Island", 3);
        let s = t.hand(P0, "Whisk Away");
        t.cast(P0, s).target(target).go();
        let c = target_choices(&t, P0).pop().unwrap();
        assert!(c.contains(&Entity::Object(attacker)) && c.contains(&Entity::Object(blocker)));
        assert!(!c.contains(&Entity::Object(idle)) && !c.contains(&Entity::Object(idle2)));
        t.resolve_all();
        let owner = if blocker_target { P1 } else { P0 };
        assert_eq!(top_cards(&t.g, owner, 1), vec![t.g.current(target)]);
        let other = if blocker_target { attacker } else { blocker };
        assert!(t.on_battlefield(other));
    }
}

#[test]
fn reforge_the_soul_wheels_each_player() {
    cr!("608.2", "701.9a", "121.1");
    // "Each player discards their hand, then draws seven cards."
    supported("Reforge the Soul");
    let mut t = TestGame::new(3);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Lightning Bolt");
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Reforge the Soul");
    t.cast(P0, s).go();
    t.resolve_all();
    for p in [P0, P1, PlayerId(2)] {
        assert_eq!(t.hand_size(p), 7, "{p:?}");
        assert_eq!(t.library_size(p), 23);
    }
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.graveyard_size(PlayerId(2)), 0, "an empty hand");
}

#[test]
fn absorb_counters_a_spell_and_gains_three_life() {
    cr!("608.2", "701.6a", "119.3");
    // "Counter target spell. You gain 3 life."
    supported("Absorb");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = p1_casts(&mut t, "Grizzly Bears", "Forest", 2, &[]);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Plains", 1);
    let a = t.hand(P0, "Absorb");
    t.cast(P0, a).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!((t.life(P0), t.life(P1)), (23, 20));
}

#[test]
fn adamant_will_pumps_and_grants_indestructible_until_end_of_turn() {
    cr!("608.2", "611.2a", "514.2", "702.12b");
    // "Target creature gets +2/+2 and gains indestructible until end of turn."
    supported("Adamant Will");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Adamant Will");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.pt(other), (2, 2));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Indestructible));
    t.lands(P1, "Swamp", 3);
    let m = t.hand(P1, "Murder");
    t.cast(P1, m).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears), "indestructible");
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn alms_of_the_vein_drains_target_opponent() {
    cr!("608.2", "115.1", "119.3");
    // "Target opponent loses 3 life and you gain 3 life."
    supported("Alms of the Vein");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Alms of the Vein");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Player(PlayerId(2))));
    assert!(!c.contains(&Entity::Player(P0)), "an opponent");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(PlayerId(2)), 20);
}

#[test]
fn warlords_fury_grants_first_strike_to_your_creatures() {
    cr!("608.2", "611.2a", "611.2c", "514.2", "702.7b");
    // "Creatures you control gain first strike until end of turn." / "Draw a card."
    supported("Warlord's Fury");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Warlord's Fury");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "drew a card");
    for c in [a, b] {
        assert!(t.obj_now(c).has_keyword(KeywordKind::FirstStrike));
    }
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::FirstStrike));
    let later = t.battlefield(P0, "Savannah Lions");
    assert!(
        !t.obj_now(later).has_keyword(KeywordKind::FirstStrike),
        "the affected set is fixed on resolution (CR 611.2c)"
    );
    // A 2/2 first striker kills a blocking 2/2 without taking damage.
    t.attack(&[(a, Entity::Player(P1))], &[(theirs, a)]);
    assert!(t.on_battlefield(a) && t.obj_now(a).damage == 0);
    assert!(!t.on_battlefield(theirs));
    next_turn(&mut t);
    assert!(!t.obj_now(a).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn revitalizing_repast_adds_a_counter_and_indestructible() {
    cr!("608.2", "122.1", "611.2a", "514.2");
    // "Put a +1/+1 counter on target creature. It gains indestructible until end of turn."
    supported("Revitalizing Repast // Old-Growth Grove");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let s = t.hand(P0, "Revitalizing Repast // Old-Growth Grove");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.counters(other, "+1/+1"), 0);
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Indestructible));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.counters(bears, "+1/+1"), 1, "the counter stays");
}

#[test]
fn fend_off_prevents_combat_damage_dealt_by_a_creature() {
    cr!("608.2", "615.1", "510.2");
    // "Prevent all combat damage that would be dealt by target creature this turn."
    supported("Fend Off");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let lions = t.battlefield(P1, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(giant, Entity::Player(P0)), (lions, Entity::Player(P0))],
    );
    block(&mut t, P0, &[(bears, giant)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Fend Off");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.on_battlefield(bears), "the Giant's damage was prevented");
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 2, "damage dealt to it isn't prevented");
    assert_eq!(t.life(P0), 18, "the other attacker's damage isn't prevented");
}

#[test]
fn guardians_pledge_pumps_your_white_creatures() {
    cr!("608.2", "611.2a", "611.2c", "514.2");
    // "White creatures you control get +2/+2 until end of turn."
    supported("Guardians' Pledge");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Savannah Lions");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Guardians' Pledge");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(lions), (4, 3));
    assert_eq!(t.pt(bears), (2, 2), "not white");
    assert_eq!(t.pt(theirs), (2, 1), "not yours");
    let later = t.battlefield(P0, "Savannah Lions");
    assert_eq!(t.pt(later), (2, 1), "the affected set is fixed on resolution");
    next_turn(&mut t);
    assert_eq!(t.pt(lions), (2, 1));
}

#[test]
fn ob_nixiliss_cruelty_shrinks_and_exiles_if_it_would_die() {
    cr!("608.2", "611.2a", "614.1a", "704.5f");
    // "Target creature gets -5/-5 until end of turn. If that creature would die this turn,
    // exile it instead."
    supported("Ob Nixilis's Cruelty");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Ob Nixilis's Cruelty");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile, "exiled instead of dying");
    assert!(t.on_battlefield(bears));
    // A creature that survives is still exiled if it dies later this turn.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Ob Nixilis's Cruelty");
    t.cast(P0, s).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (1, 1));
    t.lands(P0, "Mountain", 2);
    for target in [wurm, bears] {
        let shock = t.hand(P0, "Shock");
        t.cast(P0, shock).target(target).go();
        t.resolve_all();
    }
    assert_eq!(t.zone(wurm), Zone::Exile);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "only that creature");
}

#[test]
fn boiling_seas_destroys_all_islands() {
    cr!("608.2", "701.8a", "205.3i");
    // "Destroy all Islands."
    supported("Boiling Seas");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Island");
    let theirs = t.battlefield(P1, "Island");
    let dual = t.battlefield(P1, "Tropical Island");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Boiling Seas");
    t.cast(P0, s).go();
    t.resolve_all();
    for i in [mine, theirs, dual] {
        assert!(!t.on_battlefield(i));
    }
    assert!(t.on_battlefield(forest));
    assert_eq!(t.named_on_battlefield("Mountain").len(), 4);
}

#[test]
fn bolt_bend_changes_the_target_of_a_spell() {
    cr!("608.2", "115.7", "115.7a");
    // "Change the target of target spell or ability with a single target." (and "This
    // spell costs {3} less to cast if you control a creature with power 4 or greater.")
    supported("Bolt Bend");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let bolt = p1_casts(&mut t, "Lightning Bolt", "Mountain", 1, &[Entity::Object(mine)]);
    let div = p1_casts(&mut t, "Opt", "Island", 1, &[]);
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Bolt Bend");
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.cast(P0, s).go();
    let c = target_choices(&t, P0)[0].clone();
    assert!(c.contains(&Entity::Object(bolt)));
    assert!(!c.contains(&Entity::Object(div)), "no target");
    t.resolve();
    t.resolve_all();
    assert!(t.on_battlefield(mine), "the Bolt was redirected");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(
        t.asked().iter().any(|(p, d)| *p == P0
            && matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.contains(&Entity::Object(theirs)) && !candidates.contains(&Entity::Object(mine)))),
        "the new target must be a different legal target"
    );
}
