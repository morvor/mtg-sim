//! Rulings batch P035 — "remove all [kind] counters from ..." (Sporogenesis and the other
//! cards that phrase compiles for): every such counter on each object is removed.

use crate::r_p035_common::*;
use crate::r_s01_common::{attack_with, give_mana_for, supported, tokens};
use crate::r_s02_common::destroy;
use crate::r_s06_common::has_kw;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn assert_all_supported(names: &[&str]) {
    for n in names {
        supported(n);
    }
}

#[test]
fn remove_all_counters_cards_compile() {
    assert_all_supported(&[
        "Aether Snap",
        "Ammit Eternal",
        "Anthroplasm",
        "Ashling the Pilgrim",
        "Aurification",
        "Blitz Leech",
        "Blood Hound",
        "Enchanted River's Grasp",
        "Fraying Line",
        "Hapatra's Mark",
        "Mine Layer",
        "Norn's Dominion",
        "Oblivion Stone",
        "Perfect Intimidation",
        "Purging Stormbrood // Absorb Essence",
        "Sporogenesis",
        "Vampire Hexmage",
        "Witherscale Wurm",
    ]);
}

// ---------------------------------------------------------------------------------------
// Sporogenesis
// ---------------------------------------------------------------------------------------

#[test]
fn sporogenesis_fungus_counters_and_saprolings() {
    cr!("603.10a", "122.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sporogenesis");
    let giant = t.battlefield(P1, "Hill Giant");
    // At P0's upkeep: a fungus counter on target nontoken creature.
    t.advance_to(P1, Step::End);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(giant, "fungus"), 1);
    put(&mut t, giant, "fungus", 1);
    // It dies: a Saproling for each fungus counter, for Sporogenesis's controller.
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn sporogenesis_several_copies_each_make_saprolings() {
    cr!("603.2", "603.10a");
    ruling!(
        "Sporogenesis",
        "If more than one Sporogenesis is on the battlefield and a creature with fungus counters on it leaves the battlefield, each Sporogenesis will put a Saproling onto the battlefield for each counter."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sporogenesis");
    t.battlefield(P0, "Sporogenesis");
    t.battlefield(P0, "Sporogenesis");
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, "fungus", 2);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 6, "3 x 2 Saprolings");
}

#[test]
fn sporogenesis_leaving_removes_all_fungus_counters() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Sporogenesis",
        "If one Sporogenesis leaves the battlefield, all fungus counters are removed even if other Sporogenesis cards are on the battlefield."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Sporogenesis");
    t.battlefield(P0, "Sporogenesis");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, giant, "fungus", 2);
    put(&mut t, bears, "fungus", 1);
    put(&mut t, bears, counters::PLUS1, 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(giant, "fungus"), 0);
    assert_eq!(t.counters(bears, "fungus"), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 1, "other kinds stay");
    // The other Sporogenesis no longer has anything to count.
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Other cards whose "remove all counters" compiles
// ---------------------------------------------------------------------------------------

#[test]
fn aether_snap_removes_all_counters_and_exiles_tokens() {
    cr!("122.1");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, counters::PLUS1, 2);
    put(&mut t, giant, "charge", 1);
    let gid = t.g.current(giant);
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    give_mana_for(&mut t, P0, "Aether Snap");
    let snap = t.hand(P0, "Aether Snap");
    t.cast(P0, snap).go();
    t.resolve_all();
    assert_eq!(total_counters(&t, gid), 0);
    assert!(!t.g.is_live(tok));
}

