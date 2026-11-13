//! CR 606.6: a loyalty ability with a negative loyalty cost can't be activated unless the
//! permanent has at least that many loyalty counters, so it isn't offered as a legal
//! action until it does.

use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_minus_loyalty_ability_isnt_a_legal_action_without_enough_counters() {
    cr!("606.6", "606.3");
    let mut t = TestGame::new(2);
    // "+1: Each player discards a card. −2: Target player sacrifices a creature.
    // −6: ..." with three loyalty counters.
    let liliana = t.enter(P0, "Liliana of the Veil");
    assert_eq!(t.counters(liliana, "loyalty"), 3);
    let uids: Vec<u64> = t
        .obj_now(liliana)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.uid)
        .collect();
    assert_eq!(uids.len(), 3);
    let offered: Vec<u64> = t
        .g
        .legal_actions(P0)
        .iter()
        .filter_map(|a| match a {
            Action::Activate { source, ability } if *source == liliana => Some(*ability),
            _ => None,
        })
        .collect();
    // The +1 and the −2 can be activated; the −6 can't.
    assert_eq!(offered, vec![uids[0], uids[1]]);
}
