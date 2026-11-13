//! Rulings batch P116 — "As an additional cost to cast this spell, sacrifice ..."
//! (CR 601.2h: costs are paid while casting, so no one can respond until the spell has
//! been cast and its costs paid; CR 118.8: the additional cost is exactly what it says).

use crate::r_p116_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts `name` (with lands for its mana cost) sacrificing `victims` (answered to the
/// sacrifice choice), with the given targets. Returns the spell and the number of
/// decisions asked before the cast began.
fn cast_sacrificing(
    t: &mut TestGame,
    name: &str,
    victims: &[ObjectId],
    targets: &[Entity],
) -> (ObjectId, usize) {
    lands_for_cost(t, P0, name);
    let card = t.hand(P0, name);
    let before = n_asked(t);
    let vs: Vec<Entity> = victims.iter().map(|v| obj(*v)).collect();
    t.answer_choose(P0, &vs);
    let spell = t
        .cast_with(P0, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
    (spell, before)
}

/// The sacrificed creatures are already in the graveyard when the spell is on the stack,
/// and no player received priority while it was being cast.
fn assert_no_window(t: &TestGame, spell: ObjectId, victims: &[ObjectId], before: usize) {
    assert!(t.g.stack.contains(&spell));
    for v in victims {
        assert!(!t.on_battlefield(*v));
    }
    assert!(!priority_asked_since(t, before));
}

#[test]
fn the_sacrifice_is_paid_before_anyone_can_respond() {
    cr!("601.2h", "601.2i", "117.3c");
    ruling!(
        "Altar's Reap",
        "Players can only respond once this spell has been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to prevent you from casting this spell."
    );
    supported("Altar's Reap");
    // "As an additional cost to cast this spell, sacrifice a creature. Draw two cards."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (spell, before) = cast_sacrificing(&mut t, "Altar's Reap", &[bears], &[]);
    assert_no_window(&t, spell, &[bears], before);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The opponent may respond only now: killing the sacrificed creature is impossible.
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn the_sacrifice_is_paid_before_anyone_can_respond_other_wordings() {
    cr!("601.2h", "601.2i");
    ruling!(
        "Fling",
        "Players can only respond once this spell has been cast and all its costs have been paid. No one can try to interfere with the creature you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Bankrupt in Blood",
        "Players can respond only after Bankrupt in Blood has been cast and all its costs have been paid. No one can try to destroy the creatures you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Blood Divination",
        "Players can respond only after Blood Divination has been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Corrupted Conviction",
        "Players can't respond to this spell until it's been cast and all its costs have been paid. No one can try to interfere with the creature you sacrificed to prevent you from casting this spell."
    );
    for name in [
        "Fling",
        "Bankrupt in Blood",
        "Blood Divination",
        "Corrupted Conviction",
    ] {
        supported(name);
    }
    // Fling: "deals damage equal to the sacrificed creature's power to any target."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let (spell, before) = cast_sacrificing(&mut t, "Fling", &[giant], &[Entity::Player(P1)]);
    assert_no_window(&t, spell, &[giant], before);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Bankrupt in Blood: two creatures.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let (spell, before) = cast_sacrificing(&mut t, "Bankrupt in Blood", &[a, b], &[]);
    assert_no_window(&t, spell, &[a, b], before);
    // Blood Divination and Corrupted Conviction: one creature.
    for name in ["Blood Divination", "Corrupted Conviction"] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let (spell, before) = cast_sacrificing(&mut t, name, &[a], &[]);
        assert_no_window(&t, spell, &[a], before);
        let hand = t.hand_size(P0);
        t.resolve();
        assert!(t.hand_size(P0) > hand, "{name}");
    }
}

/// Whether P0 can cast `name` with lands for its mana cost and the creatures given.
fn can_cast_with(name: &str, creatures: usize) -> bool {
    let mut t = TestGame::new(2);
    // A target for Severed Strands ("Destroy target creature an opponent controls.").
    t.battlefield(P1, "Hill Giant");
    for _ in 0..creatures {
        t.battlefield(P0, "Grizzly Bears");
    }
    lands_for_cost(&mut t, P0, name);
    let card = t.hand(P0, name);
    can_cast(&mut t, P0, card, mtg_engine::object::CastMethod::Normal)
}

/// Casting `name` with `creatures` creatures, trying to sacrifice all of them: exactly
/// `n` are sacrificed.
fn sacrifices_exactly(name: &str, creatures: usize, n: usize) {
    let mut t = TestGame::new(2);
    let all: Vec<ObjectId> = (0..creatures)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    lands_for_cost(&mut t, P0, name);
    let card = t.hand(P0, name);
    let before = n_asked(&t);
    let vs: Vec<Entity> = all.iter().map(|v| obj(*v)).collect();
    t.answer_choose(P0, &vs);
    t.cast_with(P0, card, &[]).unwrap();
    let choices = entity_choices_since(&t, P0, before);
    assert_eq!(choices.len(), 1, "{name}: {choices:?}");
    assert_eq!((choices[0].0, choices[0].1), (n as u32, n as u32), "{name}");
    let gone = all.iter().filter(|v| !t.on_battlefield(**v)).count();
    assert_eq!(gone, n, "{name}");
}

#[test]
fn exactly_one_creature_is_sacrificed() {
    cr!("118.8", "601.2h");
    ruling!(
        "Corrupted Conviction",
        "You must sacrifice exactly one creature to cast this spell; you can't cast it without sacrificing a creature, and you can't sacrifice additional creatures."
    );
    ruling!(
        "Blood Divination",
        "You must sacrifice exactly one creature to cast Blood Divination; you can’t cast it without sacrificing a creature, and you can’t sacrifice additional creatures."
    );
    for name in [
        "Corrupted Conviction",
        "Fling",
        "Life's Legacy",
        "Pyrrhic Blast",
        "Severed Strands",
        "Village Rites",
        "Blood Divination",
    ] {
        supported(name);
        assert!(!can_cast_with(name, 0), "{name}");
        assert!(can_cast_with(name, 1), "{name}");
    }
    for name in [
        "Corrupted Conviction",
        "Life's Legacy",
        "Village Rites",
        "Blood Divination",
    ] {
        sacrifices_exactly(name, 3, 1);
    }
}

#[test]
fn bankrupt_in_blood_sacrifices_exactly_two() {
    cr!("118.8", "601.2h");
    ruling!(
        "Bankrupt in Blood",
        "You must sacrifice exactly two creatures to cast Bankrupt in Blood; you can’t cast it without sacrificing two creatures, and you can’t sacrifice additional creatures."
    );
    assert!(!can_cast_with("Bankrupt in Blood", 0));
    assert!(!can_cast_with("Bankrupt in Blood", 1));
    assert!(can_cast_with("Bankrupt in Blood", 2));
    sacrifices_exactly("Bankrupt in Blood", 4, 2);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let (_, _) = cast_sacrificing(&mut t, "Bankrupt in Blood", &[a, b], &[]);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn costly_plunder_needs_exactly_one_artifact_or_creature() {
    cr!("118.8", "601.2h");
    ruling!(
        "Costly Plunder",
        "You can't cast Costly Plunder without sacrificing a permanent, and you can't sacrifice additional permanents."
    );
    supported("Costly Plunder");
    // "As an additional cost to cast this spell, sacrifice an artifact or creature."
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Costly Plunder");
    let card = t.hand(P0, "Costly Plunder");
    assert!(!can_cast(
        &mut t,
        P0,
        card,
        mtg_engine::object::CastMethod::Normal
    ));
    // An artifact and a creature: only one of them is sacrificed.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (_, before) = cast_sacrificing(&mut t, "Costly Plunder", &[stone, bears], &[]);
    let choices = entity_choices_since(&t, P0, before);
    assert_eq!((choices[0].0, choices[0].1, choices[0].2), (1, 1, 2));
    assert_eq!(
        [stone, bears]
            .iter()
            .filter(|v| !t.on_battlefield(**v))
            .count(),
        1
    );
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn costly_plunder_a_treasure_sacrificed_for_mana_cant_also_pay_the_cost() {
    cr!("601.2g", "601.2h", "605.3a");
    ruling!(
        "Costly Plunder",
        "You can't sacrifice an artifact to generate mana to pay towards Costly Plunder's cost and also to pay its additional cost."
    );
    // Swamp + Treasure for {1}{B}: the Treasure is the only artifact, so it can't be
    // sacrificed for mana and as the additional cost too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swamp");
    let treasure = create_treasure(&mut t);
    let card = t.hand(P0, "Costly Plunder");
    let hand = t.hand_size(P0);
    assert!(t.cast(P0, card).try_go().is_err());
    assert!(t.g.is_live(treasure));
    assert!(t.in_hand(P0, "Costly Plunder"));
    assert_eq!(t.hand_size(P0), hand);
    // With another land, the Treasure pays the additional cost instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Swamp");
    let treasure2 = create_treasure(&mut t);
    let card = t.hand(P0, "Costly Plunder");
    t.answer_choose(P0, &[obj(treasure2)]);
    t.cast(P0, card).go();
    assert!(!t.g.is_live(treasure2));
}

fn create_treasure(t: &mut TestGame) -> ObjectId {
    crate::r_s02_common::create_token(t, P0, "Treasure")
}
