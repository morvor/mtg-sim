//! Targets of removal and damage effects: alternatives with different nouns, several
//! instances of "target", "controlled by different players", and damage in parts.

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn decimate_destroys_one_of_each_target() {
    cr!("115.1", "601.2c");
    assert_supported("Decimate");
    let mut t = TestGame::new(2);
    let art = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ench = t.battlefield(P1, "Glorious Anthem");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Decimate");
    t.cast(P0, spell)
        .targets(&[Entity::Object(art)])
        .targets(&[Entity::Object(bears)])
        .targets(&[Entity::Object(ench)])
        .targets(&[Entity::Object(land)])
        .go();
    t.resolve();
    for o in [art, bears, ench, land] {
        assert!(!t.on_battlefield(o));
    }
}

#[test]
fn decimate_cant_be_cast_without_every_target() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Forest");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Decimate");
    // No enchantment to target.
    assert!(t.cast(P0, spell).try_go().is_err());
}

#[test]
fn boom_box_destroys_up_to_one_of_each() {
    cr!("115.1");
    assert_supported("Boom Box");
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Boom Box");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 6);
    // No artifact chosen; a creature and a land.
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.activate(P0, bb, 0, &[]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(land));
}

#[test]
fn nahiri_exiles_a_tapped_creature_but_cant_target_an_untapped_one() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    let nahiri = t.battlefield(P0, "Nahiri, the Harbinger");
    let untapped = t.battlefield(P1, "Grizzly Bears");
    let tapped = t.battlefield(P1, "Hill Giant");
    t.g.tap(tapped);
    t.activate(P0, nahiri, 1, &[Entity::Object(tapped)]).unwrap();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&Entity::Object(tapped)));
    assert!(!cands.contains(&Entity::Object(untapped)));
    t.resolve();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn thraben_exorcism_targets_spirits_creatures_with_disturb_or_enchantments() {
    cr!("115.1");
    assert_supported("Thraben Exorcism");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ench = t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Thraben Exorcism");
    t.cast(P0, spell).target(ench).go();
    let cands = last_target_candidates(&t, P0);
    assert!(!cands.contains(&Entity::Object(bears)));
    t.resolve();
    assert!(t.in_exile("Glorious Anthem"));
}

#[test]
fn protector_of_the_wastes_targets_are_controlled_by_different_players() {
    cr!("115.1");
    assert_supported("Protector of the Wastes");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Ornithopter");
    let c = t.battlefield(P2, "Glorious Anthem");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(c)]);
    t.enter(P0, "Protector of the Wastes");
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(c));

    // Two artifacts of the same player can't both be targets.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Protector of the Wastes");
    t.resolve_all();
    assert_eq!([a, b].iter().filter(|o| t.on_battlefield(**o)).count(), 1);
}

#[test]
fn cone_of_flame_deals_damage_to_three_different_targets() {
    cr!("115.4", "120.2");
    assert_supported("Cone of Flame");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Cone of Flame");
    t.cast(P0, spell)
        .target(a)
        .target(P1)
        .target(b)
        .go();
    t.resolve();
    // 1 to the Bears, 2 to P1, 3 to the Giant.
    assert!(t.on_battlefield(a));
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(b));
}

#[test]
fn fire_and_brimstone_targets_a_player_who_attacked() {
    cr!("120.2");
    assert_supported("Fire and Brimstone");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.lands(P0, "Plains", 5);
    let spell = t.hand(P0, "Fire and Brimstone");
    t.cast(P0, spell).target(P1).go();
    let cands = last_target_candidates(&t, P0);
    assert_eq!(cands, vec![Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn volatile_arsonist_damages_each_chosen_target() {
    cr!("115.1");
    assert_supported("Volatile Arsonist // Dire-Strain Anarchist");
    let mut t = TestGame::new(2);
    let va = t.battlefield(P0, "Volatile Arsonist // Dire-Strain Anarchist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[]);
    t.attack(&[(va, Entity::Player(P1))], &[]);
    assert_eq!(t.obj_now(bears).damage, 1);
    // 1 from the trigger, the rest from combat.
    assert_eq!(t.life(P1), 20 - 1 - t.pt(va).0);
}

#[test]
fn yosei_taps_up_to_five_permanents_that_player_controls() {
    cr!("115.1");
    assert_supported("Yosei, the Morning Star");
    let mut t = TestGame::new(2);
    let yosei = t.battlefield(P0, "Yosei, the Morning Star");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.g.destroy(yosei, None);
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    let cands = last_target_candidates(&t, P0);
    assert!(!cands.contains(&Entity::Object(mine)));
}

#[test]
fn hidden_strings_taps_or_untaps_two_different_permanents() {
    cr!("115.1");
    assert_supported("Hidden Strings");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let lands = t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Hidden Strings");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(0));
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(1));
    t.cast(P0, spell)
        .target(a)
        .target(lands[0])
        .go();
    t.resolve();
    assert!(t.obj_now(a).tapped);
    assert!(!t.obj_now(lands[0]).tapped);
}

#[test]
fn onakke_javelineer_targets_a_player_or_battle() {
    cr!("115.1");
    assert_supported("Onakke Javelineer");
    let mut t = TestGame::new(2);
    let oj = t.battlefield(P0, "Onakke Javelineer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, oj, 0, &[Entity::Player(P1)]).unwrap();
    assert!(!last_target_candidates(&t, P0).contains(&Entity::Object(bears)));
    t.resolve();
    assert_eq!(t.life(P1), 18);
}
