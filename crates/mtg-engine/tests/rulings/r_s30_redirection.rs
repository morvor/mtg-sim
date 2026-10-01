//! Rulings batch S30 — redirecting damage (CR 614.9): the en-Kor ("{0}: The next 1 damage
//! that would be dealt to this creature this turn is dealt to target creature you control
//! instead."), a redirection shield that is used up, and redirection to a creature that's
//! gone.

use crate::r_s01_common::supported;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::ability_targets;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::damage_marked;
use crate::r_s30_common::damage_events;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` activates the en-Kor ability of `kor` targeting `to`, and it resolves.
fn redirect_one(t: &mut TestGame, p: PlayerId, kor: ObjectId, to: ObjectId) {
    t.activate(p, kor, 0, &[Entity::Object(to)]).unwrap();
    t.resolve();
}

#[test]
fn the_en_kor_are_supported() {
    cr!("614.9");
    for name in [
        "Nomads en-Kor",
        "Shaman en-Kor",
        "Warrior en-Kor",
        "Spirit en-Kor",
        "Lancers en-Kor",
    ] {
        supported(name);
    }
}

#[test]
fn an_en_kor_can_redirect_damage_to_itself() {
    cr!("614.9", "115.1");
    ruling!("Warrior en-Kor", "It can redirect damage to itself.");
    supported("Warrior en-Kor");
    let mut t = TestGame::new(2);
    let warrior = t.battlefield(P1, "Warrior en-Kor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // It's a legal target of its own ability.
    let candidates = ability_targets(&mut t, warrior, 0);
    assert!(candidates.contains(&Entity::Object(warrior)));
    assert!(candidates.contains(&Entity::Object(bears)));
    cast_new(&mut t, P0, "Shock", &[Entity::Object(warrior)]);
    redirect_one(&mut t, P1, warrior, warrior);
    t.resolve_all();
    // The 1 damage "redirected" to itself and the other 1: Warrior en-Kor (2/2) dies.
    assert!(t.in_graveyard(P1, "Warrior en-Kor"));
    assert_eq!(damage_marked(&t, bears), 0);
}

#[test]
fn the_en_kor_ability_can_be_activated_any_number_of_times() {
    cr!("614.9", "602.1");
    ruling!(
        "Nomads en-Kor",
        "You can use this ability as much as you want prior to damage being dealt."
    );
    supported("Nomads en-Kor");
    let mut t = TestGame::new(2);
    // Nomads en-Kor (1/1) is the target of Lightning Bolt; in response, it redirects 1
    // damage three times: twice to Hill Giant and once to Grizzly Bears.
    let nomads = t.battlefield(P1, "Nomads en-Kor");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(nomads)]);
    redirect_one(&mut t, P1, nomads, giant);
    redirect_one(&mut t, P1, nomads, giant);
    redirect_one(&mut t, P1, nomads, bears);
    t.resolve_all();
    assert!(t.on_battlefield(nomads));
    assert_eq!(damage_marked(&t, nomads), 0);
    assert_eq!(damage_marked(&t, giant), 2);
    assert_eq!(damage_marked(&t, bears), 1);
    // The shields are used up: the next damage is dealt to Nomads en-Kor.
    cast_new(&mut t, P0, "Shock", &[Entity::Object(nomads)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Nomads en-Kor"));
}

#[test]
fn redirecting_doesnt_stop_trample_damage_being_assigned_to_the_player() {
    cr!("614.9", "702.19b", "510.1c");
    ruling!(
        "Spirit en-Kor",
        "The ability of this card does not do anything to stop Trample damage from being assigned to the defending player."
    );
    supported("Spirit en-Kor");
    let mut t = TestGame::new(2);
    // Colossal Dreadmaw (6/6 trample) is blocked by Spirit en-Kor (2/2), which redirects
    // the next 2 damage to it to Hill Giant.
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    let spirit = t.battlefield(P1, "Spirit en-Kor");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(
        &mut t,
        &[(dreadmaw, Entity::Player(P1))],
        &[(spirit, dreadmaw)],
    );
    redirect_one(&mut t, P1, spirit, giant);
    redirect_one(&mut t, P1, spirit, giant);
    t.advance_to(P0, Step::EndOfCombat);
    // Lethal damage for Spirit en-Kor (2) was assigned to it, and the rest (4) to P1;
    // the 2 assigned to it was then dealt to the Giant.
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(spirit));
    assert_eq!(damage_marked(&t, spirit), 0);
    assert_eq!(damage_marked(&t, giant), 2);
}

#[test]
fn redirected_combat_damage_is_still_combat_damage() {
    cr!("614.9", "510.2");
    ruling!(
        "Lancers en-Kor",
        "When you redirect combat damage it is still combat damage."
    );
    supported("Lancers en-Kor");
    let mut t = TestGame::new(2);
    // Hill Giant (3/3) is blocked by Lancers en-Kor (3/3), which redirects 1 damage to
    // Grizzly Bears.
    let giant = t.battlefield(P0, "Hill Giant");
    let lancers = t.battlefield(P1, "Lancers en-Kor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(lancers, giant)]);
    redirect_one(&mut t, P1, lancers, bears);
    t.advance_to(P0, Step::EndOfCombat);
    let events = damage_events(&t);
    assert!(events.contains(&(giant, Entity::Object(bears), 1, true)));
    assert!(events.contains(&(giant, Entity::Object(lancers), 2, true)));
    assert_eq!(damage_marked(&t, lancers), 2);
    assert_eq!(damage_marked(&t, bears), 1);
}

#[test]
fn more_damage_than_its_toughness_can_be_redirected_to_a_creature() {
    cr!("614.9", "120.3e");
    ruling!(
        "Shaman en-Kor",
        "It is possible to redirect more damage to a creature than that creature’s toughness."
    );
    supported("Shaman en-Kor");
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P1, "Shaman en-Kor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(shaman)]);
    for _ in 0..3 {
        redirect_one(&mut t, P1, shaman, bears);
    }
    t.resolve_all();
    // All 3 damage was dealt to Grizzly Bears (2/2).
    let bolt_damage: Vec<(Entity, u32)> = damage_events(&t)
        .into_iter()
        .map(|(_, e, n, _)| (e, n))
        .collect();
    assert_eq!(bolt_damage, vec![(Entity::Object(bears), 3)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(shaman));
}

#[test]
fn damage_brought_together_by_redirection_is_dealt_at_once() {
    cr!("614.9", "603.2c");
    supported("Boros Reckoner");
    let mut t = TestGame::new(2);
    // Lightning Bolt's 3 damage to Shaman en-Kor is redirected 1 at a time to Boros
    // Reckoner ("Whenever this creature is dealt damage, it deals that much damage to
    // any target."): it's dealt 3 damage once, and its ability triggers once.
    let shaman = t.battlefield(P1, "Shaman en-Kor");
    let reckoner = t.battlefield(P1, "Boros Reckoner");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(shaman)]);
    for _ in 0..3 {
        redirect_one(&mut t, P1, shaman, reckoner);
    }
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Boros Reckoner"));
    let from_reckoner: Vec<(Entity, u32)> = damage_events(&t)
        .into_iter()
        .filter(|(s, _, _, _)| *s == reckoner)
        .map(|(_, e, n, _)| (e, n))
        .collect();
    assert_eq!(from_reckoner, vec![(Entity::Player(P0), 3)]);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn damage_isnt_redirected_to_a_creature_thats_gone() {
    cr!("614.9");
    let mut t = TestGame::new(2);
    let nomads = t.battlefield(P1, "Nomads en-Kor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    redirect_one(&mut t, P1, nomads, bears);
    // The Bears leave; Shock is dealt to Nomads en-Kor.
    cast_new(&mut t, P0, "Unsummon", &[Entity::Object(bears)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[Entity::Object(nomads)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Nomads en-Kor"));
}
