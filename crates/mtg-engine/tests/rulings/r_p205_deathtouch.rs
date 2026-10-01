//! Rulings batch P205 — deathtouch (CR 702.2) and daybound (CR 702.145).

use crate::r_s01_common::*;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::to_beginning_of_combat;
use mtg_engine::events::Event;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn a_deathtouch_trampler_dealt_lethal_damage_dies_before_avelines_trigger_resolves() {
    cr!("510.2", "704.5g", "603.3", "702.19c");
    ruling!(
        "Aveline de Grandpré",
        "If a creature you control with deathtouch deals combat damage to a player at the same time it’s dealt lethal damage (perhaps because it has trample and was blocked), it will die before Aveline’s triggered ability resolves and puts +1/+1 counters on it."
    );
    supported("Aveline de Grandpré");
    supported("Rancor");
    let mut t = TestGame::new(2);
    // Aveline (3/3 deathtouch) with Rancor: 5/3 deathtouch and trample.
    let aveline = t.battlefield(P0, "Aveline de Grandpré");
    attach_new(&mut t, P0, "Rancor", aveline);
    let giant = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(aveline, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, aveline)]);
    t.resolve_all();
    // One damage was lethal to the Giant (deathtouch); four trampled over.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 16);
    // Aveline died to the Giant's 3 damage before its trigger resolved: no counters.
    assert!(t.in_graveyard(P0, "Aveline de Grandpré"));
    assert!(t.named_on_battlefield("Aveline de Grandpré").is_empty());
}

#[test]
fn ochran_assassin_destroys_only_as_many_blockers_as_it_deals_damage_to() {
    cr!("702.2b", "510.1c", "509.1c");
    ruling!(
        "Ochran Assassin",
        "Remember that a source with deathtouch must deal damage to a creature for that creature to be destroyed. If five creatures block Ochran Assassin while its power is still 1, only one of them (of Ochran Assassin’s controller’s choice) will be the one dealt damage and destroyed. You’ll have to raise Ochran Assassin’s power to destroy more than one creature."
    );
    supported("Ochran Assassin");
    for (extra_power, destroyed) in [(0u32, 1usize), (2, 3)] {
        let mut t = TestGame::new(2);
        let assassin = t.battlefield(P0, "Ochran Assassin");
        if extra_power > 0 {
            t.g.add_counters(
                Entity::Object(assassin),
                counters::PLUS1,
                extra_power,
                None,
            );
        }
        let bears: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &[(assassin, Entity::Player(P1))]);
        let blocks: Vec<(ObjectId, ObjectId)> = bears.iter().map(|b| (*b, assassin)).collect();
        block_and_finish(&mut t, P1, &blocks);
        let alive = bears.iter().filter(|b| t.on_battlefield(**b)).count();
        assert_eq!(5 - alive, destroyed, "power {}", 1 + extra_power);
    }
}

#[test]
fn a_daybound_spell_cast_at_night_enters_nightbound_face_up_without_transforming() {
    cr!("702.145b", "712.14a");
    ruling!(
        "Werewhat",
        "If you cast a double-faced spell with daybound during night, that spell will be front face up (that is, daybound face up) on the stack. However, it will enter the battlefield with its back face up (that is, with its nightbound face up). It won’t enter the battlefield with its daybound face up and then transform."
    );
    // Werewhat itself isn't supported; Tavern Ruffian // Tavern Smasher is a typical
    // double-faced daybound card.
    supported("Tavern Ruffian // Tavern Smasher");
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    let card = t.hand(P0, "Tavern Ruffian // Tavern Smasher");
    t.lands(P0, "Mountain", 4);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).face, FaceState::Front);
    assert_eq!(t.obj(spell).chars.name, "Tavern Ruffian");
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.on_battlefield(id));
    assert_eq!(t.obj(id).face, FaceState::Back);
    assert_eq!(t.obj(id).chars.name, "Tavern Smasher");
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Transformed { .. })));
}
