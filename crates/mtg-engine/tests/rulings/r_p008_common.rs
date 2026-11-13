//! Shared helpers for rulings batch P008 (`r_p008_*.rs`): burn that can hit planeswalkers
//! ("function: burn planeswalker"). The burn helpers of batch P007 are used too.

#![allow(dead_code)]

use crate::r_p007_common::pl;
use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::events::Event;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `name` into `p`'s hand with the given lands (by name, one each) and casts it with
/// `method`, one target per slot. Returns the spell.
pub fn cast_paying(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    lands: &[&str],
    method: CastMethod,
    targets: &[Entity],
) -> ObjectId {
    supported(name);
    for l in lands {
        t.lands(p, l, 1);
    }
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, card)
        .method(method)
        .try_go()
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// Casts `name` from `p`'s hand with lands for its mana cost and resolves it (one target
/// per slot).
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
    t.resolve_all();
}

/// The damage events since `from` in this turn's events: (source, recipient, amount,
/// combat).
pub fn damage_events(t: &TestGame, from: usize) -> Vec<(ObjectId, Entity, u32, bool)> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                combat,
            } => Some((*source, *target, *amount, *combat)),
            _ => None,
        })
        .collect()
}

/// The amount of damage dealt to `to` since `from`, and whether any of it was combat
/// damage.
pub fn damage_to(t: &TestGame, from: usize, to: Entity) -> (u32, bool) {
    damage_events(t, from)
        .into_iter()
        .filter(|d| d.1 == to)
        .fold((0, false), |(n, c), d| (n + d.2, c || d.3))
}

/// A source P1 controls deals `n` noncombat damage to `to` (as a resolving effect would).
pub fn hit(t: &mut TestGame, src: ObjectId, to: Entity, n: u32) {
    t.g.deal_damage(src, to, n, false);
    t.g.flush_events();
    t.settle();
}

/// The player is dealt `n` damage by `src`.
pub fn hit_player(t: &mut TestGame, src: ObjectId, p: PlayerId, n: u32) {
    hit(t, src, pl(p), n)
}
