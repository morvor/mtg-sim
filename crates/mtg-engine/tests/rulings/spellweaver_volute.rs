//! Spellweaver Volute: "Enchant instant card in a graveyard. Whenever you cast a sorcery
//! spell, copy the enchanted instant card. You may cast the copy without paying its mana
//! cost. If you do, exile the enchanted card and attach this Aura to another instant card in
//! a graveyard." An Aura on the battlefield attached to a card in another zone (CR 303.4a,
//! 702.5a, 704.5m).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 has Spellweaver Volute attached to Lightning Bolt in P0's graveyard (cast normally).
fn setup() -> (TestGame, ObjectId, ObjectId) {
    let c = mtg_engine::card::card("Spellweaver Volute");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Island", 5);
    let v = t.hand(P0, "Spellweaver Volute");
    t.cast(P0, v).target(bolt).go();
    t.resolve();
    let v = t.g.current(v);
    t.settle();
    (t, v, bolt)
}

/// P0 casts Divination (a sorcery); the Volute's ability triggers.
fn cast_sorcery(t: &mut TestGame) {
    t.lands(P0, "Island", 3);
    let d = t.hand(P0, "Divination");
    t.g.turn.priority = Some(P0);
    t.cast(P0, d).go();
    t.settle();
}

#[test]
fn the_aura_targets_and_enchants_an_instant_card_in_a_graveyard() {
    cr!("303.4a", "702.5a", "704.5m");
    ruling!(
        "Spellweaver Volute",
        "This Aura targets, and enchants, an instant card in a graveyard."
    );
    ruling!(
        "Spellweaver Volute",
        "The Aura will be on the battlefield attached to a card in a different zone."
    );
    let (t, v, bolt) = setup();
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).attached_to, Some(Entity::Object(bolt)));
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
}

#[test]
fn casting_a_sorcery_copies_the_card_and_the_aura_moves_to_another_instant() {
    cr!("707.12", "701.3a", "303.4a");
    ruling!(
        "Spellweaver Volute",
        "After you finish casting it, the enchanted card is exiled and you must choose a new instant card in a graveyard to attach Spellweaver Volute to."
    );
    let (mut t, v, bolt) = setup();
    let shock = t.graveyard(P1, "Shock");
    cast_sorcery(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // The copy dealt 3; Divination resolved.
    assert_eq!(t.life(P1), 17);
    // The enchanted card was exiled, and the Volute is attached to the other instant.
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).attached_to, Some(Entity::Object(shock)));
}

#[test]
fn with_no_other_instant_card_the_aura_is_put_into_the_graveyard() {
    cr!("701.3b", "704.5m");
    ruling!(
        "Spellweaver Volute",
        "If you can't, Spellweaver Volute remains on the battlefield attached to nothing and is then put into the graveyard"
    );
    let (mut t, _v, _bolt) = setup();
    cast_sorcery(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Spellweaver Volute"));
}

#[test]
fn a_copy_that_isnt_cast_ceases_to_exist() {
    cr!("704.5e", "707.12");
    ruling!(
        "Spellweaver Volute",
        "If you don't, the copy remains in the graveyard and ceases to exist the next time state-based actions are checked."
    );
    let (mut t, v, bolt) = setup();
    let before = t.graveyard_size(P0);
    cast_sorcery(&mut t);
    t.answer_yes(P0, false);
    t.resolve();
    t.settle();
    // Nothing was cast; the Volute stays on the Bolt; the copy is gone.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(v).attached_to, Some(Entity::Object(bolt)));
    assert_eq!(t.graveyard_size(P0), before);
}

#[test]
fn if_the_enchanted_card_leaves_the_graveyard_the_aura_falls_off() {
    cr!("704.5m", "303.4c");
    ruling!(
        "Spellweaver Volute",
        "If the enchanted card changes zones (due to being cast with flashback, or being exiled with Cremate, for example), Spellweaver Volute \"falls off\""
    );
    let (mut t, _v, bolt) = setup();
    t.g.move_object(bolt, Zone::Exile, events::MoveCause::Effect, Some(P1))
        .unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Spellweaver Volute"));
}

#[test]
fn the_ability_uses_the_card_the_aura_enchanted_as_it_last_existed() {
    cr!("608.2h", "707.12");
    ruling!(
        "Spellweaver Volute",
        "it will check its last existence on the battlefield and identify the \"enchanted card\" as that instant card, so the card is copied"
    );
    let (mut t, v, bolt) = setup();
    cast_sorcery(&mut t);
    // In response, the Volute leaves the battlefield, then the Bolt leaves the graveyard.
    t.g.move_object(v, Zone::Graveyard(P0), events::MoveCause::Effect, Some(P1))
        .unwrap();
    t.g.move_object(bolt, Zone::Exile, events::MoveCause::Effect, Some(P1))
        .unwrap();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn if_the_aura_was_enchanting_nothing_no_copy_is_made() {
    cr!("608.2h", "704.5m");
    ruling!(
        "Spellweaver Volute",
        "it will check its last existence on the battlefield and identify the \"enchanted card\" as \"nothing,\" so no copy is created."
    );
    let (mut t, _v, bolt) = setup();
    cast_sorcery(&mut t);
    // In response, the Bolt is exiled; the Volute is put into the graveyard.
    t.g.move_object(bolt, Zone::Exile, events::MoveCause::Effect, Some(P1))
        .unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Spellweaver Volute"));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}
