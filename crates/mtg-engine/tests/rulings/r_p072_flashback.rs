//! Rulings batch P072 — effects that give cards flashback (CR 702.34): the flashback cost
//! equal to the card's mana cost, which cards get it, and when it can be used.

use crate::r_s01_common::{attack_with, supported, tapped_lands};
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s05_common::enter;
use crate::r_s07_common::cast_methods;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

#[test]
fn return_the_past_cast_a_card_again_as_soon_as_it_hits_the_graveyard() {
    cr!("702.34a", "117.3b");
    ruling!(
        "Return the Past",
        "If a card is put into your graveyard during your turn, you can cast it with flashback if it's legal to do so before any other player can take any actions."
    );
    supported("Return the Past");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Return the Past");
    t.lands(P0, "Mountain", 2);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    // P0 receives priority after the resolution (CR 117.3b) and Bolt is castable now.
    assert_eq!(t.g.turn.priority, Some(P0));
    let bolt = t.g.player(P0).graveyard[0];
    assert!(can_cast(&mut t, P0, bolt, FLASHBACK));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).method(FLASHBACK).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn flashback_equal_to_a_missing_mana_cost_cant_be_used() {
    cr!("702.34a", "118.6");
    ruling!(
        "Slickshot Lockpicker",
        "If a card with no mana cost gains flashback equal to its mana cost, it has no flashback cost. It can’t be cast this way."
    );
    let mut t = TestGame::new(2);
    let vision = t.graveyard(P0, "Ancestral Vision");
    t.answer_targets(P0, &[Entity::Object(vision)]);
    enter(&mut t, P0, "Slickshot Lockpicker");
    t.resolve_all();
    t.lands(P0, "Island", 5);
    assert!(!can_cast(&mut t, P0, vision, FLASHBACK));
}

#[test]
fn flashback_with_x_in_the_mana_cost_chooses_x() {
    cr!("702.34a", "601.2b", "107.3a");
    ruling!(
        "Slickshot Lockpicker",
        "If you cast an instant or sorcery with {X} in its mana cost and a flashback cost equal to its mana cost, you still choose the value of X as part of casting the spell and paying that cost."
    );
    // Blaze: "Blaze deals X damage to any target." ({X}{R})
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    t.answer_targets(P0, &[Entity::Object(blaze)]);
    enter(&mut t, P0, "Slickshot Lockpicker");
    t.resolve_all();
    t.lands(P0, "Mountain", 5);
    t.cast(P0, blaze)
        .method(FLASHBACK)
        .x(3)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Blaze"));
}

#[test]
fn the_flashback_card_no_alternative_costs_but_additional_costs() {
    cr!("702.34a", "118.9a", "118.8", "601.2b");
    ruling!(
        "Flashback",
        "If you cast a spell with flashback, you can't pay any alternative costs. You can pay additional costs such as kicker costs. If the spell has any mandatory additional costs, you must pay those to cast the spell with flashback."
    );
    supported("Flashback");
    // Cyclonic Rift given flashback can't also be overloaded.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let rift = t.graveyard(P0, "Cyclonic Rift");
    cast_new(&mut t, P0, "Flashback", &[Entity::Object(rift)]);
    t.resolve_all();
    t.lands(P0, "Island", 7);
    let methods = cast_methods(&mut t, P0, rift);
    assert!(methods.contains(&FLASHBACK));
    assert!(!methods.contains(&OVERLOAD));
    // Burst Lightning given flashback may be kicked.
    let mut t = TestGame::new(2);
    let burst = t.graveyard(P0, "Burst Lightning");
    cast_new(&mut t, P0, "Flashback", &[Entity::Object(burst)]);
    t.resolve_all();
    t.lands(P0, "Mountain", 5);
    t.cast(P0, burst)
        .method(FLASHBACK)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Tormenting Voice: its discard must still be paid.
    let mut t = TestGame::new(2);
    let voice = t.graveyard(P0, "Tormenting Voice");
    cast_new(&mut t, P0, "Flashback", &[Entity::Object(voice)]);
    t.resolve_all();
    t.lands(P0, "Mountain", 2);
    assert!(t.cast(P0, voice).method(FLASHBACK).try_go().is_err());
    let island = t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.cast(P0, voice).method(FLASHBACK).go();
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn backdraft_hellkite_grant_is_fixed_as_its_trigger_resolves() {
    cr!("603.4", "611.2c", "702.34a");
    ruling!(
        "Backdraft Hellkite",
        "Once Backdraft Hellkite has attacked, its ability will resolve and grant the appropriate cards flashback even if Backdraft Hellkite leaves the battlefield before the ability resolves or after it resolves but before you cast the spells."
    );
    ruling!(
        "Backdraft Hellkite",
        "Only instant and sorcery cards in your graveyard when Backdraft Hellkite's triggered ability resolves will gain flashback. Instant and sorcery cards that are put into your graveyard later in the turn won't gain flashback."
    );
    supported("Backdraft Hellkite");
    let mut t = TestGame::new(2);
    let hellkite = t.battlefield(P0, "Backdraft Hellkite");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    attack_with(&mut t, &[(hellkite, Entity::Player(P1))]);
    // The Hellkite leaves before its trigger resolves.
    destroy(&mut t, hellkite);
    t.resolve_all();
    let shock = t.graveyard(P0, "Shock");
    t.lands(P0, "Mountain", 2);
    assert!(can_cast(&mut t, P0, bolt, FLASHBACK));
    assert!(!can_cast(&mut t, P0, shock, FLASHBACK));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).method(FLASHBACK).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
