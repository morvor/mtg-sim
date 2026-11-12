//! Shared helpers for the tests of rulings batch P175 (`r_p175_*.rs`): permanents and
//! spells that tap permanents ("tapper" effects) and the many temporary attacking or
//! copied tokens grouped with them, attack taxes, the Ring tempting you, and casting
//! spells without paying their mana costs.

#![allow(dead_code)]

pub use crate::r_p009_common::{kill, obj, pl};
pub use crate::r_p016_common::{cast_at, cast_multi, lots, stack_targets, target_choices};
pub use crate::r_p194_common::{end_step, mana};
pub use crate::r_p195_common::{attacking, on_stack, tap, tapped, top_is_trigger_of};
pub use crate::r_p196_common::{ability_resolved_from, give_control_until_eot, pump};
pub use crate::r_p198_common::{act, act_only, ctl, exile, pass_turn, to_upkeep};
pub use crate::r_s01_common::{attack_with, creatures, supported, tokens, with_subtype};
use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel, TokenSpec, Value};
use mtg_engine::decision::Decision;
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Gives `p` control of `id` indefinitely (as a resolving "gain control of target
/// permanent" effect would), and settles.
pub fn give_control(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Untaps the permanent (as an effect would) and settles.
pub fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.flush_events();
    t.settle();
}

/// Returns the permanent to its owner's hand (as an effect would) and settles.
pub fn bounce(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    t.g.move_object(id, Zone::Hand(owner), MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// The `(min, max)` of the target choices asked of `p` since decision `from`.
pub fn target_bounds(t: &TestGame, p: PlayerId, from: usize) -> Vec<(u32, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets { min, max, .. } if *q == p => Some((*min, *max)),
            _ => None,
        })
        .collect()
}

/// The number of cards `p` drew this turn.
pub fn drawn(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Drew { player, .. } if *player == p))
        .count()
}

/// The number of tokens created this turn.
pub fn tokens_created(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::TokenCreated { .. }))
        .count()
}

/// `p` creates a token (as a resolving effect would) and settles. Returns the token.
pub fn make_token(t: &mut TestGame, p: PlayerId, spec: TokenSpec) -> ObjectId {
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    let tok = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .expect("no token");
    t.g.flush_events();
    t.settle();
    tok
}
