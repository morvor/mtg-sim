//! Shared helpers for the tests of rulings batch S23 (`r_s23_*.rs`): casting (timing and
//! costs of spells cast by effects, "without paying its mana cost", modes, copies) and
//! color. (The helpers of batches S01–S20 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{Duration, Effect, Sel};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::text_change::TextWords;
use mtg_engine::types::*;
use mtg_engine::*;

/// The colors of the object (followed across zone changes) now, after recomputing.
pub fn colors_now(t: &mut TestGame, id: ObjectId) -> ColorSet {
    t.g.recompute();
    t.obj_now(id).chars.colors
}

/// A set of colors.
pub fn colors_of(cs: &[Color]) -> ColorSet {
    cs.iter().copied().collect()
}

/// A text-changing effect (Sleight of Mind style, CR 612) resolves for `p` on `target`,
/// replacing the `from`th offered word of `words` with the `to`th of the others.
pub fn change_text(
    t: &mut TestGame,
    p: PlayerId,
    target: ObjectId,
    words: TextWords,
    from: usize,
    to: usize,
) {
    t.answer(p, DecisionKind::Option, Answer::Index(from));
    t.answer(p, DecisionKind::Option, Answer::Index(to));
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(target))]];
    t.g.exec(
        &Effect::ChangeText {
            what: Sel::Target(0),
            words,
            exclude: vec![],
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Index of `c` among the color words offered by a text-changing effect, skipping
/// `without` (the word being replaced).
pub fn color_word_idx(c: Color, without: Option<Color>) -> usize {
    Color::ALL
        .iter()
        .filter(|x| Some(**x) != without)
        .position(|x| *x == c)
        .unwrap()
}

/// Wraps `p`'s agent so that the first time `p` is asked to choose objects among which
/// some are named in `names`, those are chosen, in the order of `names` (for cards whose
/// ids aren't known in advance, e.g. cards exiled by the resolving effect); every other
/// decision gets the scripted answer.
pub fn choose_named_once(t: &mut TestGame, p: PlayerId, names: &[&str]) {
    use mtg_engine::decision::{Agent, Decision};
    struct Pick {
        inner: Box<dyn Agent>,
        names: Vec<String>,
        done: bool,
    }
    impl Agent for Pick {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if let (false, Decision::ChooseEntities { candidates, .. }) = (self.done, d) {
                let picked: Vec<Entity> = self
                    .names
                    .iter()
                    .filter_map(|n| {
                        candidates
                            .iter()
                            .find(|e| e.object().is_some_and(|o| g.obj(o).chars.name == *n))
                            .copied()
                    })
                    .collect();
                if !picked.is_empty() {
                    self.done = true;
                    return Answer::Entities(picked);
                }
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Pick {
        inner,
        names: names.iter().map(|n| n.to_string()).collect(),
        done: false,
    });
}
