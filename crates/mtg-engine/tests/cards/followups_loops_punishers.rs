//! Repeated processes (CR 608.2c) and punishers paid by another player or by any player
//! (CR 118.12, 118.12a).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn invoke_despair_repeats_for_an_enchantment_and_a_planeswalker() {
    cr!("608.2c", "609.3");
    ruling!(
        "Invoke Despair",
        "You repeat the process for an enchantment and a planeswalker whether the target opponent was able to sacrifice a creature or not."
    );
    supported("Invoke Despair");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Invoke Despair");
    let hand = t.hand_size(P0);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    // The creature is sacrificed; no enchantment and no planeswalker: twice 2 life and a
    // card.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn firemind_s_foresight_finds_instants_with_mana_values_3_2_and_1() {
    cr!("608.2c", "701.23a");
    supported("Firemind's Foresight");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Cancel");
    t.library_top(P0, "Counterspell");
    t.library_top(P0, "Opt");
    t.lands(P0, "Island", 6);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Firemind's Foresight");
    t.cast(P0, spell).go();
    t.resolve();
    for name in ["Cancel", "Counterspell", "Opt"] {
        assert!(t.in_hand(P0, name), "{name}");
    }
}

#[test]
fn torment_of_hailfire_repeats_x_times() {
    cr!("608.2c", "118.12a");
    ruling!("Torment of Hailfire", "If X is 0, Torment of Hailfire resolves with no effect.");
    supported("Torment of Hailfire");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Torment of Hailfire");
    t.cast(P0, spell).x(2).go();
    t.resolve();
    // They decline to sacrifice or discard (the default): 3 life each time.
    assert_eq!(t.life(P1), 14);
}

#[test]
fn sin_prodder_any_opponent_may_have_you_put_that_card_into_your_graveyard() {
    cr!("101.4", "118.12");
    ruling!(
        "Sin Prodder",
        "Each opponent in turn order, starting with the one after you in turn order, may choose to have you put that card into your graveyard."
    );
    supported("Sin Prodder");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sin Prodder");
    t.library_top(P0, "Hill Giant");
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 16);
    // Nobody does: the card goes to your hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sin Prodder");
    t.library_top(P0, "Hill Giant");
    t.answer_yes(P1, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn temporal_extortion_any_player_may_pay_half_their_life() {
    cr!("118.12", "107.1a");
    ruling!(
        "Temporal Extortion",
        "As the triggered ability resolves, the active player gets the option to perform the action."
    );
    supported("Temporal Extortion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Temporal Extortion");
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert!(t.in_graveyard(P0, "Temporal Extortion"));
}

#[test]
fn delaying_shield_lose_life_for_each_counter_unless_you_pay() {
    cr!("118.12a", "122.8");
    supported("Delaying Shield");
    let mut t = TestGame::new(2);
    let shield = t.battlefield(P0, "Delaying Shield");
    t.g.add_counters(Entity::Object(shield), "delay", 3, None);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(shield, "delay"), 0);
    // No mana to pay: 1 life for each of the three.
    assert_eq!(t.life(P0), 17);
}

#[test]
fn rhystic_syphon_unless_target_player_pays() {
    cr!("118.12a");
    supported("Rhystic Syphon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Rhystic Syphon");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (25, 15));
    // The player pays {3}: nothing happens.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.lands(P1, "Plains", 3);
    let spell = t.hand(P0, "Rhystic Syphon");
    t.answer_yes(P1, true);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn tourach_s_chant_unless_they_put_a_counter_on_a_creature() {
    cr!("118.12a", "122.1");
    supported("Tourach's Chant");
    supported("Thelon's Chant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tourach's Chant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.enter(P1, "Forest");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.pt(bears), (1, 1));
    // No creature: they can't put a counter, so they're dealt 3 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tourach's Chant");
    t.enter(P1, "Forest");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
