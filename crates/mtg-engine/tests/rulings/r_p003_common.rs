//! Shared helpers for rulings batch P003 (`r_p003_*.rs`): effects that make artifacts,
//! lands and themselves creatures (layers 4 and 7b, CR 613), "the amount of mana spent",
//! artifactfall, attacking matters, anthems, divided damage and artifactify.

#![allow(dead_code)]

use crate::r_s01_common::supported;
use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Adds the mana of `cost` (e.g. "2UU": two colorless, two blue) to `p`'s pool.
pub fn mana(t: &mut TestGame, p: PlayerId, cost: &str) {
    for c in cost.chars() {
        let ty = match c {
            'W' => ManaType::W,
            'U' => ManaType::U,
            'B' => ManaType::B,
            'R' => ManaType::R,
            'G' => ManaType::G,
            'C' => ManaType::C,
            d => {
                let n = d.to_digit(10).expect("a mana symbol");
                t.g.players[p.idx()].mana_pool.add_type(ManaType::C, n);
                continue;
            }
        };
        t.g.players[p.idx()].mana_pool.add_type(ty, 1);
    }
}

/// `p` activates the activated ability of `src` whose text contains `needle`, with the
/// mana of `cost` in their pool and the given targets (one per target slot). The ability
/// stays on the stack.
pub fn activate(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    needle: &str,
    cost: &str,
    targets: &[Entity],
) {
    mana(t, p, cost);
    for x in targets {
        t.answer_targets(p, &[*x]);
    }
    crate::r_s06_common::activate_containing(t, p, src, needle)
        .unwrap_or_else(|e| panic!("activating {needle:?}: {e:?}"));
}

/// As [`activate`], then everything resolves.
pub fn activate_resolve(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    needle: &str,
    cost: &str,
    targets: &[Entity],
) {
    activate(t, p, src, needle, cost, targets);
    t.resolve_all();
}

/// `p` casts the real card `name` from hand with the mana of `cost` in their pool and the
/// given targets (one per target slot); it stays on the stack.
pub fn cast(t: &mut TestGame, p: PlayerId, name: &str, cost: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    let c = t.hand(p, name);
    mana(t, p, cost);
    for x in targets {
        t.answer_targets(p, &[*x]);
    }
    let s = t
        .cast(p, c)
        .try_go()
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"));
    t.settle();
    s
}

/// As [`cast`], then everything resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, cost: &str, targets: &[Entity]) {
    cast(t, p, name, cost, targets);
    t.resolve_all();
}

/// Giant Growth (+3/+3 until end of turn) on the creature, cast by its controller.
pub fn giant_growth(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let p = t.obj_now(id).controller;
    cast_resolve(t, p, "Giant Growth", "G", &[obj(id)]);
}

/// Twisted Image ("Switch target creature's power and toughness until end of turn. Draw a
/// card.") on the creature, cast by its controller.
pub fn switch_pt(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let p = t.obj_now(id).controller;
    cast_resolve(t, p, "Twisted Image", "U", &[obj(id)]);
}

/// Puts `n` counters of `kind` on the permanent as an effect would, and settles.
pub fn put(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(obj(id), kind, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Destroys the permanent and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

/// Deals `n` noncombat damage to the permanent from `source`, and settles.
pub fn damage(t: &mut TestGame, source: ObjectId, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.deal_damage(source, obj(id), n, false);
    t.g.flush_events();
    t.settle();
}

/// Puts a real card onto the battlefield (no ETB) and settles.
pub fn add(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let id = t.battlefield(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// A real card enters the battlefield through a zone change and its triggers go on the
/// stack.
pub fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

pub fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

pub fn is_type(t: &TestGame, id: ObjectId, ty: CardType) -> bool {
    t.obj_now(id).is(ty)
}

pub fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

/// The number of activated abilities of the object whose text contains `s`.
pub fn activated_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(s))
        .count()
}

/// The damage marked on the permanent now (0 if it left the battlefield).
pub fn dmg(t: &TestGame, id: ObjectId) -> u32 {
    if t.on_battlefield(id) {
        t.obj_now(id).damage
    } else {
        0
    }
}

/// Whether `p` may activate the ability of `src` whose text contains `needle` now (with
/// the mana of `cost`): tries it, undoing nothing on failure.
pub fn can_activate(t: &mut TestGame, p: PlayerId, src: ObjectId, needle: &str, cost: &str) -> bool {
    mana(t, p, cost);
    crate::r_s06_common::activate_containing(t, p, src, needle).is_ok()
}

/// Queues `p`'s division of damage among the targets.
pub fn divide(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}
