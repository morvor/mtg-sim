//! CR 702.50 Epic.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::with_cost;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn snakes(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Snake"))
        .count()
}

/// Casts Endless Swarm for P0 with eight Forests.
fn cast_swarm(t: &mut TestGame) {
    t.lands(P0, "Forest", 8);
    let swarm = t.hand(P0, "Endless Swarm");
    t.cast(P0, swarm).go();
    t.resolve_all();
}

#[test]
fn epic_stops_casting_and_copies_the_spell_each_upkeep() {
    cr!("702.50", "702.50a");
    assert_supported("Endless Swarm");
    let mut t = TestGame::new(2);
    // Endless Swarm: a Snake for each card in hand.
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    cast_swarm(&mut t);
    assert_eq!(snakes(&t, P0), 2);
    assert!(t.in_graveyard(P0, "Endless Swarm"));
    // At the beginning of P0's next upkeep, a copy (without epic) resolves: P0 has two
    // cards in hand then.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(snakes(&t, P0), 4);
    // The copy didn't create another epic trigger: one copy each upkeep.
    assert_eq!(t.g.delayed_triggers.len(), 1);
    // For the rest of the game.
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // Three cards in hand now (a draw step happened in between).
    assert_eq!(snakes(&t, P0), 4 + 3);
}

#[test]
fn enduring_ideal_fetches_an_enchantment_every_upkeep() {
    cr!("702.50a");
    assert_supported("Enduring Ideal");
    let mut t = TestGame::new(2);
    t.g.search_finds_by_default = true;
    t.library_top(P0, "Glorious Anthem");
    t.library_top(P0, "Glorious Anthem");
    t.lands(P0, "Plains", 7);
    let ideal = t.hand(P0, "Enduring Ideal");
    t.cast(P0, ideal).go();
    t.resolve_all();
    let count = |t: &TestGame| t.named_on_battlefield("Glorious Anthem").len();
    assert_eq!(count(&t), 1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(count(&t), 2);
}

#[test]
fn after_epic_resolves_its_controller_cant_cast_spells() {
    cr!("702.50a", "702.50b");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    cast_swarm(&mut t);
    // P0 can't cast spells anymore, for the rest of the game.
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    t.clear_answers();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    // The other player isn't affected.
    t.clear_answers();
    t.lands(P1, "Mountain", 1);
    let b2 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b2).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn a_new_target_may_be_chosen_for_each_copy() {
    cr!("702.50a");
    let def = with_cost(
        custom_card(
            "Eternal Embers",
            "Sorcery",
            None,
            "Eternal Embers deals 2 damage to any target.\nEpic",
        ),
        "{R}",
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 1);
    let embers = t.custom(P0, def, object::Zone::Hand(P0));
    t.cast(P0, embers).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // At P0's next upkeep the copy keeps its target or gets a new one: P2 this time.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P1), 18);
    // The next copy keeps the original target (P1).
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn copies_are_put_onto_the_stack_even_though_spells_cant_be_cast() {
    cr!("702.50b");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    cast_swarm(&mut t);
    let before = snakes(&t, P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    // The copy is on the stack as a spell; it wasn't cast.
    let copies: Vec<ObjectId> = t
        .g
        .stack
        .iter()
        .copied()
        .filter(|s| t.g.obj(*s).kind == object::ObjKind::SpellCopy)
        .collect();
    assert!(copies.is_empty(), "the copy is created as the trigger resolves");
    t.resolve();
    let copies: Vec<ObjectId> = t
        .g
        .stack
        .iter()
        .copied()
        .filter(|s| t.g.obj(*s).kind == object::ObjKind::SpellCopy)
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(!t.obj_now(copies[0]).has_keyword(keywords::KeywordKind::Epic));
    assert!(t.g.history.spells_cast.len() <= 1);
    t.resolve_all();
    assert!(snakes(&t, P0) > before);
}
