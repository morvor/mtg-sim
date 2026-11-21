//! The worked examples of CR 601 (casting spells), played out as tests.

use crate::r600_examples_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn tap_two_target_creatures_needs_two_different_creatures() {
    cr!("601.2c");
    cr_example!("601.2c", "Tap two target creatures");
    let mut t = TestGame::new(2);
    let tap_two = || custom_costed("Tap Two", "Instant", "{0}", "Tap two target creatures.");
    let a = t.battlefield(P1, "Grizzly Bears");
    let c = t.custom(P0, tap_two(), Zone::Hand(P0));
    // With only one creature, the spell's two different legal targets can't be chosen.
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(a)]);
    assert!(t.cast(P0, c).try_go().is_err());
    assert!(t.in_hand(P0, "Tap Two"));
    t.clear_answers();
    // With a second creature, choosing the same one twice still isn't a legal choice.
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(a)]);
    let s = t.cast(P0, c).go();
    let chosen = &chosen_targets(&t, s)[0];
    assert_eq!(chosen.len(), 2);
    assert!(chosen.contains(&Entity::Object(a)) && chosen.contains(&Entity::Object(b)));
    t.resolve();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
}

#[test]
fn an_artifact_land_can_be_both_target_artifact_and_target_land() {
    cr!("601.2c");
    cr_example!("601.2c", "can target the same artifact land twice");
    supported("Seat of the Synod");
    let mut t = TestGame::new(2);
    let seat = t.battlefield(P1, "Seat of the Synod");
    let def = custom_costed(
        "Artifact and Land",
        "Sorcery",
        "{0}",
        "Destroy target artifact and target land.",
    );
    let c = t.custom(P0, def, Zone::Hand(P0));
    let s = t.cast(P0, c).target(seat).target(seat).go();
    assert_eq!(
        chosen_targets(&t, s),
        vec![vec![Entity::Object(seat)], vec![Entity::Object(seat)]]
    );
    t.resolve();
    assert!(t.in_graveyard(P1, "Seat of the Synod"));
}

