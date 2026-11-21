//! Rulings batch P092 — assorted: Tithe Taker's tax, Shortcut to Mushrooms' end-step
//! check, Dire Fleet Ravager's life loss, Barbed Servitor, Bad Deal, Asylum Visitor, and
//! Bolas's Citadel's look at the top card.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_cast, destroy};
use crate::r_s06_common::damage;
use crate::r_s20_common::tap_for_mana;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn tithe_taker_taxes_free_activated_abilities_but_not_mana_or_triggered_abilities() {
    cr!("602.2b", "601.2f", "605.1a", "603.1");
    ruling!(
        "Tithe Taker",
        "An opponent's activated ability that costs no mana to activate will cost {1} plus its non-mana costs during your turn."
    );
    ruling!(
        "Tithe Taker",
        "An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate. Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected by Tithe Taker."
    );
    supported("Tithe Taker");
    supported("Prodigal Sorcerer");
    // It's P0's turn. P1's Prodigal Sorcerer ({T}: 1 damage) needs {1} more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tithe Taker");
    let sorc = t.battlefield(P1, "Prodigal Sorcerer");
    assert!(!can_activate(&mut t, P1, sorc));
    let land = t.battlefield(P1, "Mountain");
    assert!(can_activate(&mut t, P1, sorc));
    t.activate(P1, sorc, 0, &[Entity::Player(P0)]).unwrap();
    assert!(t.obj_now(land).tapped);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // P1's Llanowar Elves (a mana ability) isn't taxed.
    let elves = t.battlefield(P1, "Llanowar Elves");
    assert!(tap_for_mana(&mut t, P1, elves, "Add"));
    assert!(t.obj_now(elves).tapped);
    // P1's triggered ability (Thragtusk's) works as usual.
    t.answer_yes(P1, true);
    let life = t.life(P1);
    t.enter(P1, "Thragtusk");
    t.resolve_all();
    assert_eq!(t.life(P1), life + 5);
}

#[test]
fn tithe_taker_increase_applies_before_reductions_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3");
    ruling!(
        "Tithe Taker",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of Tithe Taker's effect), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Baral, Chief of Compliance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tithe Taker");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    // {R} + {1}: one Mountain isn't enough.
    assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal));
    // Baral's reduction ({1} less) removes the increase: {R}.
    t.battlefield(P1, "Baral, Chief of Compliance");
    assert!(can_cast(&mut t, P1, bolt, CastMethod::Normal));
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

/// Shortcut to Mushrooms on P0's battlefield; runs to P0's end step and resolves.
fn mushrooms_end_step(t: &mut TestGame, target: ObjectId) -> usize {
    t.answer_targets(P0, &[obj(target)]);
    t.advance_to(P0, Step::End);
    t.settle();
    let n = t.stack_len();
    t.resolve_all();
    n
}

#[test]
fn shortcut_to_mushrooms_sees_permanents_that_left_before_it_arrived() {
    cr!("603.4", "603.10");
    ruling!(
        "Shortcut to Mushrooms",
        "Shortcut to Mushrooms doesn't need to have been on the battlefield when the permanent you controlled left the battlefield."
    );
    supported("Shortcut to Mushrooms");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.battlefield(P0, "Shortcut to Mushrooms");
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(mushrooms_end_step(&mut t, giant), 1);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
}

#[test]
fn shortcut_to_mushrooms_triggers_once_and_only_if_something_already_left() {
    cr!("603.4");
    ruling!(
        "Shortcut to Mushrooms",
        "Shortcut to Mushrooms's last ability will trigger only once during your end step, no matter how many permanents you controlled left the battlefield this turn. However, if no permanents you control have left the battlefield so far this turn as your end step begins, the ability won't trigger at all. It's not possible to cause a permanent you control to leave the battlefield during the end step in time to have the ability trigger."
    );
    // Several left: one trigger, one counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shortcut to Mushrooms");
    for name in ["Grizzly Bears", "Llanowar Elves", "Forest"] {
        let id = t.battlefield(P0, name);
        destroy(&mut t, id);
    }
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(mushrooms_end_step(&mut t, giant), 1);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    // Nothing left: no trigger; a permanent leaving during the end step is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shortcut to Mushrooms");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(mushrooms_end_step(&mut t, giant), 0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.counters(giant, "+1/+1"), 0);
}

/// Dire Fleet Ravager enters under P0's control and its ability resolves.
fn ravager(t: &mut TestGame) {
    supported("Dire Fleet Ravager");
    t.enter(P0, "Dire Fleet Ravager");
    t.resolve_all();
}

