//! Shared helpers for the tests of rulings batch P015 (`r_p015_*.rs`): power- and
//! toughness-defining abilities and the cards that carry them, and spells and abilities
//! that change targets.

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Executes `effect` as if a spell or ability `p` controls resolved.
pub fn run(t: &mut TestGame, p: PlayerId, effect: Effect) {
    let mut ctx = Ctx::new(None, p);
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// `p` gains control of `id` indefinitely.
pub fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let id = t.g.current(id);
    run(
        t,
        p,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![id])),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
    );
}

/// Moves `id` to `zone` (an effect, not a cost) and settles.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) {
    let id = t.g.current(id);
    t.g.move_object(id, zone, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
}

/// Puts `name` onto the battlefield under `p`'s control through a real zone change and
/// puts its enters triggers on the stack.
pub fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

pub fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::ENERGY)
}

/// Whether `id` (followed to its current object) is on the battlefield.
pub fn on_bf(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).zone == Zone::Battlefield
}

/// Where `id` is attacking, if it is.
pub fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attack_target(t.g.current(id)))
}

/// `p`'s creature tokens with the given subtype on the battlefield.
pub fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}
