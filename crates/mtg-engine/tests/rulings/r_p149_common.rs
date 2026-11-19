//! Shared helpers for the tests of rulings batch P149 (`r_p149_*.rs`): "unless a player
//! pays" (rhystic) abilities, ward creatures and a mix of other rulings: exile-until
//! effects, triggers on damage and draws, tokens and copies, costs and casting
//! permissions. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

pub use crate::r_p003_common::{damage, kill, mana, obj, pl, put};
pub use crate::r_p016_common::{cast_at, lots};
pub use crate::r_p198_common::{act, act_only, pass_turn};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` puts the real card `name` onto the battlefield through a real zone change, with
/// the given answers for its enters trigger's targets (one per slot), and settles (the
/// trigger is on the stack).
pub fn enter_targeting(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// The players asked a decision matching `pred` since decision `from`, in order.
pub fn asked_who(t: &TestGame, from: usize, pred: impl Fn(&Decision) -> bool) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| pred(d))
        .map(|(p, _)| *p)
        .collect()
}

/// `p` draws `n` cards (as an effect would make them) and settles.
pub fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

/// The special actions offered by effects (CR 116.2c: "unless [a player] pays ... before
/// that step") that `p` could take if they held priority now.
pub fn offers(t: &mut TestGame, p: PlayerId) -> Vec<Action> {
    t.g.recompute();
    let prio = t.g.turn.priority.replace(p);
    let v = mtg_engine::special_actions::available(&t.g, p)
        .into_iter()
        .filter(|a| matches!(a, Action::Special(SpecialAction::Offer { .. })))
        .collect();
    t.g.turn.priority = prio;
    v
}

/// `p`, holding priority, takes the first special action offered to them by an effect
/// (with plenty of mana to pay for it).
pub fn pay_offer(t: &mut TestGame, p: PlayerId) {
    lots(t, p);
    t.g.turn.priority = Some(p);
    let a = offers(t, p).into_iter().next().expect("an offer");
    t.g.take_action(p, a);
}

/// `p`'s poison counters.
pub fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.players[p.idx()]
        .counters
        .get("poison")
        .copied()
        .unwrap_or(0)
}
