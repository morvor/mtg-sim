//! Rulings batch P001 — Confluences and Seasons, modal spells whose modes may be chosen
//! more than once (CR 700.2d, 700.2i): copies keep their modes (CR 707.10), a mode with a
//! target can be chosen only with a legal target (CR 700.2a), a spell with some legal
//! targets left still resolves (CR 608.2b), and each mode is a separate event.

use crate::r_p001_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::{cast_new, change_copy_targets, lands_for_cost, spell_copies};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Fiery Confluence's modes: 0 "1 damage to each creature", 1 "2 damage to each
/// opponent", 2 "Destroy target artifact".
const FIERY: &str = "Fiery Confluence";

fn modes_asked_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count()
}

/// The modes offered as available in the mode choices asked since decision `from`.
fn available_modes(t: &TestGame, from: usize) -> Vec<Vec<usize>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_copied_confluence_keeps_its_modes_but_may_get_new_targets() {
    cr!("707.10", "707.10c", "700.2");
    ruling!(
        "Fiery Confluence",
        "If a Confluence is copied, the effect that creates the copy will usually allow you to choose new targets, but you can't choose new modes."
    );
    supported(FIERY);
    supported("Twincast");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Juggernaut");
    lands_for_cost(&mut t, P0, FIERY);
    let card = t.hand(P0, FIERY);
    let spell = t.cast(P0, card).modes(&[2, 1, 1]).target(a).go();
    let modes = chosen_modes(&t, spell);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Twincast", &[obj(spell)]);
    change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(chosen_modes(&t, copy), modes);
    assert_eq!(modes_asked_since(&t, from), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Juggernaut"));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn a_targeted_mode_needs_a_legal_target_and_each_choice_has_its_own() {
    cr!("700.2a", "700.2d", "700.2i", "601.2c");
    ruling!(
        "Fiery Confluence",
        "If a mode requires a target, you can select that mode only if there's a legal target available. Ignore the targeting requirements for modes you don't choose. Each time you select that mode, you can choose a different target, or you can choose the same target."
    );
    ruling!(
        "Season of the Burrow",
        "If a mode requires a target, you can select that mode only if there's a legal target available. Ignore the targeting requirements for modes you don't choose. Each time you select that mode, you can choose a different target, or you can choose the same target."
    );
    supported(FIERY);
    supported("Season of the Burrow");
    // No artifact: "Destroy target artifact" can't be chosen, the other modes can.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, FIERY);
    let card = t.hand(P0, FIERY);
    let from = t.asked().len();
    // An answer choosing it isn't taken.
    let spell = t.cast(P0, card).modes(&[2, 1, 1]).go();
    assert_eq!(available_modes(&t, from), vec![vec![0, 1]]);
    assert!(!chosen_modes(&t, spell).contains(&2));
    t.resolve_all();
    // With an artifact, all three are available.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ornithopter");
    lands_for_cost(&mut t, P0, FIERY);
    let card = t.hand(P0, FIERY);
    let from = t.asked().len();
    t.cast(P0, card).modes(&[1, 1, 1]).go();
    assert_eq!(available_modes(&t, from), vec![vec![0, 1, 2]]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Two artifacts: the mode twice, with different targets or the same one.
    for same in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Ornithopter");
        let b = t.battlefield(P1, "Juggernaut");
        lands_for_cost(&mut t, P0, FIERY);
        let card = t.hand(P0, FIERY);
        let second = if same { a } else { b };
        t.cast(P0, card)
            .modes(&[2, 2, 1])
            .target(a)
            .target(second)
            .go();
        t.resolve_all();
        assert!(!t.on_battlefield(a));
        assert_eq!(t.on_battlefield(b), same);
        assert_eq!(t.life(P1), 18);
    }
    // Season of the Burrow: {P}{P}{P} "Return target permanent card with mana value 3 or
    // less from your graveyard ..." can't be chosen with none there; {P}{P} "Exile target
    // nonland permanent" can be chosen twice with two targets.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Season of the Burrow");
    let card = t.hand(P0, "Season of the Burrow");
    let from = t.asked().len();
    t.cast(P0, card).modes(&[1, 1, 0]).target(a).target(b).go();
    assert_eq!(available_modes(&t, from), vec![vec![0, 1]]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
}

#[test]
fn a_confluence_resolves_if_any_of_its_targets_is_still_legal() {
    cr!("608.2b");
    ruling!(
        "Fiery Confluence",
        "If all targets for the chosen modes become illegal before the Confluence resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    ruling!(
        "Verdant Confluence",
        "If all targets for the chosen modes become illegal before the Confluence resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    // Fiery Confluence: destroy target artifact twice and 2 damage to each opponent.
    for both_gone in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Ornithopter");
        let b = t.battlefield(P1, "Juggernaut");
        lands_for_cost(&mut t, P0, FIERY);
        let card = t.hand(P0, FIERY);
        t.cast(P0, card).modes(&[2, 2, 1]).target(a).target(b).go();
        destroy(&mut t, a);
        if both_gone {
            destroy(&mut t, b);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Juggernaut"));
        // With no legal target, nothing happens: not even the untargeted damage.
        assert_eq!(t.life(P1), if both_gone { 20 } else { 18 });
    }
    // Verdant Confluence: two +1/+1 counters on target creature twice, and return target
    // permanent card from your graveyard to your hand.
    supported("Verdant Confluence");
    for both_gone in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.graveyard(P0, "Hill Giant");
        lands_for_cost(&mut t, P0, "Verdant Confluence");
        let card = t.hand(P0, "Verdant Confluence");
        t.cast(P0, card)
            .modes(&[0, 0, 1])
            .target(bears)
            .target(bears)
            .target(giant)
            .go();
        destroy(&mut t, bears);
        if both_gone {
            let g = t.g.current(giant);
            t.g.exile_object(g, None);
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Hill Giant"), !both_gone);
        assert!(t.in_graveyard(P0, "Verdant Confluence"));
    }
}

#[test]
fn each_fiery_confluence_damage_mode_is_a_separate_event() {
    cr!("700.2d", "615.1");
    ruling!(
        "Fiery Confluence",
        "If the first or second modes are chosen multiple times, each of those modes represents a separate damage-dealing event."
    );
    supported("Guardian Seraph");
    // Guardian Seraph: "If a source an opponent controls would deal damage to you,
    // prevent 1 of that damage." It prevents 1 of each of the three 2-damage events.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Guardian Seraph");
    lands_for_cost(&mut t, P0, FIERY);
    let card = t.hand(P0, FIERY);
    t.cast(P0, card).modes(&[1, 1, 1]).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
