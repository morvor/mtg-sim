//! Shared helpers for rulings batch P004 (`r_p004_*.rs`): "exile ... until this leaves the
//! battlefield" permanents (banish, CR 610.3), temporary exile effects, attack-matters
//! cards and assorted others.

#![allow(dead_code)]

use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// How a banishing card's exile ability is put on the stack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum How {
    /// An enters trigger of a permanent that enters under P0's control.
    Enters,
    /// An Aura cast by P0 on a host of the given card (put onto P0's battlefield).
    Aura(&'static str),
    /// "Whenever this creature attacks" (attacking P1).
    Attacks,
    /// "Whenever this creature is dealt damage".
    Damaged,
    /// Its first activated ability (main phase, mana given).
    Activated,
    /// Hixus: P1's creature deals combat damage to P0 the turn Hixus entered.
    CombatDamageToYou,
}

/// The banishing cards of the batch that exile one target (or "that creature") until they
/// leave the battlefield, with how their exile ability is put on the stack.
pub const BANISHERS: &[(&str, How)] = &[
    ("Alabaster Host Intercessor", How::Enters),
    ("Banishing Light", How::Enters),
    ("Borrowed Time", How::Enters),
    ("Chained to the Rocks", How::Aura("Mountain")),
    ("Chains of Custody", How::Aura("Hill Giant")),
    ("Colossal Whale", How::Attacks),
    ("Deputy of Detention", How::Enters),
    ("Dimensional Exile", How::Aura("Plains")),
    ("Fairgrounds Warden", How::Enters),
    ("Faith Unbroken", How::Aura("Hill Giant")),
    ("Gelatinous Cube", How::Enters),
    ("Glass Casket", How::Enters),
    ("Hieromancer's Cage", How::Enters),
    ("Hixus, Prison Warden", How::CombatDamageToYou),
    ("In the Trenches", How::Activated),
    ("Ixalan's Binding", How::Enters),
    ("Liminal Hold", How::Enters),
    ("Makeshift Binding", How::Enters),
    ("Mystical Tether", How::Enters),
    ("Portable Hole", How::Enters),
    ("Prayer of Binding", How::Enters),
    ("Seam Rip", How::Enters),
    ("Silkwrap", How::Enters),
    ("Stasis Snare", How::Enters),
    ("Stormplain Detainment", How::Enters),
    ("Thopter Arrest", How::Enters),
    ("Trapjaw Tyrant", How::Damaged),
    ("Werefox Bodyguard", How::Enters),
];

pub fn how(name: &str) -> How {
    BANISHERS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, h)| *h)
        .unwrap_or(How::Enters)
}

/// Gives `p` plenty of mana of every color.
pub fn mana(t: &mut TestGame, p: PlayerId) {
    for m in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        t.g.players[p.idx()].mana_pool.add_type(m, 8);
    }
}

/// P0's banisher `name` puts its exile ability on the stack aimed at `target` (a permanent
/// P1 controls). Returns the banisher; the ability is left on the stack (for Hixus, the
/// target is the creature that deals the combat damage).
pub fn banish_trigger(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    supported(name);
    t.g.turn.active = P0;
    match how(name) {
        How::Enters => {
            t.answer_targets(P0, &[obj(target)]);
            let e = t.enter(P0, name);
            t.settle();
            e
        }
        How::Aura(host) => {
            let host = t.battlefield(P0, host);
            crate::r_p208_common::cast_aura_leave_trigger_with(t, P0, name, host, &[obj(target)])
        }
        How::Attacks => {
            let w = t.battlefield(P0, name);
            t.answer_targets(P0, &[obj(target)]);
            crate::r_s01_common::attack_with(t, &[(w, Entity::Player(P1))]);
            w
        }
        How::Damaged => {
            let w = t.battlefield(P0, name);
            t.answer_targets(P0, &[obj(target)]);
            let src = t.battlefield(P1, "Raging Goblin");
            t.g.deal_damage(src, obj(w), 1, false);
            t.g.flush_events();
            t.settle();
            w
        }
        How::Activated => {
            let w = t.battlefield(P0, name);
            t.set_step(P0, Step::PrecombatMain);
            mana(t, P0);
            t.activate(P0, w, 0, &[obj(target)])
                .expect("couldn't activate");
            t.settle();
            w
        }
        How::CombatDamageToYou => {
            // P1 attacks P0 with `target` (unblocked); Hixus entered this turn.
            t.set_step(P1, Step::BeginningOfCombat);
            let h = t.enter(P0, name);
            t.settle();
            t.answer(
                P1,
                DecisionKind::Attackers,
                mtg_engine::decision::Answer::Attackers(vec![(target, Entity::Player(P0))]),
            );
            t.advance_to(P1, Step::CombatDamage);
            t.settle();
            h
        }
    }
}

/// Destroys `id` now (no state-based actions are checked afterwards).
pub fn destroy_now(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
}

/// Like [`destroy_now`], then processes the events (the return happens) without checking
/// state-based actions.
pub fn destroy_and_flush(t: &mut TestGame, id: ObjectId) {
    destroy_now(t, id);
    t.g.flush_events();
}

/// The zone the object (followed across zone changes) is in now.
pub fn zone_now(t: &TestGame, id: ObjectId) -> Zone {
    t.g.obj(t.g.current(id)).zone
}

/// `p` casts the real spell `name` from hand (paying from a full mana pool) with the given
/// targets (one per slot), leaving it on the stack.
pub fn cast_now(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    mana(t, p);
    let c = t.hand(p, name);
    t.cast_with(p, c, targets).expect("couldn't cast")
}
