//! Shared helpers for the tests of rulings batch S16 (`r_s16_*.rs`): squad, start your
//! engines!, station, storied, storm, strive, support, surge, surveil, survival, suspect,
//! suspend, tempting offer, tiered, time travel, toxic, training. (The helpers of batches
//! S01–S15 are used too.)

#![allow(dead_code)]

use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Sets `p`'s speed (CR 702.179).
pub fn set_speed(t: &mut TestGame, p: PlayerId, speed: u32) {
    t.g.players[p.idx()].speed = Some(speed);
    t.g.recompute();
}

/// Copies of spells named `name` on the stack.
pub fn spell_copies_named(t: &TestGame, name: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.kind == ObjKind::SpellCopy && o.chars.name == name
        })
        .count()
}

/// Puts charge counters on `id` as an effect would (CR 122).
pub fn add_charge(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(
        Entity::Object(id),
        mtg_engine::types::counters::CHARGE,
        n,
        None,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Advances from the current turn of `p` to the beginning of its second (postcombat) main
/// phase: the abilities that trigger then are on the stack and `p` has priority.
pub fn to_second_main(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::PostcombatMain);
    t.settle();
}

/// Names of the permanents `p` controls that are tokens.
pub fn token_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.chars.name.to_string())
        .collect()
}