#[test]
fn altars_reap_sacrificing_thunderscape_familiar_still_costs_only_b() {
    cr!("601.2f", "601.2h");
    cr_example!("601.2h", "You cast Altar’s Reap");
    supported("Altar's Reap");
    supported("Thunderscape Familiar");
    let mut t = TestGame::new(2);
    let familiar = t.battlefield(P0, "Thunderscape Familiar");
    let swamp = t.lands(P0, "Swamp", 1)[0];
    let reap = t.hand(P0, "Altar's Reap");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(familiar)]);
    t.cast(P0, reap).go();
    // The cost was locked in at {B} before the Familiar was sacrificed to pay it.
    assert!(t.in_graveyard(P0, "Thunderscape Familiar"));
    assert!(t.obj(swamp).tapped);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn void_winnower_doesnt_stop_rolling_thunder_from_being_begun() {
    cr!("601.3a", "601.2e", "601.6");
    cr_example!("601.3a", "Void Winnower");
    supported("Void Winnower");
    supported("Rolling Thunder");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Void Winnower");
    let mountains = t.lands(P0, "Mountain", 4);
    let thunder = t.hand(P0, "Rolling Thunder");
    // An opponent may begin to cast it: the chosen X may make its mana value odd.
    assert!(t
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == thunder)));
    // X = 2 makes the mana value 4 (even): the casting is illegal and reversed.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    assert!(t.cast(P0, thunder).x(2).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Rolling Thunder"));
    assert!(mountains.iter().all(|m| !t.obj(*m).tapped));
    // X = 1 makes it 3 (odd): it's cast.
    t.clear_answers();
    t.cast(P0, thunder).x(1).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn an_aura_flash_permission_lets_a_bestow_card_be_cast_bestowed() {
    cr!("601.3b", "702.103b");
    cr_example!("601.3b", "creature card with bestow in your hand");
    supported("Nyxborn Rollicker");
    let bestow = CastMethod::Keyword(KeywordKind::Bestow);
    let mut t = TestGame::new(2);
    // The abstract effect of the example, compiled from its own words.
    let permission = custom_card(
        "Aura Flash",
        "Enchantment",
        None,
        "You may cast Aura spells as though they had flash.",
    );
    t.custom(P0, permission, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let rollicker = t.hand(P0, "Nyxborn Rollicker");
    t.set_step(P1, Step::End);
    // At instant speed it may be begun bestowed (an Aura spell), but not as a creature.
    assert!(can_cast(&mut t, P0, rollicker, &bestow));
    assert!(!can_cast(&mut t, P0, rollicker, &CastMethod::Normal));
    t.cast(P0, rollicker).method(bestow.clone()).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn garruks_horde_allows_casting_a_noncreature_morph_card_face_down_from_the_library() {
    cr!("601.3e", "702.37c");
    cr_example!("601.3e", "noncreature card with morph");
    supported("Garruk's Horde");
    supported("Lumithread Field");
    let face_down = CastMethod::FaceDown(KeywordKind::Morph);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    t.lands(P0, "Plains", 3);
    let field = t.library_top(P0, "Lumithread Field");
    // As the enchantment it is, it can't be cast from the library ...
    assert!(!can_cast(&mut t, P0, field, &CastMethod::Normal));
    // ... but cast face down with morph it's a creature spell, which Garruk's Horde allows.
    assert!(can_cast(&mut t, P0, field, &face_down));
    let s = t.cast(P0, field).method(face_down.clone()).go();
    assert!(t.obj(s).chars.is(CardType::Creature));
    t.resolve();
    let fd = t.g.current(field);
    assert!(t.on_battlefield(fd));
    assert!(t.obj(fd).face_down);
    assert_eq!(t.pt(fd), (2, 2));
}

#[test]
fn melek_allows_chop_down_and_garruks_horde_allows_giant_killer() {
    cr!("601.3e", "715.3a");
    cr_example!("601.3e", "Melek, Izzet Paragon");
    supported("Melek, Izzet Paragon");
    supported("Garruk's Horde");
    supported("Giant Killer");
    let chop_down = CastMethod::Half(1);
    // Melek: Chop Down (an instant) but not Giant Killer (a creature).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melek, Izzet Paragon");
    t.lands(P0, "Plains", 3);
    let big = t.battlefield(P1, "Craw Wurm");
    let killer = t.library_top(P0, "Giant Killer");
    assert!(!can_cast(&mut t, P0, killer, &CastMethod::Normal));
    assert!(can_cast(&mut t, P0, killer, &chop_down));
    t.cast(P0, killer).method(chop_down.clone()).target(big).go();
    t.resolve_all();
    assert!(!t.on_battlefield(big));

    // Garruk's Horde: Giant Killer but not Chop Down.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    t.lands(P0, "Plains", 3);
    t.battlefield(P1, "Craw Wurm");
    let killer = t.library_top(P0, "Giant Killer");
    assert!(!can_cast(&mut t, P0, killer, &chop_down));
    assert!(can_cast(&mut t, P0, killer, &CastMethod::Normal));
    t.cast(P0, killer).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Giant Killer").len(), 1);
}

#[test]
fn inscription_of_abundance_kicked_may_choose_any_number_of_modes() {
    cr!("601.4", "601.2b", "702.33a");
    cr_example!("601.4", "Inscription of Abundance");
    supported("Inscription of Abundance");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, "Inscription of Abundance");
    // The modes are announced (two of them) before the kicker is chosen to be paid.
    let s = t
        .cast(P0, c)
        .modes(&[0, 1])
        .kicked(true)
        .target(bears)
        .target(P0)
        .go();
    assert_eq!(t.obj(s).stack.as_ref().unwrap().chosen.len(), 2);
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    // The greatest power among P0's creatures is now 4.
    assert_eq!(t.life(P0), 24);
}
