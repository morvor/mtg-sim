//! Triggers on who tapped a permanent and why (CR 603.2e, 701.26a, 508.1f): "Whenever you
//! tap an untapped creature an opponent controls" (Solitary Sanctuary, Icewrought Sentry,
//! Hylda of the Icy Crown, Sharae of Numbing Depths) and "if it isn't being declared as an
//! attacker" (Verity Circle, Rhoda, Geist Avenger).

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::ability::{Effect, Filter, Sel};
use mtg_engine::*;

#[test]
fn solitary_sanctuary_triggers_when_you_tap_an_opponents_creature() {
    cr!("603.2e", "701.26a");
    supported("Solitary Sanctuary");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // Its enters ability taps P1's creature; the tap trigger targets mine.
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.enter(P0, "Solitary Sanctuary");
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
}

#[test]
fn solitary_sanctuary_ignores_an_already_tapped_creature_and_the_opponents_own_taps() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Solitary Sanctuary",
        "It won't trigger if a spell or ability you control instructs an opponent to tap a creature they control."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Solitary Sanctuary");
    // Already tapped: P0 tapping it is no change, so no trigger (CR 603.2e).
    t.g.objects[theirs.0 as usize].tapped = true;
    t.answer_targets(P0, &[Entity::Object(mine)]);
    assert!(!t.g.tap_with(theirs, false, Some(P0), false));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A spell P0 controls has P1 tap it: P1 taps it, not P0.
    t.g.objects[theirs.0 as usize].tapped = false;
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::AsPlayer {
            who: mtg_engine::ability::PlayerRef::Player(P1),
            effect: Box::new(Effect::Tap {
                what: Sel::All(Filter::Objects(vec![theirs])),
            }),
        },
        &mut ctx,
    );
    t.settle();
    assert!(t.obj_now(theirs).tapped);
    assert_eq!(t.stack_len(), 0);
    // P1 taps it by attacking with it: P1 tapped it, not P0.
    t.g.objects[theirs.0 as usize].tapped = false;
    t.answer_targets(P0, &[Entity::Object(mine)]);
    declare(&mut t, P1, &[(theirs, Entity::Player(P0))]);
    t.settle();
    assert!(t.obj_now(theirs).tapped);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 0);
}

#[test]
fn icewrought_sentry_pumps_when_its_attack_trigger_taps_a_creature() {
    cr!("603.12", "701.26a");
    supported("Icewrought Sentry");
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Icewrought Sentry");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, P0, &[(sentry, Entity::Player(P1))]);
    mana(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    assert_eq!(t.pt(sentry), (4, 4));
    // Its own controller's attack tapped nothing of an opponent's (vigilance anyway).
    assert!(!t.obj_now(sentry).tapped);
}

#[test]
fn sharae_draws_once_each_turn_for_a_batch_of_creatures_you_tap() {
    cr!("603.2c", "603.2e");
    supported("Sharae of Numbing Depths");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sharae of Numbing Depths");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    // P0 taps two of them at once with an effect, then another.
    let hand = t.hand_size(P0);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::Tap {
            what: Sel::All(Filter::Objects(vec![a, b])),
        },
        &mut ctx,
    );
    t.g.end_event_batch();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::Tap {
            what: Sel::All(Filter::Objects(vec![c])),
        },
        &mut ctx,
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "only once each turn");
}

#[test]
fn verity_circle_ignores_attackers_being_declared() {
    cr!("508.1f", "603.4");
    ruling!(
        "Verity Circle",
        "If a creature with vigilance is declared as an attacker and then becomes tapped while it's attacking, Verity Circle's first ability triggers."
    );
    supported("Verity Circle");
    supported("Rhoda, Geist Avenger");
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Verity Circle");
    let rhoda = t.battlefield(P0, "Rhoda, Geist Avenger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let vigilant = t.battlefield(P1, "Steadfast Sentinel");
    t.lands(P0, "Island", 5);
    // Declared as attackers: the Bears become tapped, the Angel doesn't.
    t.answer_yes(P0, true);
    declare(
        &mut t,
        P1,
        &[(bears, Entity::Player(P0)), (vigilant, Entity::Player(P0))],
    );
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let hand = t.hand_size(P0);
    // Tapped while attacking by Verity Circle's ability: it triggers (both cards).
    t.activate(P0, circle, 0, &[Entity::Object(vigilant)]).ok();
    t.resolve_all();
    assert!(t.obj_now(vigilant).tapped);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(rhoda, "+1/+1"), 1);
}
