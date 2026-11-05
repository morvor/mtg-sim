//! Rulings batch P110 — Nevermore and Null Chamber ("Spells with the chosen name can't be
//! cast"): the name is chosen as the permanent enters (CR 614.12, 614.1c), so there's no
//! window before the prohibition applies; the prohibition stops casting only (CR 601.3,
//! 101.2), not spells already on the stack or putting cards onto the battlefield; it ends
//! when the permanent leaves (CR 611.3a). Split cards and modal double-faced cards have
//! each face's name (CR 709.4a, 712.8).

use crate::r_p110_common::*;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Nevermore enters naming `name`.
fn nevermore(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Nevermore");
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    let n = t.enter(P0, "Nevermore");
    t.settle();
    assert_eq!(t.obj_now(n).choices.card_name.as_deref(), Some(name));
    n
}

#[test]
fn nevermore_doesnt_stop_putting_the_card_onto_the_battlefield() {
    cr!("601.3", "101.2");
    ruling!(
        "Nevermore",
        "Although the named card can't be cast, it can still be put onto the battlefield by a spell or ability (if it's a permanent card)."
    );
    supported("Sneak Attack");
    // Sneak Attack: "{R}: You may put a creature card from your hand onto the battlefield."
    let mut t = TestGame::new(2);
    nevermore(&mut t, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    let sneak = t.battlefield(P0, "Sneak Attack");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    act(&mut t, P0, sneak, "put a creature card", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn nevermore_naming_one_face_of_a_modal_double_faced_card() {
    cr!("712.8", "601.3");
    ruling!(
        "Nevermore",
        "If you name one of the faces of a modal double-faced card, players may still play the face you didn't name."
    );
    // Kazandu Mammoth // Kazandu Valley: naming the Mammoth, the Valley can still be
    // played.
    let mut t = TestGame::new(2);
    nevermore(&mut t, "Kazandu Mammoth");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    let card = t.hand(P1, "Kazandu Mammoth // Kazandu Valley");
    assert!(!can_cast(&mut t, P1, card, CastMethod::Normal));
    assert!(can_play_land(&mut t, P1, card));
    t.play_land(P1, card).unwrap();
    assert_eq!(t.named_on_battlefield("Kazandu Valley").len(), 1);
}

#[test]
fn null_chamber_doesnt_counter_spells_on_the_stack() {
    cr!("601.3", "101.2", "702.8a");
    ruling!(
        "Null Chamber",
        "If you somehow manage to cast this while other spells are on the stack, those spells won't be countered."
    );
    supported("Null Chamber");
    supported("Vedalken Orrery");
    // P1 casts Grizzly Bears; in response P0 casts Null Chamber (as though it had flash,
    // thanks to Vedalken Orrery), both players naming Grizzly Bears.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Vedalken Orrery");
    t.lands(P1, "Forest", 2);
    let b = t.hand(P1, "Grizzly Bears");
    t.cast(P1, b).go();
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer(P1, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    cast_new(&mut t, P0, "Null Chamber", &[]);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Null Chamber").len(), 1);
    let b2 = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    assert!(!can_cast(&mut t, P1, b2, CastMethod::Normal));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn nevermore_applies_as_soon_as_the_name_is_chosen() {
    cr!("614.12", "614.1c", "117.3b");
    ruling!(
        "Nevermore",
        "No one can cast spells or activate abilities between the time a card is named and the time that Nevermore's ability starts to work."
    );
    // P0 casts Nevermore naming Lightning Bolt and the game runs on: at every priority
    // P1 gets after the name was chosen, Bolt can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            let named = g
                .permanents()
                .any(|o| o.chars.name == "Nevermore" && o.choices.card_name.is_some());
            let bolt_castable = g.clone().legal_actions(P1).iter().any(|a| {
                matches!(a, mtg_engine::decision::Action::Cast { card, .. }
                    if g.obj(*card).chars.name == "Lightning Bolt")
            });
            (named, bolt_castable)
        },
    );
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    cast_new(&mut t, P0, "Nevermore", &[]);
    t.advance_to(P0, Step::End);
    let seen = seen.lock().unwrap().clone();
    assert!(seen.contains(&(false, true)), "Bolt castable before");
    assert!(seen.contains(&(true, false)), "priority after");
    assert!(seen.iter().all(|(named, castable)| !(named & castable)));
    let _ = bolt;
}

#[test]
fn nevermore_doesnt_affect_spells_already_on_the_stack() {
    cr!("601.3", "101.2");
    ruling!(
        "Nevermore",
        "Spells with the chosen name that somehow happen to already be on the stack when Nevermore enters are not affected by Nevermore's ability."
    );
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    nevermore(&mut t, "Lightning Bolt");
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn nevermores_name_can_be_cast_again_once_it_leaves() {
    cr!("611.3a", "601.3");
    ruling!(
        "Nevermore",
        "The named card can be cast again once Nevermore leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let n = nevermore(&mut t, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(!can_cast(&mut t, P0, b, CastMethod::Normal));
    destroy(&mut t, n);
    assert!(can_cast(&mut t, P0, b, CastMethod::Normal));
}

#[test]
fn nevermore_naming_half_of_a_split_card() {
    cr!("709.4a", "702.102d", "601.3");
    ruling!(
        "Nevermore",
        "You can name either half of a split card, but not both. If you do so, that half (and both halves, if the split card has fuse) can't be cast. The other half is unaffected."
    );
    // Wear // Tear (fuse): naming both at once isn't a legal choice.
    supported("Nevermore");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Wear // Tear".into()));
    let n = t.enter(P0, "Nevermore");
    assert_eq!(t.obj_now(n).choices.card_name.as_deref(), Some(""));
    // Naming Tear: Tear and the fused spell can't be cast; Wear can.
    let mut t = TestGame::new(2);
    nevermore(&mut t, "Tear");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Plains", 1);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    let card = t.hand(P1, "Wear // Tear");
    assert_eq!(
        legal_cast_methods(&mut t, P1, card),
        vec![CastMethod::Half(0)]
    );
    // A split card without fuse (Fire // Ice): naming Fire, Ice can still be cast.
    let mut t = TestGame::new(2);
    nevermore(&mut t, "Fire");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Island", 1);
    let card = t.hand(P1, "Fire // Ice");
    assert_eq!(
        legal_cast_methods(&mut t, P1, card),
        vec![CastMethod::Half(1)]
    );
}
