//! Rulings batch S30 — the order of replacement and prevention effects that modify how
//! damage is dealt (CR 616.1): the player being dealt damage, or the controller of the
//! permanent being dealt damage, chooses — not the controller of the source.

use crate::r_s01_common::supported;
use crate::r_s03_common::respond;
use crate::r_s19_common::add_lore;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s29_common::replacement_choosers;
use crate::r_s30_common::pick_replacement;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn double_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "double")
}

fn plus_2_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "The Flame of Keld")
}

/// P0 controls Furnace of Rath and resolves chapter III of The Flame of Keld, then casts
/// Shock at `target` (P1 or P1's creature), with `p1_choice` answering P1's order
/// choice. Returns the players asked to order replacement effects.
fn shock_with_flame_and_furnace(
    t: &mut TestGame,
    target: Entity,
    p1_choice: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
) -> Vec<PlayerId> {
    // Furnace of Rath: "If a source would deal damage to a permanent or player, it deals
    // double that damage to that permanent or player instead."
    t.battlefield(P0, "Furnace of Rath");
    // The Flame of Keld, III: "If a red source you control would deal damage to a
    // permanent or player this turn, it deals that much damage plus 2 to that permanent or
    // player instead."
    let saga = t.battlefield(P0, "The Flame of Keld");
    add_lore(t, saga, 3);
    t.resolve_all();
    respond(t, P1, p1_choice);
    let from = t.asked().len();
    cast_new(t, P0, "Shock", &[target]);
    t.resolve_all();
    replacement_choosers(t, from)
}

#[test]
fn the_player_being_dealt_damage_orders_the_damage_replacements() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "The Flame of Keld",
        "If multiple replacement effects would modify how damage would be dealt, the player being dealt damage (or the controller of the permanent being dealt damage) chooses the order in which to apply those effects."
    );
    supported("The Flame of Keld");
    supported("Furnace of Rath");
    // P1 applies doubling first: 2 × 2 + 2 = 6.
    let mut t = TestGame::new(2);
    let asked = shock_with_flame_and_furnace(&mut t, Entity::Player(P1), double_first);
    assert_eq!(asked, vec![P1]);
    assert_eq!(t.life(P1), 14);
    // P1 applies "plus 2" first: (2 + 2) × 2 = 8.
    let mut t = TestGame::new(2);
    shock_with_flame_and_furnace(&mut t, Entity::Player(P1), plus_2_first);
    assert_eq!(t.life(P1), 12);
    // Damage to P1's creature (Indomitable Ancients, 2/10): its controller, P1, chooses.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Indomitable Ancients");
    let asked = shock_with_flame_and_furnace(&mut t, Entity::Object(wall), double_first);
    assert_eq!(asked, vec![P1]);
    assert_eq!(t.obj_now(wall).damage, 6);
}

#[test]
fn the_player_who_would_be_dealt_damage_orders_prevention_and_replacement() {
    cr!("616.1", "615.1a");
    ruling!(
        "Consulate Surveillance",
        "If multiple prevention and/or replacement effects are trying to apply to the same damage, the player who would be dealt damage chooses the order in which to apply them."
    );
    supported("Consulate Surveillance");
    let mut t = TestGame::new(2);
    // P1's Furnace of Rath doubles damage; P0's Consulate Surveillance: "Pay {E}{E}:
    // Prevent all damage that would be dealt to you this turn by a source of your choice."
    t.battlefield(P1, "Furnace of Rath");
    t.enter(P0, "Consulate Surveillance");
    t.resolve_all();
    let surveillance = t.named_on_battlefield("Consulate Surveillance")[0];
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(P0).go();
    t.answer_choose(P0, &[Entity::Object(spell)]);
    t.activate(P0, surveillance, 0, &[]).unwrap();
    t.resolve();
    let from = t.asked().len();
    t.resolve_all();
    // P0 (not the Furnace's controller or the source's) chose the order; all the damage
    // was prevented either way.
    assert_eq!(replacement_choosers(&t, from), vec![P0]);
    assert_eq!(t.life(P0), 20);
}

/// P1 casts Remedy ("Prevent the next 5 damage that would be dealt this turn to any
/// number of targets, divided as you choose.") with this division, and it resolves.
fn remedy(t: &mut TestGame, targets: &[Entity], division: &[i64]) {
    lands_for_cost(t, P1, "Remedy");
    let card = t.hand(P1, "Remedy");
    t.answer_targets(P1, targets);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(division.to_vec()));
    t.cast_with(P1, card, &[]).unwrap();
    t.resolve_all();
}

fn remedy_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Remedy")
}

fn furnace_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Furnace of Rath")
}

/// P1 casts Remedy ("Prevent the next 5 damage that would be dealt this turn to any
/// number of targets, divided as you choose."): 1 to P1's Hill Giant and 4 to P1. Then
/// P0, who controls Furnace of Rath, Shocks the Giant, P1 answering the order choice with
/// `choice`. Returns the players asked to order replacement effects, and the Giant.
fn shock_through_remedy_and_furnace(
    t: &mut TestGame,
    choice: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
) -> (Vec<PlayerId>, ObjectId) {
    t.battlefield(P0, "Furnace of Rath");
    let giant = t.battlefield(P1, "Hill Giant");
    remedy(t, &[Entity::Object(giant), Entity::Player(P1)], &[1, 4]);
    t.resolve_all();
    respond(t, P1, choice);
    let from = t.asked().len();
    cast_new(t, P0, "Shock", &[Entity::Object(giant)]);
    t.resolve_all();
    (replacement_choosers(t, from), giant)
}

#[test]
fn remedy_and_angel_of_salvation_are_supported() {
    cr!("601.2d", "615.1a");
    supported("Remedy");
    supported("Angel of Salvation");
    // Each target's share of the prevention is its own shield.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    remedy(&mut t, &[Entity::Object(giant), Entity::Player(P1)], &[2, 3]);
    t.resolve_all();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn angel_of_salvations_trigger_divides_its_prevention_as_it_goes_on_the_stack() {
    cr!("603.3d", "601.2d", "615.1a");
    supported("Angel of Salvation");
    let mut t = TestGame::new(2);
    // "When this creature enters, prevent the next 5 damage that would be dealt this turn
    // to any number of targets, divided as you choose." 3 to Hill Giant, 2 to P1.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P1, &[Entity::Object(giant), Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    let from = t.asked().len();
    t.enter(P1, "Angel of Salvation");
    t.settle();
    // The division was announced as the ability was put on the stack.
    assert_eq!(t.stack_len(), 1);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Divide { total: 5, .. })));
    t.resolve_all();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn the_permanents_controller_orders_prevention_and_doubling() {
    cr!("616.1", "615.1a", "614.1a");
    ruling!(
        "Remedy",
        "If multiple replacement effects apply to a player or permanent being dealt damage, that player or the controller of that permanent chooses the order to apply them, not the controller of the source of damage."
    );
    // Prevention first: 2 - 1 = 1, doubled to 2: the Giant survives.
    let mut t = TestGame::new(2);
    let (asked, giant) = shock_through_remedy_and_furnace(&mut t, remedy_first);
    assert_eq!(asked, vec![P1]);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    // Doubling first: 4 - 1 = 3: the Giant dies.
    let mut t = TestGame::new(2);
    let (asked, _) = shock_through_remedy_and_furnace(&mut t, furnace_first);
    assert_eq!(asked, vec![P1]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}
