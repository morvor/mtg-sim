//! Rules checks made while fuzzing.

use mtg_engine::decision::PassiveAgent;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

/// A rules violation seen during a game.
#[derive(Clone, Debug)]
pub struct Violation {
    pub turn: u32,
    pub what: String,
}

/// Wraps an agent and, before a sample of its priority decisions, checks that the game
/// performed every state-based action before giving the player priority (CR 117.5,
/// 704.3): state-based actions checked on a copy of the game must find nothing to do.
pub struct CheckingAgent<A: Agent> {
    pub inner: A,
    /// Check every `every`th priority decision.
    pub every: u32,
    seen: u32,
    pub violations: Arc<Mutex<Vec<Violation>>>,
}

impl<A: Agent> CheckingAgent<A> {
    pub fn new(inner: A, every: u32, violations: Arc<Mutex<Vec<Violation>>>) -> Self {
        CheckingAgent {
            inner,
            every: every.max(1),
            seen: 0,
            violations,
        }
    }
}

impl<A: Agent> Agent for CheckingAgent<A> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if matches!(d, Decision::Priority { .. }) && g.result.is_none() {
            self.seen += 1;
            if self.seen % self.every == 0 {
                let mut copy = g.clone();
                copy.set_agents(
                    (0..g.players.len())
                        .map(|_| Box::new(PassiveAgent) as Box<dyn Agent>)
                        .collect(),
                );
                copy.logging = true;
                let before = copy.log.len();
                if copy.check_sbas() {
                    let what: Vec<String> =
                        copy.log[before..].iter().map(|l| l.text.clone()).collect();
                    self.violations.lock().unwrap().push(Violation {
                        turn: g.turn.number,
                        what: format!(
                            "{p:?} got priority with state-based actions pending: {}",
                            if what.is_empty() {
                                "(nothing logged)".to_string()
                            } else {
                                what.join("; ")
                            }
                        ),
                    });
                }
            }
        }
        self.inner.decide(g, p, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::agents::RandomAgent;
    use mtg_engine::decision::Action;
    use mtg_engine::testing::TestGame;

    #[test]
    fn reports_priority_given_with_a_state_based_action_pending() {
        let mut t = TestGame::new(2);
        let found: Arc<Mutex<Vec<Violation>>> = Arc::default();
        let mut agent = CheckingAgent::new(RandomAgent::new(1), 1, found.clone());
        let priority = Decision::Priority {
            actions: vec![Action::Pass],
        };
        agent.decide(&t.g, PlayerId(0), &priority);
        assert!(found.lock().unwrap().is_empty());
        // A player at 0 life loses the next time state-based actions are checked
        // (CR 704.5a), which must happen before anyone gets priority.
        t.g.players[1].life = 0;
        agent.decide(&t.g, PlayerId(0), &priority);
        let v = found.lock().unwrap();
        assert_eq!(v.len(), 1, "{v:?}");
        // The game itself is untouched.
        assert!(!t.has_lost(PlayerId(1)));
    }
}
