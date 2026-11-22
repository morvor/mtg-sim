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

/// Wraps `p`'s agent so that each "choose an option" decision offering the next of
/// `words` is answered with that option, in order (for option lists whose order isn't
/// known in advance); the decisions are still logged, and every other decision gets the
/// scripted answer.
pub fn pick_options(t: &mut TestGame, p: PlayerId, words: &[&str]) {
    use mtg_engine::decision::{Agent, Answer};
    struct Pick {
        inner: Box<dyn Agent>,
        words: std::collections::VecDeque<String>,
    }
    impl Agent for Pick {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            let scripted = self.inner.decide(g, p, d);
            if let (Some(w), Decision::ChooseOption { options, .. }) = (self.words.front(), d) {
                if let Some(i) = options.iter().position(|o| o == w) {
                    self.words.pop_front();
                    return Answer::Index(i);
                }
            }
            scripted
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Pick {
        inner,
        words: words.iter().map(|w| w.to_string()).collect(),
    });
}