#[test]
fn ammit_eternal_counters_and_removal() {
    cr!("702.130a", "510.3a");
    let mut t = TestGame::new(2);
    let ammit = t.battlefield(P0, "Ammit Eternal");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(ammit, counters::MINUS1), 1);
    assert_eq!(t.pt(ammit), (4, 4));
    attack_with(&mut t, &[(ammit, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 4);
    assert_eq!(t.counters(ammit, counters::MINUS1), 0);
    assert_eq!(t.pt(ammit), (5, 5));
}

#[test]
fn anthroplasm_replaces_its_counters() {
    cr!("107.3a", "614.1c");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Anthroplasm");
    t.settle();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    let cur = t.g.current(a);
    t.g.objects[cur.0 as usize].summoning_sick = false;
    t.lands(P0, "Wastes", 5);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let a_now = t.g.current(a);
    t.activate(P0, a_now, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 5);
}

#[test]
fn ashling_third_resolution_deals_damage_equal_to_counters_removed() {
    cr!("608.2c", "120.3");
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling the Pilgrim");
    let wurm = t.battlefield(P1, "Craw Wurm");
    put(&mut t, ashling, counters::PLUS1, 1);
    t.lands(P0, "Mountain", 6);
    for _ in 0..2 {
        t.activate(P0, ashling, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.counters(ashling, counters::PLUS1), 3);
    assert_eq!(t.life(P1), 20);
    // Third: four counters are removed, and it deals 4 damage to each creature and player.
    t.activate(P0, ashling, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Craw Wurm"), "4 damage to a 6/4");
    let _ = wurm;
    assert!(t.in_graveyard(P0, "Ashling the Pilgrim"));
}

#[test]
fn aurification_gold_counters_walls_and_cleanup() {
    cr!("702.3b", "603.6c");
    let mut t = TestGame::new(2);
    let aur = t.battlefield(P0, "Aurification");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(bears, "gold"), 1);
    assert!(has_kw(&t, bears, KeywordKind::Defender));
    assert!(t
        .obj_now(bears)
        .chars
        .subtypes
        .iter()
        .any(|s| s.as_str() == "Wall"));
    destroy(&mut t, aur);
    t.resolve_all();
    assert_eq!(t.counters(bears, "gold"), 0);
    assert!(!has_kw(&t, bears, KeywordKind::Defender));
}

#[test]
fn blitz_leech_shrinks_and_removes_all_counters() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, counters::PLUS1, 2);
    put(&mut t, giant, "charge", 1);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Blitz Leech");
    t.resolve_all();
    assert_eq!(total_counters(&t, giant), 0);
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn blood_hound_counters_for_damage_removed_at_end_step() {
    cr!("120.3", "122.6");
    let mut t = TestGame::new(2);
    let hound = t.battlefield(P0, "Blood Hound");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(P0).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(hound, counters::PLUS1), 3);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(hound, counters::PLUS1), 0);
}

#[test]
fn enchanted_rivers_grasp_taps_and_removes_counters() {
    cr!("502.3", "613.1f");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, counters::PLUS1, 2);
    give_mana_for(&mut t, P0, "Enchanted River's Grasp");
    let aura = t.hand(P0, "Enchanted River's Grasp");
    t.cast(P0, aura).target(giant).go();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(total_counters(&t, giant), 0);
    assert!(t.obj_now(giant).chars.abilities.is_empty());
    t.advance_to(P1, Step::Draw);
    assert!(t.obj_now(giant).tapped, "doesn't untap");
}

#[test]
fn fraying_line_pay_or_exile() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(mine)]);
    let line = t.enter(P0, "Fraying Line");
    t.resolve_all();
    assert_eq!(t.counters(mine, "rope"), 1);
    // P1 pays {2} and puts a rope counter on the Wurm.
    t.lands(P1, "Wastes", 2);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(theirs)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(theirs, "rope"), 1);
    // P0 doesn't pay: Fraying Line and each creature without a rope counter are exiled,
    // then all rope counters are removed.
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(line), Zone::Exile);
    assert_eq!(t.zone(other), Zone::Exile);
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    assert_eq!(t.counters(mine, "rope"), 0);
    assert_eq!(t.counters(theirs, "rope"), 0);
}

#[test]
fn hapatras_mark_hexproof_and_removes_minus_counters() {
    cr!("702.11b");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    put(&mut t, giant, counters::MINUS1, 2);
    put(&mut t, giant, "charge", 1);
    give_mana_for(&mut t, P0, "Hapatra's Mark");
    let mark = t.hand(P0, "Hapatra's Mark");
    t.cast(P0, mark).target(giant).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 0);
    assert_eq!(t.counters(giant, "charge"), 1, "only -1/-1 counters");
    assert!(has_kw(&t, giant, KeywordKind::Hexproof));
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn mine_layer_mines_and_cleanup() {
    cr!("603.6c", "701.8a");
    let mut t = TestGame::new(2);
    let layer = t.battlefield(P0, "Mine Layer");
    let a = t.battlefield(P1, "Forest");
    let b = t.battlefield(P1, "Island");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, layer, 0, &[Entity::Object(a)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(a, "mine"), 1);
    // Tapping the mined land destroys it.
    let a_now = t.g.current(a);
    t.g.tap(a_now);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"));
    // A second mine; Mine Layer leaves: the counter is removed.
    let cur = t.g.current(layer);
    t.g.objects[cur.0 as usize].tapped = false;
    t.activate(P0, layer, 0, &[Entity::Object(b)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(b, "mine"), 1);
    destroy(&mut t, layer);
    t.resolve_all();
    assert_eq!(t.counters(b, "mine"), 0);
    let b_now = t.g.current(b);
    t.g.tap(b_now);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(b));
}

