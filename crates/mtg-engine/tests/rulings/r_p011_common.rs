//! Shared helpers for rulings batch P011 (`r_p011_*.rs`): burn with a set's mechanic,
//! burn that damages you, cantrips, combat-damage changes (Doran), and "card types /
//! cards in graveyard matter". (The helpers of P009, P012, S01 and S25 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts an artifact creature card (artifact, creature), a land card and an instant card
/// into `p`'s graveyard: four card types (delirium). Returns the cards.
pub fn delirium(t: &mut TestGame, p: PlayerId) -> Vec<ObjectId> {
    ["Ornithopter", "Forest", "Opt"]
        .iter()
        .map(|n| t.graveyard(p, n))
        .collect()
}

/// The number of card types among cards in `p`'s graveyard now.
pub fn graveyard_types(t: &TestGame, p: PlayerId) -> usize {
    let mut seen = std::collections::BTreeSet::new();
    for id in &t.g.player(p).graveyard {
        for ct in t.g.obj(*id).chars.card_types.iter() {
            seen.insert(format!("{ct:?}"));
        }
    }
    seen.len()
}

/// The combat damage step of the current combat: advances from wherever combat is to the
/// end of combat step of the active player's turn.
pub fn finish_combat(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, mtg_engine::turn::Step::EndOfCombat);
}
