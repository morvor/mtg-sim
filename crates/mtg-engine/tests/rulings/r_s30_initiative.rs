//! Rulings batch S30 — the initiative (CR 726): its inherent rules, and two teammates in a
//! Two-Headed Giant game dealing combat damage to the player who has it.

use crate::r_s01_common::{supported, with_subtype};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::designations::take_initiative;
use mtg_engine::dungeons;
use mtg_engine::game::GameConfig;
use mtg_engine::monarch_initiative::{is_inherent, INITIATIVE_STEAL};
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The room of `p`'s venture marker in Undercity.
fn undercity_room(t: &TestGame, p: PlayerId) -> Option<usize> {
    dungeons::marker(&t.g, p).map(|(d, r)| {
        assert_eq!(t.obj_now(d).chars.name.as_str(), "Undercity");
        r
    })
}

/// The inherent "takes the initiative" triggered abilities on the stack: (controller,
/// the player the ability will give the initiative to).
fn steal_triggers(t: &TestGame) -> Vec<PlayerId> {
    t.g.stack
        .iter()
        .filter_map(|id| {
            let o = t.g.obj(*id);
            match o.stack.as_deref().map(|s| &s.kind) {
                Some(StackKind::Triggered { source, .. })
                    if is_inherent(&t.g, *source, INITIATIVE_STEAL) =>
                {
                    Some(o.controller)
                }
                _ => None,
            }
        })
        .collect()
}

#[test]
fn the_initiative_ventures_on_taking_and_upkeep_and_moves_with_combat_damage() {
    cr!("726.1", "726.2", "726.3");
    ruling!(
        "Feywild Caretaker",
        "The initiative is a designation a player can have. A player with the initiative designation is said to “have the initiative.” The initiative carries two inherent rules."
    );
    supported("Feywild Caretaker");
    let mut t = TestGame::new(2);
    // "When this creature enters, you take the initiative." Taking it ventures into
    // Undercity.
    t.enter(P0, "Feywild Caretaker");
    t.resolve_all();
    assert_eq!(t.g.initiative, Some(P0));
    assert_eq!(undercity_room(&t, P0), Some(0));
    // An ability that refers to having the initiative: "At the beginning of your end step,
    // if you have the initiative, create a 1/1 blue Faerie Dragon creature token with
    // flying."
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Faerie").len(), 1);
    // At the beginning of the upkeep of the player with the initiative, that player
    // ventures into Undercity (and only then).
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(t.g.stack.is_empty());
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(undercity_room(&t, P0).unwrap() > 0);
    // One or more creatures P1 controls deal combat damage to P0: P1 takes the initiative
    // (and ventures into Undercity).
    let bear = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(
        &[(bear, Entity::Player(P0)), (lions, Entity::Player(P0))],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.g.initiative, Some(P1));
    assert_eq!(undercity_room(&t, P1), Some(0));
    // Without the initiative, P0's end step ability does nothing (it made a second token
    // in P0's previous end step, while P0 had the initiative).
    assert_eq!(with_subtype(&t, P0, "Faerie").len(), 2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Faerie").len(), 2);
}

#[test]
fn two_teammates_dealing_combat_damage_both_take_the_initiative_in_turn() {
    cr!("726.2", "726.3", "810.7", "603.3b");
    ruling!(
        "Explore the Underdark",
        "In a Two-Headed Giant game, if both players on a team deal combat damage to the player that has the initiative at the same time, the player with the initiative will choose the order of the triggered abilities."
    );
    supported("Explore the Underdark");
    // P0 and P1 against P2 and P3; P2 has the initiative.
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    take_initiative(&mut t.g, P2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.g.initiative, Some(P2));
    let a0 = t.battlefield(P0, "Grizzly Bears");
    let a1 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a0, Entity::Player(P2)), (a1, Entity::Player(P2))]),
    );
    let from = t.asked().len();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    // One ability for each attacking player, both controlled by P2, who ordered them.
    assert_eq!(steal_triggers(&t), vec![P2, P2]);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P2 && matches!(d, Decision::Order { .. })));
    // As they resolve, one teammate takes the initiative and ventures into Undercity, then
    // the other does the same and keeps it.
    t.resolve();
    let first = t.g.initiative.unwrap();
    assert!(first == P0 || first == P1);
    t.resolve();
    assert_eq!(undercity_room(&t, first), Some(0));
    t.resolve_all();
    let last = if first == P0 { P1 } else { P0 };
    assert_eq!(t.g.initiative, Some(last));
    assert_eq!(undercity_room(&t, last), Some(0));
    assert_eq!(undercity_room(&t, first), Some(0));
}
