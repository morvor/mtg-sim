//! Shared helpers for the tests of rulings batch P095 (`r_p095_*.rs`): life payments and
//! life loss (paying life as a cost versus losing life as an effect), life gain and the
//! abilities that care about it, and the cards of the "life payment" Tagger function.
//! (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to `p`'s next upkeep (from any point) and puts the upkeep triggers on the
/// stack.
pub fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    crate::r_p138_common::to_upkeep(t, p);
}

/// Advances to the beginning of `p`'s end step (from earlier in `p`'s turn) and puts the
/// end step triggers on the stack.
pub fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

/// Sets `p`'s life total directly (no life is gained or lost).
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    crate::r_s33_common::set_life(t, p, life);
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost, then casts it
/// with the given targets (one per target slot). Returns the spell.
pub fn cast_new(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    crate::r_s25_common::cast_new(t, p, name, targets)
}

/// P1 casts Cancel ("Counter target spell.") targeting `spell` and it resolves.
pub fn counter_with_cancel(t: &mut TestGame, spell: ObjectId) {
    cast_new(t, P1, "Cancel", &[Entity::Object(spell)]);
    t.resolve();
}

/// The amounts of the life-loss events of `p` since index `from` of this turn's events.
pub fn losses_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    crate::r_p096_common::losses_since(t, from, p)
}

/// The amounts of the life-gain events of `p` since index `from` of this turn's events.
pub fn gains_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    crate::r_s33_common::life_gains_since(t, from, p)
}

/// The amounts of life `p` paid since index `from` of this turn's events.
pub fn paid_life_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<i32> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Custom {
                name,
                player: Some(q),
                amount,
                ..
            } if *q == p && name.as_str() == mtg_engine::life_totals::PAID_LIFE => Some(*amount),
            _ => None,
        })
        .collect()
}

/// The number of triggered or activated abilities on the stack whose source is `source`.
pub fn abilities_from(t: &TestGame, source: ObjectId) -> usize {
    crate::r_s25_common::abilities_from(t, source).len()
}

/// Queues `p`'s answer to a "choose X" decision.
pub fn answer_x(t: &mut TestGame, p: PlayerId, x: i64) {
    t.answer(p, DecisionKind::X, Answer::Number(x));
}

/// Moves a permanent to its owner's hand (as an effect would) and settles.
pub fn bounce(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Hand(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
    t.g.flush_events();
}

/// `source` deals `n` (noncombat) damage to `to`, and the game settles.
pub fn damage(t: &mut TestGame, source: ObjectId, to: Entity, n: u32) {
    crate::r_p041_common::damage(t, source, to, n);
}

/// `p` gains `n` life (one life-gaining event, as an effect would), and the game settles.
pub fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// `p` loses `n` life (as an effect would), and the game settles.
pub fn lose(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}
