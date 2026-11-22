//! Shared helpers for the tests of rulings batch P142 (`r_p142_*.rs`): repeatable card
//! advantage — "reveal until" and "look at the top card" engines, recursion from the
//! graveyard, cascade-granting permanents, cost reducers and draw replacements. Batch P141
//! covered the same cards, so its helpers are reused.

#![allow(dead_code)]

pub use crate::r_p016_common::lots;
pub use crate::r_p141_common::*;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The names of the cards in `p`'s library, top first.
pub fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// The name of the top card of `p`'s library.
pub fn top_name(t: &TestGame, p: PlayerId) -> String {
    library_names(t, p).first().cloned().unwrap_or_default()
}

/// The number of times a library was shuffled this turn.
pub fn shuffles(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { .. }))
        .count()
}

/// The names of the cards in `p`'s library, sorted (to compare contents, not order).
pub fn library_contents(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v = library_names(t, p);
    v.sort();
    v
}

/// Queues a "no" for `p`'s next yes/no decision.
pub fn no(t: &mut TestGame, p: PlayerId) {
    t.answer_yes(p, false);
}

pub use crate::r_p117_common::exile;
pub use crate::r_p130_common::loyalty;

/// Runs the game (players passing) until something is on the stack and a player has
/// priority: the triggered ability that should be responded to.
pub fn until_stack(t: &mut TestGame) {
    let ok = t.g.run_until(10_000, |g| {
        !g.stack.is_empty()
            && g.turn.stage == mtg_engine::turn::Stage::Priority
            && g.turn.priority.is_some()
    });
    assert!(ok, "nothing was put on the stack");
}

/// Gives `p` `n` counters of the player counter `kind` (experience, energy, ...).
pub fn give_player_counters(t: &mut TestGame, p: PlayerId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Player(p), kind, n, None);
    t.g.flush_events();
    t.settle();
}

/// The candidates of every target choice asked of `p` since decision `from`, flattened.
pub fn offered(t: &TestGame, p: PlayerId, from: usize) -> Vec<Entity> {
    target_candidates(t, p, from).into_iter().flatten().collect()
}

/// Wraps `p`'s agent so that `p`'s next target choice offering what `cards` have become
/// (cards that will move before the choice, CR 400.7: a creature that dies before its
/// dies trigger's targets are chosen) chooses those objects. Other decisions are answered
/// as scripted.
pub fn target_moved(t: &mut TestGame, p: PlayerId, cards: &[ObjectId]) {
    use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
    struct Moved {
        inner: Box<dyn Agent>,
        cards: Vec<ObjectId>,
        done: bool,
    }
    impl Agent for Moved {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if let Decision::ChooseTargets { candidates, .. } = d {
                let now: Vec<Entity> = self
                    .cards
                    .iter()
                    .map(|c| Entity::Object(g.current(*c)))
                    .filter(|e| candidates.contains(e))
                    .collect();
                if !self.done && !now.is_empty() {
                    self.done = true;
                    // Keep the scripted queue in step with the decisions asked.
                    let _ = self.inner.decide(g, p, d);
                    return Answer::Entities(now);
                }
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(Moved {
        inner,
        cards: cards.to_vec(),
        done: false,
    });
}
