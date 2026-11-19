//! Shared helpers for the tests of item `rulings-residue-counters` (`r_residue_counters_*.rs`):
//! the open rulings left in batches P035, P119, P120 and P121 — counters being put,
//! removed, moved, doubled and counted. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::turn::Step;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p002_common::replacement_choices;
pub use crate::r_p035_common::{put, to_end_step, total_counters};
pub use crate::r_p103_common::prefer_replacement;
pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_p119_common::{cast_slots, enter_with, giant_growth, with_counters};
pub use crate::r_p120_common::{gain_life, give_minus1, give_plus1, plus1, tokens_named};
pub use crate::r_p121_common::{cast_resolve, damage, exile_now, power, remove_plus1};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, custom_card, place, stack_library, supported,
    triggers_on_stack,
};
pub use crate::r_s02_common::{create_token, destroy};
pub use crate::r_s24_common::{controller, enter_together, tapped};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// The damage marked on the object (followed across zone changes).
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(t.g.current(id)).damage
}

/// Removes `n` counters of `kind` from the permanent as one event and settles.
pub fn remove(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.remove_counters(Entity::Object(id), kind, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Whether a decision of the given kind was asked of `p` since decision `from`.
pub fn was_asked(t: &TestGame, p: PlayerId, from: usize, kind: fn(&Decision) -> bool) -> bool {
    t.asked()[from..].iter().any(|(q, d)| *q == p && kind(d))
}

/// The amounts of damage dealt to `target` this turn, in order.
pub fn dealt_to(t: &TestGame, target: Entity) -> Vec<u32> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                target: x, amount, ..
            } if *x == target => Some(*amount),
            _ => None,
        })
        .collect()
}

/// Advances to P0's next upkeep and puts its triggered abilities on the stack.
pub fn to_upkeep(t: &mut TestGame) {
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

/// P0 casts Dismantle on `target`; the counters go on P0's Mind Stone (as +1/+1 counters
/// unless `charge`). Returns the Mind Stone.
pub fn dismantle(t: &mut TestGame, target: ObjectId, meddle: impl FnOnce(&mut TestGame)) -> ObjectId {
    let stone = t.battlefield(P0, "Mind Stone");
    t.set_step(P0, Step::PrecombatMain);
    cast_new(t, P0, "Dismantle", &[obj(target)]);
    meddle(t);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[obj(stone)]);
    t.answer_targets(P0, &[obj(stone)]);
    t.resolve_all();
    stone
}

/// The +1/+1 and charge counters on the object.
pub fn dismantled(t: &TestGame, id: ObjectId) -> u32 {
    plus1(t, id) + t.counters(id, "charge")
}

/// P0's Nissa with `loyalty` loyalty counters activates her 0 ability with `top` on top of
/// P0's library. Returns (Nissa, the top card, the index of the first decision asked).
pub fn nissa_zero(t: &mut TestGame, loyalty: u32, top: &str) -> (ObjectId, ObjectId, usize) {
    t.set_step(P0, Step::PrecombatMain);
    let n = t.battlefield(P0, "Nissa, Steward of Elements");
    t.g.objects[n.0 as usize]
        .counters
        .insert("loyalty".into(), loyalty);
    t.g.dirty = true;
    let card = t.library_top(P0, top);
    let from = t.asked().len();
    t.activate(P0, n, 1, &[]).unwrap();
    (n, card, from)
}

/// P0's Xolatoyac enters, putting a flood counter on P0's Forest. Returns (Xolatoyac,
/// the Forest).
pub fn flood(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[obj(forest)]);
    let x = t.enter(P0, "Xolatoyac, the Smiling Flood");
    t.resolve_all();
    assert_eq!(t.counters(forest, "flood"), 1);
    (x, forest)
}

/// The rules texts of the object's abilities now.
pub fn ability_texts(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect()
}

/// P0's Daxos makes a Spirit token. Returns it.
pub fn daxos_spirit(t: &mut TestGame) -> ObjectId {
    let d = t.battlefield(P0, "Daxos the Returned");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve_all();
    let spirits = crate::r_s01_common::with_subtype(t, P0, "Spirit");
    assert_eq!(spirits.len(), 1);
    spirits[0]
}

/// P0's Combine Guildmage moves a +1/+1 counter from `from` onto `to`; `meddle` runs with
/// the ability on the stack.
pub fn guildmage_move(
    t: &mut TestGame,
    from: ObjectId,
    to: ObjectId,
    meddle: impl FnOnce(&mut TestGame),
) {
    let gm = t.battlefield(P0, "Combine Guildmage");
    t.lands(P0, "Island", 2);
    t.activate(P0, gm, 1, &[obj(from), obj(to)]).unwrap();
    meddle(t);
    t.resolve_all();
}
