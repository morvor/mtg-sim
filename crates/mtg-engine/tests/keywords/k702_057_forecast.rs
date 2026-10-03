//! CR 702.57 Forecast.

use crate::common_k702_011_017::assert_supported;
use mtg_engine::ability::AbilityKind;
use mtg_engine::kw::forecast;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the forecast ability of `card` for `p`, with `targets`.
fn activate_forecast(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    targets: &[Entity],
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let card = t.g.current(card);
    let uid = t
        .g
        .obj(card)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with("Forecast"))
        .map(|a| a.uid)
        .expect("no forecast ability");
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, card, uid);
    t.g.flush_events();
    if r.is_err() {
        t.clear_answers();
    }
    r
}

#[test]
fn a_forecast_ability_is_activated_from_the_hand() {
    cr!("702.57", "702.57a");
    assert_supported("Steeling Stance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let stance = t.hand(P0, "Steeling Stance");
    t.set_step(P0, Step::Upkeep);
    // "Forecast — {W}, Reveal this card from your hand: Target creature gets +1/+1 until
    // end of turn."
    activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
    // The card stays in its owner's hand.
    assert!(t.in_hand(P0, "Steeling Stance"));
    // It isn't an ability of the card anywhere else.
    let on_bf = t.battlefield(P0, "Pride of the Clouds");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    assert!(activate_forecast(&mut t, P0, on_bf, &[]).is_err());
}

#[test]
fn forecast_only_during_its_owners_upkeep_and_once_each_turn() {
    cr!("702.57b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let stance = t.hand(P0, "Steeling Stance");
    // Not in the main phase, nor in an opponent's upkeep.
    t.set_step(P0, Step::PrecombatMain);
    assert!(activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).is_err());
    t.set_step(P1, Step::Upkeep);
    assert!(activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).is_err());
    // In its owner's upkeep, once.
    t.set_step(P0, Step::Upkeep);
    activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).is_err());
    assert_eq!(t.pt(bears), (3, 3));
    // Again in the next upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn the_card_is_revealed_until_it_leaves_the_hand_or_the_upkeep_ends() {
    cr!("702.57b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let stance = t.hand(P0, "Steeling Stance");
    t.set_step(P0, Step::Upkeep);
    assert!(!forecast::is_revealed(&t.g, stance));
    activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).unwrap();
    assert!(forecast::is_revealed(&t.g, stance));
    t.resolve();
    assert!(forecast::is_revealed(&t.g, stance));
    // A step that isn't an upkeep step begins.
    t.advance_to(P0, Step::Draw);
    assert!(!forecast::is_revealed(&t.g, stance));

    // Leaving the hand ends it too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let stance = t.hand(P0, "Steeling Stance");
    t.set_step(P0, Step::Upkeep);
    activate_forecast(&mut t, P0, stance, &[Entity::Object(bears)]).unwrap();
    t.g.discard(P0, stance, None);
    let now = t.g.current(stance);
    assert!(!forecast::is_revealed(&t.g, stance));
    assert!(!forecast::is_revealed(&t.g, now));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn a_forecast_cost_can_tap_creatures_that_just_arrived() {
    cr!("702.57a");
    ruling!(
        "Sky Hussar",
        "including ones you haven't controlled continuously since the beginning of your most recent turn"
    );
    assert_supported("Sky Hussar");
    let mut t = TestGame::new(2);
    let a = t.battlefield_sick(P0, "Savannah Lions");
    let b = t.battlefield_sick(P0, "Coral Merfolk");
    let hussar = t.hand(P0, "Sky Hussar");
    t.set_step(P0, Step::Upkeep);
    activate_forecast(&mut t, P0, hussar, &[]).unwrap();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    t.resolve();
    // "Draw a card."
    assert_eq!(t.hand_size(P0), 2);
}
