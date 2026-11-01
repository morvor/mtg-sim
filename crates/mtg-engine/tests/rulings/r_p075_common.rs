//! Shared helpers for the tests of rulings batch P075 (`r_p075_*.rs`): abilities that put
//! -1/-1 or +1/+1 counters (on one creature or on each of a group), and abilities that grant
//! protection, reach, rebound, shroud, evasion, trample or tap abilities. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_p125_common::{at_p1, choose_color, exile_now, has};
pub use crate::r_p151_common::{act, damage, next_upkeep};
pub use crate::r_p152_common::{gain, sacrifice, set_life};
pub use crate::r_p212_common::{back_face, boost, grant};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, creatures, give_mana_for, stack_library, supported,
    tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, destroy};
pub use crate::r_s20_common::{tap_for_mana, to_beginning_of_combat};
pub use crate::r_s24_common::{enter_together, tapped};

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// The total mana in `p`'s mana pool.
pub fn pool_total(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).mana_pool.total() as u32
}

/// The +1/+1 counters on the object (followed across zone changes).
pub fn plus(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), "+1/+1")
}

/// The -1/-1 counters on the object (followed across zone changes).
pub fn minus(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), "-1/-1")
}

/// `p` casts the real spell `name` from hand (given lands for its mana cost) with the given
/// targets, leaving it on the stack.
pub fn cast_card(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    t.cast(p, c).targets(targets).go()
}

/// The number of decisions since `from` for which `pred` holds.
pub fn count_since(t: &TestGame, from: usize, pred: impl Fn(&Decision) -> bool) -> usize {
    asked_since(t, from).iter().filter(|(_, d)| pred(d)).count()
}

/// Whether the decision is a yes/no question.
pub fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

/// Queues `p`'s choice of mode `i` (an index into the full list of modes).
pub fn choose_mode(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Modes, Answer::Indices(vec![i]));
}

/// Draws a card for `p` (as an effect would) and settles.
pub fn draw(t: &mut TestGame, p: PlayerId) {
    t.g.draw_cards(p, 1);
    t.g.flush_events();
    t.settle();
}

/// The cards in `p`'s hand.
pub fn hand(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).hand.clone()
}

/// The permanents leave the battlefield at the same time, each going to its owner's
/// graveyard (one simultaneous zone change, as a destroy-all effect would), then settles.
pub fn leave_together(t: &mut TestGame, ids: &[ObjectId]) {
    use mtg_engine::ability::LibraryPosition;
    use mtg_engine::events::MoveCause;
    use mtg_engine::object::Zone;
    use mtg_engine::replacement::{EtbInfo, MoveEv};
    let moves = ids
        .iter()
        .map(|id| {
            let obj = t.g.current(*id);
            MoveEv {
                obj,
                to: Zone::Graveyard(t.g.obj(obj).owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: None,
                etb: EtbInfo::default(),
                source: None,
            }
        })
        .collect();
    t.g.move_objects(moves);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}
