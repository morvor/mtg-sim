//! Rulings batch P189 — Take for a Ride ("has flash as long as you've committed a crime
//! this turn", CR 700.13, 702.8a), and general rulings of C.A.M.P. (Fortifications, CR
//! 301.6) and Mausoleum Secrets (land colors, CR 105.2c).

use crate::r_p076_common::mana;
use crate::r_s01_common::supported;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn take_for_a_ride_targets_any_creature() {
    cr!("115.1", "113.3b");
    ruling!(
        "Take for a Ride",
        "Take for a Ride can target any creature, even one that’s untapped or one you already control."
    );
    supported("Take for a Ride");
    // An untapped creature an opponent controls.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield_sick(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 3);
    let card = t.hand(P0, "Take for a Ride");
    t.cast(P0, card).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.obj(theirs).controller, P0);
    assert!(!t.obj(theirs).tapped);
    assert!(t.obj(theirs).chars.has_keyword(KeywordKind::Haste));
    // A tapped creature you already control: it untaps and gains haste.
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.objects[mine.0 as usize].tapped = true;
    mana(&mut t, P0, ManaType::R, 3);
    let card = t.hand(P0, "Take for a Ride");
    t.cast(P0, card).target(mine).go();
    t.resolve_all();
    assert_eq!(t.obj(mine).controller, P0);
    assert!(!t.obj(mine).tapped);
    assert!(t.obj(mine).chars.has_keyword(KeywordKind::Haste));
}

#[test]
fn take_for_a_ride_has_flash_after_a_crime() {
    cr!("700.13", "702.8a", "307.1");
    supported("Take for a Ride");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 3);
    let card = t.hand(P0, "Take for a Ride");
    // No crime yet: it can't be cast on an opponent's turn.
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, card).target(bears).try_go().is_err());
    t.clear_answers();
    // Shock targeting the opponent is a crime.
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.g.turn.priority = Some(P0);
    t.cast(P0, card).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj(bears).controller, P0);
}

#[test]
fn camp_fortifications_follow_equipment_rules() {
    cr!("301.6", "702.67a");
    ruling!(
        "C.A.M.P.",
        "Fortification is to lands what Equipment is to creatures. Except for the difference regarding what type of permanent they can be attached to, Fortifications and Equipment follow the same rules."
    );
    // Darksteel Garrison (fully supported): "Fortify {3}".
    supported("Darksteel Garrison");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 6);
    // Like equip, fortify can't target the wrong kind of permanent...
    if t.activate(P0, garrison, 0, &[Entity::Object(bears)]).is_ok() {
        t.resolve_all();
    }
    assert_ne!(t.obj(garrison).attached_to, Some(Entity::Object(bears)));
    t.activate(P0, garrison, 0, &[Entity::Object(forest)])
        .expect("fortify a land you control");
    t.resolve_all();
    assert_eq!(t.obj(garrison).attached_to, Some(Entity::Object(forest)));
    // ...and the Fortification stays on the battlefield, unattached, if the land leaves.
    t.g.move_object(forest, Zone::Hand(P0), events::MoveCause::Effect, None);
    t.settle();
    assert!(t.on_battlefield(garrison));
    assert_eq!(t.obj(garrison).attached_to, None);
}

#[test]
fn mausoleum_secrets_swamp_is_colorless() {
    cr!("105.2c", "202.2");
    ruling!(
        "Mausoleum Secrets",
        "A land card that produces black mana, even a Swamp, normally has no color."
    );
    let mut t = TestGame::new(2);
    let swamp = t.library_top(P0, "Swamp");
    let on_bf = t.battlefield(P0, "Swamp");
    assert!(t.obj(swamp).chars.colors.is_colorless());
    assert!(t.obj(on_bf).chars.colors.is_colorless());
    t.activate(P0, on_bf, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 1);
}
