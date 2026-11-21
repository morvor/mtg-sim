//! A replacement effect that leaves none of the counters an event would put leaves no
//! counters of that kind (CR 614.1a, 614.6): with Vizier of Remedies ("If one or more
//! -1/-1 counters would be put on a creature you control, that many -1/-1 counters minus
//! one are put on it instead."), Wickerbough Elder ("This creature enters with a -1/-1
//! counter on it.") enters with no -1/-1 counter (CR 614.1c, 122.6), so the only kind of
//! counter on it is Grumgully, the Generous's +1/+1 counter, and Nesting Grounds' "Move a
//! counter" has no kind to choose (CR 122.5).

use crate::r614_audit_common::*;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Wraps `p`'s agent: a choice of a kind of counter takes `kind` if it's offered. Every
/// decision is still logged by the scripted agent, which answers the others.
fn picks_kind(t: &mut TestGame, p: PlayerId, kind: &'static str) {
    struct Kind {
        inner: Box<dyn Agent>,
        kind: &'static str,
    }
    impl Agent for Kind {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
            let scripted = self.inner.decide(g, p, d);
            match d {
                Decision::ChooseOption {
                    prompt, options, ..
                } if prompt.contains("kind of counter") => options
                    .iter()
                    .position(|o| o == self.kind)
                    .map_or(scripted, Answer::Index),
                _ => scripted,
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(Kind { inner, kind });
}

#[test]
fn wickerbough_elder_entering_under_vizier_of_remedies_has_no_minus1_kind_to_move() {
    cr!("614.1a", "614.1c", "122.6", "122.5");
    ruling!(
        "Vizier of Remedies",
        "it enters with that many counters minus one"
    );
    ruling!(
        "Nesting Grounds",
        "You choose which kind of counter to move as that ability resolves."
    );
    supported("Vizier of Remedies");
    supported("Wickerbough Elder");
    supported("Grumgully, the Generous");
    supported("Nesting Grounds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of Remedies");
    t.battlefield(P0, "Grumgully, the Generous");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elder = t.enter(P0, "Wickerbough Elder");
    assert_eq!(t.counters(elder, "-1/-1"), 0);
    assert_eq!(t.counters(elder, "+1/+1"), 1);
    // Nesting Grounds: "{1}, {T}: Move a counter from target permanent you control onto a
    // second target permanent. Activate only as a sorcery." The Elder's only kind of
    // counter is +1/+1: no kind is asked for and the +1/+1 counter moves. (Were the Elder
    // left with an empty "-1/-1" kind, it would be offered, and choosing it would move
    // nothing.)
    let grounds = t.battlefield(P0, "Nesting Grounds");
    t.lands(P0, "Wastes", 1);
    picks_kind(&mut t, P0, "-1/-1");
    let from = t.asked().len();
    t.activate(P0, grounds, 1, &[obj(elder), obj(bears)])
        .expect("Nesting Grounds's second ability");
    t.resolve_all();
    let kinds_asked: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.contains("kind of counter") => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(kinds_asked.is_empty(), "{kinds_asked:?}");
    assert_eq!(t.counters(elder, "+1/+1"), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}
