//! Replacement effects that modify how other permanents enter (CR 614.1d, 614.12), and
//! "it becomes day as this enters" (CR 731).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn opponents_creatures_and_nonbasic_lands_enter_tapped() {
    cr!("614.1d", "614.12");
    assert_supported("Thalia, Heretic Cathar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thalia, Heretic Cathar");
    let their_bears = t.enter(P1, "Grizzly Bears");
    let their_forest = t.enter(P1, "Forest");
    let their_nonbasic = t.enter(P1, "Reliquary Tower");
    let my_bears = t.enter(P0, "Grizzly Bears");
    assert!(t.obj_now(their_bears).tapped);
    assert!(!t.obj_now(their_forest).tapped, "basic land");
    assert!(t.obj_now(their_nonbasic).tapped, "nonbasic land");
    assert!(!t.obj_now(my_bears).tapped, "your own creature");
}

#[test]
fn opponents_creature_spell_enters_tapped() {
    cr!("614.1d", "608.3a");
    assert_supported("Imposing Sovereign");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imposing Sovereign");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn entering_controller_is_the_one_it_will_have() {
    cr!("614.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imposing Sovereign");
    // A card P1 owns put onto the battlefield under P0's control: not an opponent's.
    let id =
        t.g.create_card_object(card("Grizzly Bears"), P1, object::Zone::Nowhere);
    let new =
        t.g.move_object_ev(replacement::MoveEv {
            obj: id,
            to: object::Zone::Battlefield,
            pos: ability::LibraryPosition::Top,
            cause: events::MoveCause::Effect,
            by: Some(P0),
            etb: replacement::EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    assert!(!t.g.obj(new).tapped);
}

#[test]
fn general_enter_tapped_effect_doesnt_affect_itself() {
    cr!("614.12");
    assert_supported("Orb of Dreams");
    let mut t = TestGame::new(2);
    let orb = t.enter(P0, "Orb of Dreams");
    assert!(!t.obj_now(orb).tapped);
    let bears = t.enter(P0, "Grizzly Bears");
    assert!(t.obj_now(bears).tapped);
}

// ---------------------------------------------------------------------------
// Day and night
// ---------------------------------------------------------------------------

#[test]
fn it_becomes_day_as_it_enters() {
    cr!("731.1", "731.1a", "614.1c");
    assert_supported("Firmament Sage");
    let mut t = TestGame::new(2);
    assert_eq!(t.g.day, None);
    let hand = t.hand_size(P0);
    t.enter(P0, "Firmament Sage");
    t.resolve_all();
    assert_eq!(t.g.day, Some(true));
    // Becoming day from neither isn't "night becomes day": no card drawn.
    assert_eq!(t.hand_size(P0), hand);
    // Night becomes day triggers it.
    t.g.set_day(false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn it_doesnt_become_day_if_it_is_night() {
    cr!("731.1");
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    t.enter(P0, "Firmament Sage");
    assert_eq!(t.g.day, Some(false));
}
