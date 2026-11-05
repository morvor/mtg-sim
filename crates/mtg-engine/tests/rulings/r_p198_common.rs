//! Shared helpers for the tests of rulings batch P198 (`r_p198_*.rs`): effects that last
//! indefinitely (CR 611.2), control changes (CR 613.1b) and upkeep costs (cumulative
//! upkeep, "sacrifice ~ unless you pay ...").

#![allow(dead_code)]

pub use crate::r_p009_common::{kill, obj, pl};
pub use crate::r_p016_common::{cast_at, cast_multi, lots};
pub use crate::r_p189_common::bounce;
pub use crate::r_p194_common::{damage, mana};
pub use crate::r_s01_common::{creatures, supported, with_subtype};
use mtg_engine::ability::AbilityKind;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` activates the `index`th activated ability of `src` with plenty of mana and the
/// given targets (one per slot), then everything resolves.
pub fn act(t: &mut TestGame, p: PlayerId, src: ObjectId, index: usize, targets: &[Entity]) {
    lots(t, p);
    let src = t.g.current(src);
    t.activate(p, src, index, targets)
        .expect("activation failed");
    t.resolve_all();
}

/// Like [`act`], leaving the ability on the stack.
pub fn act_only(t: &mut TestGame, p: PlayerId, src: ObjectId, index: usize, targets: &[Entity]) {
    lots(t, p);
    let src = t.g.current(src);
    t.activate(p, src, index, targets)
        .expect("activation failed");
}

/// Passes the turn: advances to the precombat main phase of the next player's turn
/// (resolving whatever triggers on the way with the scripted/default answers).
pub fn pass_turn(t: &mut TestGame) {
    let n = t.g.players.len() as u8;
    let next = PlayerId((t.g.turn.active.0 + 1) % n);
    t.advance_to(next, Step::PrecombatMain);
}

/// Advances to `p`'s next upkeep (from any point) and puts the upkeep triggers on the
/// stack.
pub fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        pass_turn(t);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Moves a permanent to exile (as an effect would) and settles.
pub fn exile(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// The controller of the permanent now.
pub fn ctl(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Whether the permanent has an ability whose text contains `text`.
pub fn has_ability(t: &TestGame, id: ObjectId, text: &str) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains(text))
}

/// Number of activated abilities the permanent has now.
pub fn activated_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

/// The creature types of the permanent now, sorted.
pub fn creature_types(t: &TestGame, id: ObjectId) -> Vec<String> {
    let mut v: Vec<String> = t
        .obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .filter(|s| mtg_engine::types::is_creature_type(s))
        .collect();
    v.sort();
    v
}

/// Wraps `p`'s agent: each "choose an option" decision is answered with the first of
/// `words` (in order) that it offers, consuming it; other decisions go to the scripted
/// agent. Returns the option lists offered.
pub fn choose_words(
    t: &mut TestGame,
    p: PlayerId,
    words: &[&str],
) -> std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>> {
    use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
    struct Words {
        inner: Box<dyn Agent>,
        words: Vec<String>,
        seen: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    }
    impl Agent for Words {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if let Decision::ChooseOption { options, .. } = d {
                self.seen.lock().unwrap().push(options.clone());
                if let Some((wi, oi)) = self
                    .words
                    .iter()
                    .enumerate()
                    .find_map(|(wi, w)| options.iter().position(|o| o == w).map(|oi| (wi, oi)))
                {
                    self.words.remove(wi);
                    return Answer::Index(oi);
                }
            }
            self.inner.decide(g, p, d)
        }
    }
    let seen = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(Words {
        inner,
        words: words.iter().map(|s| s.to_string()).collect(),
        seen: seen.clone(),
    });
    seen
}
