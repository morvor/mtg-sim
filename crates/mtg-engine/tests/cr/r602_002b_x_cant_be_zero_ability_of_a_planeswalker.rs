//! CR 107.3a, 602.2b: an activated ability with "X can't be 0" can be activated if some
//! value of X that could be announced satisfies it, here 1. The search for that value
//! also counts the loyalty counters of the source (they pay a "−X" loyalty cost); for a
//! planeswalker's {X} ability it mustn't take them away from the mana available.

use crate::r703_common::oracle_card;
use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_planeswalkers_mana_x_ability_with_x_cant_be_0_can_be_activated_for_1() {
    cr!("107.3a", "602.2b");
    let mut t = TestGame::new(2);
    let walker = t.custom(
        P0,
        oracle_card(
            "Test Walker",
            "Legendary Planeswalker — Test",
            "{3}",
            None,
            "+1: You gain 1 life.\n{X}: Draw X cards. X can't be 0.",
        ),
        Zone::Battlefield,
    );
    t.g.objects[walker.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    t.g.recompute();
    t.lands(P0, "Island", 1);
    let draw = t
        .obj_now(walker)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(1)
        .map(|a| a.uid)
        .unwrap();
    assert!(t.g.legal_actions(P0).iter().any(
        |a| matches!(a, Action::Activate { source, ability } if *source == walker && *ability == draw)
    ));
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(1));
    t.activate(P0, walker, 1, &[]).expect("activate the {X} ability");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