#[test]
fn norns_dominion_fate_counters_protect_then_are_removed() {
    cr!("901.7", "311.7");
    let mut t = crate::r_s19_common::planechase_game(2);
    crate::r_s19_common::start_planar_deck(&mut t, P0, &["Norn's Dominion", "Tazeem"]);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.answer_yes(P0, true);
    crate::r_s19_common::chaos(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(mine, "fate"), 1);
    mtg_engine::planechase::planeswalk(&mut t.g, P0);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(mine));
    assert_eq!(t.counters(mine, "fate"), 0);
    let _ = theirs;
}

#[test]
fn oblivion_stone_fate_counters_protect_then_are_removed() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Oblivion Stone");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Wastes", 9);
    t.activate(P0, stone, 0, &[Entity::Object(mine)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(mine, "fate"), 1);
    let cur = t.g.current(stone);
    t.g.objects[cur.0 as usize].tapped = false;
    t.activate(P0, stone, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Oblivion Stone"));
    assert!(t.on_battlefield(mine) && t.on_battlefield(land));
    assert_eq!(t.counters(mine, "fate"), 0);
}

#[test]
fn perfect_intimidation_both_modes() {
    cr!("700.2d");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, counters::PLUS1, 2);
    t.hand(P1, "Island");
    t.hand(P1, "Forest");
    t.hand(P1, "Swamp");
    give_mana_for(&mut t, P0, "Perfect Intimidation");
    let pi = t.hand(P0, "Perfect Intimidation");
    t.cast(P0, pi).modes(&[0, 1]).target(P1).target(giant).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1, "two cards exiled");
    assert_eq!(total_counters(&t, giant), 0);
}

#[test]
fn purging_stormbrood_removes_counters_and_absorb_essence() {
    cr!("720.3");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, counters::PLUS1, 2);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Purging Stormbrood // Absorb Essence");
    t.resolve_all();
    assert_eq!(total_counters(&t, giant), 0);
    // Absorb Essence (the Omen half).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Purging Stormbrood // Absorb Essence");
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::Lifelink));
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
    assert_eq!(t.zone(card), Zone::Library(P0), "shuffled into the library");
}

#[test]
fn vampire_hexmage_removes_all_counters() {
    cr!("306.5c", "704.5i");
    let mut t = TestGame::new(2);
    let hexmage = t.battlefield(P0, "Vampire Hexmage");
    let pw = t.battlefield(P1, "Jace Beleren");
    put(&mut t, pw, counters::LOYALTY, 3);
    t.activate(P0, hexmage, 0, &[Entity::Object(pw)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Vampire Hexmage"));
    assert!(t.in_graveyard(P1, "Jace Beleren"), "0 loyalty");
}

#[test]
fn witherscale_wurm_grants_wither_and_sheds_minus_counters() {
    cr!("702.80a", "509.1h");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Witherscale Wurm");
    let blocker = t.battlefield(P1, "Craw Wurm");
    put(&mut t, wurm, counters::MINUS1, 2);
    crate::r_s03_common::to_blockers(&mut t, &[(wurm, Entity::Player(P1))], &[(blocker, wurm)]);
    t.resolve_all();
    assert!(has_kw(&t, blocker, KeywordKind::Wither));
    t.advance_to(P0, Step::EndOfCombat);
    // The blocker dealt its damage as -1/-1 counters.
    assert_eq!(t.counters(wurm, counters::MINUS1), 2 + 6);
    // Combat damage to an opponent removes all -1/-1 counters.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Witherscale Wurm");
    put(&mut t, wurm, counters::MINUS1, 2);
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.counters(wurm, counters::MINUS1), 0);
    assert_eq!(t.pt(wurm), (9, 9));
}
