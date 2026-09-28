//! Shared helpers for the tests of rulings batch S23 (`r_s23_*.rs`): casting (timing and
//! costs of spells cast by effects, "without paying its mana cost", modes, copies) and
//! color. (The helpers of batches S01–S20 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::give_mana_for;
use mtg_engine::ability::{Duration, Effect, Sel};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::text_change::TextWords;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts the real instant or sorcery `name` from `p`'s hand (with the lands to pay for it)
/// targeting `targets` (one per target slot), and resolves it.
pub fn cast_and_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    give_mana_for(t, p, name);
    let card = t.hand(p, name);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
    t.resolve_all();
}

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
