//! Rulings batch S05 — eminence (an ability word): triggered abilities of commanders that
//! function "if [this] is in the command zone or on the battlefield" (CR 113.6b, 603.4).

use crate::r_s01_common::*;
use crate::r_s05_common::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const ARAHBO: &str = "Arahbo, Roar of the World";

/// A Commander game with Arahbo in P0's command zone as their commander, Savannah Lions
/// (a 2/1 Cat) on P0's battlefield, in P0's precombat main phase.
fn setup() -> (TestGame, ObjectId, ObjectId) {
    supported(ARAHBO);
    supported("Savannah Lions");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let arahbo = t.command(P0, ARAHBO);
    t.g.objects[arahbo.0 as usize].is_commander = true;
    t.g.players[0].commander_names.push(ARAHBO.into());
    let lions = t.battlefield(P0, "Savannah Lions");
    (t, arahbo, lions)
}

#[test]
fn eminence_triggers_from_the_command_zone_and_the_battlefield_only() {
    cr!("113.6b", "603.4");
    ruling!(
        "Arahbo, Roar of the World",
        "This commander's eminence ability is a triggered ability. The commander must be on the battlefield or in the command zone as the trigger event occurs and also as the triggered ability resolves."
    );
    // Arahbo: "Eminence — At the beginning of combat on your turn, if Arahbo is in the
    // command zone or on the battlefield, another target Cat you control gets +3/+3 until
    // end of turn."
    // From the command zone.
    let (mut t, _arahbo, lions) = setup();
    t.answer_targets(P0, &[Entity::Object(lions)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(lions), (5, 4));
    // From the battlefield.
    let (mut t, arahbo, lions) = setup();
    let arahbo = move_to(&mut t, arahbo, Zone::Battlefield).unwrap();
    assert!(t.on_battlefield(arahbo));
    t.answer_targets(P0, &[Entity::Object(lions)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(lions), (5, 4));
    // Not from the graveyard or exile (its owner chose not to put it into the command
    // zone), or a hand.
    for zone in [Zone::Graveyard(P0), Zone::Exile, Zone::Hand(P0)] {
        let (mut t, arahbo, lions) = setup();
        t.answer_yes(P0, false);
        move_to(&mut t, arahbo, zone);
        assert_eq!(t.zone(arahbo), zone);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
    t.settle();
        assert_eq!(t.stack_len(), 0, "triggered from {zone:?}");
        assert_eq!(t.pt(lions), (2, 1));
    }
}

#[test]
fn eminence_does_nothing_if_the_commander_left_its_zone_before_it_resolves() {
    cr!("603.4", "400.7");
    ruling!(
        "Arahbo, Roar of the World",
        "If the commander is in an appropriate zone as the trigger event occurs but leaves that zone, the ability won't do anything as it resolves."
    );
    // It triggers in the command zone, then Arahbo goes to P0's hand.
    let (mut t, arahbo, lions) = setup();
    t.answer_targets(P0, &[Entity::Object(lions)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, arahbo, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.pt(lions), (2, 1));
}

#[test]
fn eminence_from_the_battlefield_does_nothing_once_the_commander_is_in_the_command_zone() {
    cr!("603.4", "400.7", "903.9a");
    ruling!(
        "Arahbo, Roar of the World",
        "Notably, if your commander is on the battlefield and its eminence ability triggers, but it's put into the command zone before that ability resolves, that ability won't do anything as it resolves. This is because an object that changes zones is considered a new object."
    );
    let (mut t, arahbo, lions) = setup();
    let arahbo = move_to(&mut t, arahbo, Zone::Battlefield).unwrap();
    t.answer_targets(P0, &[Entity::Object(lions)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Arahbo dies and its owner puts it into the command zone.
    t.answer_yes(P0, true);
    t.g.destroy(arahbo, None);
    t.settle();
    assert_eq!(t.zone(arahbo), Zone::Command);
    t.resolve_all();
    assert_eq!(t.pt(lions), (2, 1));
}
