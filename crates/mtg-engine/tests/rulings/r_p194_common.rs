//! Shared helpers for the tests of rulings batch P194 (`r_p194_*.rs`): abilities that
//! look back over the turn (life gained or lost, damage dealt, creatures that died,
//! cards put into graveyards, permanents that entered), unpreventable damage, and
//! tokens.

#![allow(dead_code)]

pub use crate::r_p009_common::{gain, kill, lose, obj, pl};
pub use crate::r_p016_common::target_choices;
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Total mana left in `p`'s pool.
pub fn pool(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).mana_pool.total() as u32
}

/// Builds a game with `setup`, then `p` tries to cast the real card `name` from their
/// hand with exactly `colored` plus `generic` colorless mana in their pool. Whether it
/// was cast with all of that mana spent.
pub fn casts_for(
    setup: &dyn Fn(&mut TestGame),
    p: PlayerId,
    name: &str,
    colored: &[(ManaType, u32)],
    generic: u32,
) -> bool {
    let mut t = TestGame::new(2);
    setup(&mut t);
    let c = t.hand(p, name);
    for (ty, n) in colored {
        mana(&mut t, p, *ty, *n);
    }
    mana(&mut t, p, ManaType::C, generic);
    t.cast(p, c).try_go().is_ok() && pool(&t, p) == 0
}

/// Asserts that, after `setup`, `name` costs exactly `colored` plus `generic` generic
/// mana: it can be cast for that, and not for one generic less.
pub fn assert_total_cost(
    setup: &dyn Fn(&mut TestGame),
    p: PlayerId,
    name: &str,
    colored: &[(ManaType, u32)],
    generic: u32,
) {
    assert!(
        casts_for(setup, p, name, colored, generic),
        "{name}: not castable for {colored:?} + {generic}"
    );
    if generic > 0 {
        assert!(
            !casts_for(setup, p, name, colored, generic - 1),
            "{name}: castable for {colored:?} + {}",
            generic - 1
        );
    }
}

/// `caster` casts a real Shock at `target` and it resolves.
pub fn shock(t: &mut TestGame, caster: PlayerId, target: Entity) {
    let s = t.hand(caster, "Shock");
    mana(t, caster, ManaType::R, 1);
    t.cast(caster, s).target(target).go();
    t.resolve_all();
}

/// `source` deals `n` noncombat damage to `to` (as a resolving effect would), then
/// settles.
pub fn damage(t: &mut TestGame, source: ObjectId, to: Entity, n: u32) {
    t.g.deal_damage(source, to, n, false);
    t.g.flush_events();
    t.settle();
}

/// Advances to P0's end step and resolves everything that triggers there.
pub fn end_step(t: &mut TestGame) {
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

/// The damage events this turn as (source, recipient, amount).
pub fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}
