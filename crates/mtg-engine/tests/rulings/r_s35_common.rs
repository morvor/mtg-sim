//! Shared helpers for the tests of rulings batch S35 (`r_s35_*.rs`): rulings shared by
//! many cards about miscellaneous mechanics — kindred, the Ring, dice, snow mana,
//! creature types, Slivers, votes, power/toughness, the party, monarch, venture into
//! Undercity, text changes. (The helpers of batches S01–S29 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A die roll reported this turn: (player, sides, natural result, result).
pub fn die_rolls(t: &TestGame) -> Vec<(PlayerId, u32, u32, u32)> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::DieRolled {
                player,
                sides,
                result,
                natural,
                planar: false,
            } => Some((*player, *sides, *natural, *result)),
            _ => None,
        })
        .collect()
}

/// Loads the dice: the next die rolls have these natural results, in order.
pub fn load_dice(t: &mut TestGame, naturals: &[u32]) {
    t.g.dice.loaded.extend(naturals.iter().copied());
}

/// The options of the "choose a creature type" decisions `p` was asked since decision
/// `from` (each such decision offers the creature types).
pub fn creature_type_options(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseOption { options, .. }
                if *q == p && options.iter().any(|o| o == "Human") =>
            {
                Some(options.clone())
            }
            _ => None,
        })
        .collect()
}
