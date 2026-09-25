//! CR 116.2i: in a Planechase game, rolling the planar die is a special action.

use super::r107_planechase::{add_planar_deck, face_up_names, force_next_roll, planechase_game};
use super::r114_common::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::planechase::{self, PlanarFace, PLANAR_DIE_ACTION};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn roll_action() -> SpecialAction {
    SpecialAction::Other {
        name: PLANAR_DIE_ACTION.into(),
        obj: None,
    }
}

fn can_roll(t: &mut TestGame, p: PlayerId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Special(roll_action()))
}

fn roll(t: &mut TestGame, p: PlayerId) -> bool {
    force_next_roll(t, PlanarFace::Blank);
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(roll_action()))
        .is_ok()
}

fn add_mana(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ManaType::C, n);
}

#[test]
fn rolling_the_planar_die_costs_one_more_each_time_this_turn() {
    cr!("116.2i", "901.9", "901.9a");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere"]);
    planechase::set_starting_plane(&mut t.g);
    t.set_step(P0, Step::PrecombatMain);
    // The first roll this turn costs nothing, and doesn't use the stack; a blank face
    // does nothing.
    assert!(can_roll(&mut t, P0));
    assert!(roll(&mut t, P0));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    // The second costs {1}, the third {2}.
    assert!(!can_roll(&mut t, P0));
    assert!(!roll(&mut t, P0));
    add_mana(&mut t, P0, 1);
    assert!(can_roll(&mut t, P0));
    assert!(roll(&mut t, P0));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    add_mana(&mut t, P0, 1);
    assert!(!roll(&mut t, P0));
    add_mana(&mut t, P0, 1);
    assert!(roll(&mut t, P0));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // Next turn, the first roll is free again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(roll(&mut t, P0));
}

#[test]
fn only_the_active_player_in_a_main_phase_with_an_empty_stack_may_roll() {
    cr!("116.2i", "901.9");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    add_planar_deck(&mut t, P1, &["The Fourth Sphere"]);
    planechase::set_starting_plane(&mut t.g);
    t.set_step(P0, Step::PrecombatMain);
    // Not the non-active player.
    assert!(!can_roll(&mut t, P1));
    assert!(!roll(&mut t, P1));
    // Not with a spell on the stack.
    let gift = t.custom(
        P0,
        free_instant("Quick Gift"),
        mtg_engine::object::Zone::Hand(P0),
    );
    t.cast(P0, gift).go();
    assert!(!can_roll(&mut t, P0));
    t.resolve_all();
    // Not outside the main phases.
    t.set_step(P0, Step::Upkeep);
    assert!(!can_roll(&mut t, P0));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_roll(&mut t, P0));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_roll(&mut t, P0));
}

#[test]
fn rolls_caused_by_effects_dont_count_toward_the_cost() {
    cr!("116.2i", "901.9");
    let mut t = planechase_game(2, false);
    // Stairs to Infinity: "Whenever you roll the planar die, draw a card."
    add_planar_deck(&mut t, P0, &["Stairs to Infinity", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    t.set_step(P0, Step::PrecombatMain);
    // Fractured Powerstone: "{T}: Roll the planar die. Activate only as a sorcery."
    let stone = t.battlefield(P0, "Fractured Powerstone");
    force_next_roll(&mut t, PlanarFace::Blank);
    t.activate(P0, stone, 1, &[]).unwrap();
    t.resolve_all();
    // P0 rolled the planar die (Stairs to Infinity triggered)...
    assert_eq!(t.hand_size(P0), 1);
    // ...but hasn't taken the special action this turn: it still costs nothing.
    assert!(can_roll(&mut t, P0));
    assert!(roll(&mut t, P0));
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Now it costs {1}.
    assert!(!roll(&mut t, P0));
}
