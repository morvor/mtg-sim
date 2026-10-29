//! How many counters a permanent enters with (CR 614.1c, 122.6), replacement effects on
//! other entering permanents computed for their source (CR 614.1d), and values measured
//! for the chosen player (CR 607.2d).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

// ---------------------------------------------------------------------------
// Self ETB amounts
// ---------------------------------------------------------------------------

#[test]
fn kicked_creature_enters_with_additional_time_counters() {
    cr!("614.1c", "702.33d");
    assert_supported("Ravaging Riftwurm");
    let time_counters = |kicked: bool| {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 7);
        let w = t.hand(P0, "Ravaging Riftwurm");
        t.cast(P0, w).kicked(kicked).go();
        t.resolve();
        t.counters(w, "time")
    };
    // Three more than it enters with from vanishing alone.
    assert_eq!(time_counters(true), time_counters(false) + 3);
}

#[test]
fn one_counter_plus_one_for_each_other_creature() {
    cr!("614.1c", "122.6");
    assert_supported("Sheriff of Safe Passage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Grizzly Bears");
    let s = t.enter(P0, "Sheriff of Safe Passage");
    assert_eq!(t.counters(s, "+1/+1"), 3);
}

#[test]
fn stun_counters_equal_to_three_minus_x() {
    cr!("614.1c", "107.3m");
    assert_supported("Slumbering Trudge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Slumbering Trudge");
    t.cast(P0, s).x(1).go();
    t.resolve();
    assert_eq!(t.counters(s, "stun"), 2);
    assert!(t.obj_now(s).tapped);

    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let s = t.hand(P0, "Slumbering Trudge");
    t.cast(P0, s).x(3).go();
    t.resolve();
    assert_eq!(t.counters(s, "stun"), 0);
    assert!(!t.obj_now(s).tapped);
}

#[test]
fn counters_for_instant_and_sorcery_cards_in_all_graveyards() {
    cr!("614.1c", "404.1");
    assert_supported("Academy Elite");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Divination");
    t.graveyard(P1, "Grizzly Bears");
    let e = t.enter(P0, "Academy Elite");
    assert_eq!(t.counters(e, "+1/+1"), 2);
}

#[test]
fn counters_equal_to_life_lost_by_opponents_this_turn() {
    cr!("614.1c", "119.3");
    assert_supported("Cryptborn Horror");
    let mut t = TestGame::new(3);
    t.g.lose_life(P1, 2);
    t.g.lose_life(P2, 3);
    t.g.lose_life(P0, 4);
    let h = t.enter(P0, "Cryptborn Horror");
    assert_eq!(t.counters(h, "+1/+1"), 5);
}

#[test]
fn counters_unless_another_red_spell_was_cast() {
    cr!("614.1c", "601.2i");
    assert_supported("Hotheaded Giant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let g = t.hand(P0, "Hotheaded Giant");
    t.cast(P0, g).go();
    t.resolve();
    // Only the Giant itself was cast.
    assert_eq!(t.counters(g, "-1/-1"), 2);

    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    let g = t.hand(P0, "Hotheaded Giant");
    t.cast(P0, g).go();
    t.resolve();
    assert_eq!(t.counters(g, "-1/-1"), 0);
}

#[test]
fn optional_cost_paid_as_it_enters() {
    cr!("614.1c", "614.12a", "118.12");
    assert_supported("Rescuer Sphinx");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let s = t.enter(P0, "Rescuer Sphinx");
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.counters(s, "+1/+1"), 1);

    // Declining: no counter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    let s = t.enter(P0, "Rescuer Sphinx");
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(s, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// Replacement effects on other permanents, computed for their source
// ---------------------------------------------------------------------------

#[test]
fn others_enter_with_as_many_counters_as_the_source_has() {
    cr!("614.1d", "122.6");
    assert_supported("Bloodspore Thrinax");
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Bloodspore Thrinax");
    t.g.add_counters(Entity::Object(th), "+1/+1", 3, None);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    let theirs = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn others_enter_with_counters_equal_to_the_sources_toughness() {
    cr!("614.1d", "122.6");
    assert_supported("Arwen, Weaver of Hope");
    let mut t = TestGame::new(2);
    let arwen = t.battlefield(P0, "Arwen, Weaver of Hope");
    t.g.add_counters(Entity::Object(arwen), "+1/+1", 2, None);
    // Arwen is now 4/3.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

#[test]
fn creatures_of_either_listed_type_enter_with_a_counter() {
    cr!("614.1d", "122.6");
    assert_supported("Arlinn, Voice of the Pack");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arlinn, Voice of the Pack");
    let wolf = t.enter(P0, "Timberpack Wolf");
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(wolf, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn replacement_effect_functions_from_the_graveyard() {
    cr!("614.1d", "113.6b");
    assert_supported("Dearly Departed");
    let mut t = TestGame::new(2);
    // On the battlefield, it does nothing.
    t.battlefield(P0, "Dearly Departed");
    let v1 = t.enter(P0, "Elite Vanguard");
    assert_eq!(t.counters(v1, "+1/+1"), 0);

    let mut t = TestGame::new(2);
    t.graveyard(P0, "Dearly Departed");
    let v = t.enter(P0, "Elite Vanguard");
    let bears = t.enter(P0, "Grizzly Bears");
    let theirs = t.enter(P1, "Elite Vanguard");
    assert_eq!(t.counters(v, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// "the chosen type" / "the chosen player"
// ---------------------------------------------------------------------------

#[test]
fn players_cant_cast_spells_of_the_chosen_card_type() {
    cr!("607.2d", "601.2");
    assert_supported("Archon of Valor's Reach");
    let mut t = TestGame::new(2);
    // artifact, enchantment, instant, sorcery, or planeswalker: choose instant.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.enter(P0, "Archon of Valor's Reach");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let goblin = t.hand(P1, "Raging Goblin");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    assert!(t.cast(P1, goblin).try_go().is_ok());
}

#[test]
fn power_and_toughness_from_the_chosen_players_hand() {
    cr!("604.3", "607.2d", "613.4a");
    assert_supported("Entropic Specter");
    let mut t = TestGame::new(3);
    t.hand(P1, "Grizzly Bears");
    t.hand(P2, "Grizzly Bears");
    t.hand(P2, "Grizzly Bears");
    t.hand(P2, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let s = t.enter(P0, "Entropic Specter");
    let n = t.hand_size(P2) as i32;
    assert_eq!(t.pt(s), (n, n));
    t.hand(P2, "Grizzly Bears");
    assert_eq!(t.pt(s), (n + 1, n + 1));
}

#[test]
fn power_from_lands_the_chosen_player_controls() {
    cr!("604.3", "607.2d", "613.4a");
    assert_supported("Pallimud");
    let mut t = TestGame::new(2);
    let lands = t.lands(P1, "Mountain", 3);
    t.g.tap(lands[0]);
    t.g.tap(lands[1]);
    let p = t.enter(P0, "Pallimud");
    assert_eq!(t.pt(p), (2, 3));
}
