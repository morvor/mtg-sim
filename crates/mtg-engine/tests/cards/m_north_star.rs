//! North Star (hand-written, `src/cards/north_star.rs`): "{4}, {T}: For one spell this
//! turn, you may spend mana as though it were mana of any type to pay that spell's mana
//! cost. (Additional costs are still paid normally.)"

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 in their main phase activates North Star with four Mountains, and has `more`
/// untapped Mountains left.
fn activated(more: usize) -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let star = t.battlefield(P0, "North Star");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, star, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(star).tapped);
    t.lands(P0, "Mountain", more);
    t
}

#[test]
fn north_star_compiles() {
    let def = mtg_engine::card::card("North Star");
    assert!(
        def.unsupported_text().is_empty(),
        "{:?}",
        def.unsupported_text()
    );
}

#[test]
fn one_spell_may_be_paid_for_with_mana_of_any_type_once() {
    cr!("609.4b", "601.2h");
    let mut t = activated(6);
    // Divination ({2}{U}) paid with red mana.
    let div = t.hand(P0, "Divination");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.cast(P0, div).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), 2);
    // The permission is used up: a second blue spell can't be paid for with red mana.
    let div2 = t.hand(P0, "Divination");
    assert!(t.cast(P0, div2).try_go().is_err());
    assert_eq!(t.zone(div2), Zone::Hand(P0));
}

#[test]
fn any_type_includes_colorless_and_the_player_chooses_the_spell() {
    interp!("one-spell-spend-permission");
    cr!("609.4b", "107.4c");
    let mut t = activated(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0 declines the permission for a red spell it can pay for anyway ...
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, false);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // ... and uses it for Spatial Contortion's {C}, paid with red mana.
    let sc = t.hand(P0, "Spatial Contortion");
    t.answer_yes(P0, true);
    t.cast(P0, sc).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn additional_costs_are_still_paid_normally() {
    interp!("one-spell-spend-permission");
    cr!("609.4b", "601.2f");
    // Into the Roil ({1}{U}, kicker {1}{U}) with only red mana: its mana cost may be paid
    // with it, its kicker cost can't, so it isn't offered.
    let mut t = activated(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let roil = t.hand(P0, "Into the Roil");
    t.answer_yes(P0, true);
    t.cast(P0, roil)
        .target(Entity::Object(bears))
        .kicked(true)
        .go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.hand_size(P0) == 0, "not kicked: no card drawn");

    // With an Island, kicked: the Island pays the kicker's {U}, red mana the rest
    // ({1}{U} of the mana cost as though any type, and the kicker's {1}).
    let mut t = activated(3);
    let island = t.battlefield(P0, "Island");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Plains");
    let roil = t.hand(P0, "Into the Roil");
    t.answer_yes(P0, true);
    t.cast(P0, roil)
        .target(Entity::Object(bears))
        .kicked(true)
        .go();
    t.resolve_all();
    assert!(t.obj_now(island).tapped);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Plains"), "kicked: P0 draws a card");
}

#[test]
fn the_permission_lasts_only_this_turn() {
    cr!("609.4b", "514.2");
    let mut t = activated(0);
    t.advance_to(P1, Step::Upkeep);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let div = t.hand(P0, "Divination");
    assert!(t.cast(P0, div).try_go().is_err());
}