#[test]
fn dire_fleet_ravager_rounds_up() {
    cr!("107.1a", "119.3");
    ruling!(
        "Dire Fleet Ravager",
        "For example, if you have 13 life, you'll lose 5 life."
    );
    ruling!(
        "Dire Fleet Ravager",
        "If a player has 1 life, that player loses 1 life. If each player has 0 life after that, the game's a draw."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].life = 13;
    t.g.players[1].life = 20;
    ravager(&mut t);
    assert_eq!(t.life(P0), 8);
    assert_eq!(t.life(P1), 13);
    // Both at 1: both at 0, a draw.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 1;
    t.g.players[1].life = 1;
    ravager(&mut t);
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.life(P1), 0);
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn dire_fleet_ravager_in_two_headed_giant_uses_the_team_life_total() {
    cr!("810.9", "810.9a", "810.2");
    ruling!(
        "Dire Fleet Ravager",
        "In a Two-Headed Giant game, each player loses a third of the team's life total rounded up. For example, if a team has 13 life, each player on that team loses 5 life and the team loses 10 life total."
    );
    let mut t = two_headed_giant();
    // Team 1 (P2, P3) at 13 life; team 0 at 30.
    t.g.players[2].life = 13;
    t.g.players[3].life = 13;
    assert_eq!(t.life(P0), 30);
    ravager(&mut t);
    // Each player of the team at 13 loses 5: the team loses 10.
    assert_eq!(t.life(P2), 3);
    assert_eq!(t.life(P3), 3);
    // Team 0 at 30: each player loses 10; the team loses 20.
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.life(P1), 10);
}

#[test]
fn barbed_servitor_dealt_more_than_its_toughness() {
    cr!("120.3e", "120.6", "702.12b");
    ruling!(
        "Barbed Servitor",
        "A creature can be dealt an amount of damage greater than its toughness. For example, if Barbed Servitor is dealt 3 damage, its last ability causes the target opponent to lose 3 life."
    );
    supported("Barbed Servitor");
    let mut t = TestGame::new(2);
    let servitor = t.battlefield(P0, "Barbed Servitor");
    let source = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, source, 3, servitor);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(servitor));
}

#[test]
fn bad_deal_draws_then_opponents_discard_in_turn_order_then_life_loss() {
    cr!("101.4", "608.2c", "701.9a");
    ruling!(
        "Bad Deal",
        "As Bad Deal resolves, first you draw two cards. Then the next opponent in turn order (or, if it's an opponent's turn, the opponent whose turn it is) chooses two cards in hand without revealing them, then each other opponent in turn order does the same. All the chosen cards are discarded at the same time. Then each player loses 2 life."
    );
    supported("Bad Deal");
    let mut t = TestGame::new(3);
    for p in [P1, P2] {
        for name in ["Grizzly Bears", "Hill Giant", "Forest"] {
            t.hand(p, name);
        }
    }
    give_mana_for(&mut t, P0, "Bad Deal");
    let c = t.hand(P0, "Bad Deal");
    t.cast(P0, c).go();
    let hand0 = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand0 + 2);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.graveyard_size(P2), 2);
    for p in [P0, P1, P2] {
        assert_eq!(t.life(p), 18);
    }
    // P1 chose before P2.
    let choosers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(p, d)| *p != P0 && matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(choosers.first(), Some(&P1), "{choosers:?}");
    assert_eq!(choosers.last(), Some(&P2), "{choosers:?}");
}

#[test]
fn a_second_asylum_visitor_does_nothing_after_the_first_draws() {
    cr!("603.4");
    ruling!(
        "Asylum Visitor",
        "Asylum Visitor's triggered ability checks the active player's hand as the upkeep begins and as the trigger resolves. If that player has a card in hand as it resolves, you won't draw a card or lose 1 life."
    );
    supported("Asylum Visitor");
    let mut t = TestGame::new(2);
    // P1 controls two; P1's upkeep begins with P1's hand empty.
    t.battlefield(P1, "Asylum Visitor");
    t.battlefield(P1, "Asylum Visitor");
    assert_eq!(t.hand_size(P1), 0);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    // The first drew a card; the second saw it in P1's hand and did nothing.
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn bolass_citadel_lets_you_look_at_the_top_card_without_priority() {
    cr!("401.5", "116.1");
    ruling!(
        "Bolas's Citadel",
        "Bolas's Citadel lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority."
    );
    supported("Bolas's Citadel");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Forest");
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    t.battlefield(P0, "Bolas's Citadel");
    let top = t.library_top(P0, "Mountain");
    t.advance_to(P1, Step::Upkeep);
    // P1 has priority in P1's turn; P0 still may look; P1 may not.
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, t.g.current(top)));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, t.g.current(top)));
}
