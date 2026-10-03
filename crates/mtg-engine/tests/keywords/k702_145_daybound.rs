//! CR 702.145 Daybound and Nightbound (with CR 731 day and night).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const RUFFIAN: &str = "Tavern Ruffian // Tavern Smasher";

/// Puts a Tavern Ruffian onto the battlefield under `p`'s control without casting it:
/// Zombify returns it from the graveyard.
fn reanimate(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let card = t.graveyard(p, RUFFIAN);
    t.lands(p, "Swamp", 4);
    let spell = t.hand(p, "Zombify");
    t.cast(p, spell).target(card).go();
    t.resolve_all();
    t.g.current(card)
}

fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

#[test]
fn daybound_and_nightbound_lines_compile() {
    cr!("702.145a");
    for c in [
        RUFFIAN,
        "Village Watch // Village Reavers",
        "Hound Tamer // Untamed Pup",
    ] {
        let def = mtg_engine::card::card(c);
        assert!(def.unsupported_text().is_empty(), "{c}: {:?}", def.unsupported_text());
        // Daybound on the front face, nightbound on the back face.
        let words = |face| -> Vec<String> {
            def.characteristics(face)
                .keywords()
                .filter(|k| k.kind == KeywordKind::DayboundAndNightbound)
                .map(|k| k.text.as_deref().unwrap_or_default().to_lowercase())
                .collect()
        };
        assert_eq!(words(FaceState::Front), ["daybound"], "{c}");
        assert_eq!(words(FaceState::Back), ["nightbound"], "{c}");
    }
}

#[test]
fn a_daybound_permanent_makes_it_day() {
    cr!("702.145d", "731.1");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "Day and night are designations that the game itself can have. The game starts as neither."
    );
    let mut t = TestGame::new(2);
    assert_eq!(t.g.day, None);
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, RUFFIAN);
    t.cast(P0, card).go();
    t.resolve_all();
    let id = t.named_on_battlefield("Tavern Ruffian")[0];
    assert_eq!(t.g.day, Some(true));
    assert_eq!(t.obj_now(id).face, FaceState::Front);
    assert_eq!(t.pt(id), (2, 5));
}

#[test]
fn a_daybound_spell_cast_at_night_enters_transformed() {
    cr!("702.145b");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "If you cast a spell with daybound during night, that spell will be front face up (that is, daybound face up) on the stack. However, it will enter the battlefield with its back face up"
    );
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, RUFFIAN);
    let spell = t.cast(P0, card).go();
    assert_eq!(name(&t, spell), "Tavern Ruffian");
    assert_eq!(t.obj_now(spell).face, FaceState::Front);
    t.resolve_all();
    assert!(t.named_on_battlefield("Tavern Ruffian").is_empty());
    let id = t.named_on_battlefield("Tavern Smasher")[0];
    assert_eq!(t.obj_now(id).face, FaceState::Back);
    assert_eq!(t.pt(id), (6, 5));
    // It entered that way: it didn't transform (no transform event, it's still night).
    assert_eq!(t.g.day, Some(false));
}

#[test]
fn a_daybound_permanent_put_onto_the_battlefield_at_night_enters_transformed() {
    cr!("702.145b");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "If it is night, permanents with daybound that enter the battlefield without being cast will enter with their nightbound faces up."
    );
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    let id = reanimate(&mut t, P0);
    assert_eq!(name(&t, id), "Tavern Smasher");
    // During the day it enters front face up.
    let mut t = TestGame::new(2);
    t.g.set_day(true);
    let id = reanimate(&mut t, P0);
    assert_eq!(name(&t, id), "Tavern Ruffian");
}

#[test]
fn daybound_and_nightbound_permanents_transform_as_it_becomes_night_and_day() {
    cr!("702.145b", "702.145e", "731.1a");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "Double-faced permanents with daybound transform to their nightbound faces as it becomes night."
    );
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "It happens any time it becomes day or night, not just during the untap step."
    );
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, RUFFIAN);
    t.settle();
    assert_eq!(t.g.day, Some(true));
    // An effect makes it night: it transforms at once.
    t.g.set_day(false);
    assert_eq!(name(&t, id), "Tavern Smasher");
    assert_eq!(t.pt(id), (6, 5));
    t.g.set_day(true);
    assert_eq!(name(&t, id), "Tavern Ruffian");
    assert_eq!(t.obj_now(id).face, FaceState::Front);
}

#[test]
fn it_becomes_night_in_the_untap_step_and_daybound_permanents_transform() {
    cr!("702.145b", "731.2a");
    let mut t = TestGame::new(2);
    let id = t.battlefield(P1, RUFFIAN);
    t.settle();
    assert_eq!(t.g.day, Some(true));
    // P0 casts no spells this turn: it becomes night in P1's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.day, Some(false));
    assert_eq!(name(&t, id), "Tavern Smasher");
}

#[test]
fn permanents_with_daybound_or_nightbound_transform_only_through_those_abilities() {
    cr!("702.145b", "702.145e");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "Permanents with daybound and nightbound can't transform via any means other than their daybound and nightbound abilities."
    );
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, RUFFIAN);
    t.settle();
    assert!(!mtg_engine::dfc::transform(&mut t.g, id));
    assert_eq!(name(&t, id), "Tavern Ruffian");
    t.g.set_day(false);
    assert_eq!(name(&t, id), "Tavern Smasher");
    assert!(!mtg_engine::dfc::transform(&mut t.g, id));
    assert_eq!(name(&t, id), "Tavern Smasher");
}

#[test]
fn daybound_wins_when_both_appear_while_its_neither_day_nor_night() {
    cr!("702.145d", "702.145f", "702.145g");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "If it's neither day nor night, and a creature with daybound and a creature with nightbound somehow appear on the battlefield at the same time, it becomes day. The creature with nightbound will transform."
    );
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    let night = reanimate(&mut t, P0);
    assert_eq!(name(&t, night), "Tavern Smasher");
    t.g.day = None;
    let day = t.battlefield(P1, RUFFIAN);
    t.settle();
    assert_eq!(t.g.day, Some(true));
    assert_eq!(name(&t, day), "Tavern Ruffian");
    assert_eq!(name(&t, night), "Tavern Ruffian");
}

#[test]
fn a_nightbound_permanent_alone_makes_it_night() {
    cr!("702.145g");
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    let id = reanimate(&mut t, P0);
    t.g.day = None;
    t.settle();
    assert_eq!(t.g.day, Some(false));
    assert_eq!(name(&t, id), "Tavern Smasher");
}
