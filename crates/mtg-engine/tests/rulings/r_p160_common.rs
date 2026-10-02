//! Shared helpers for the tests of rulings batch P160 (`r_p160_*.rs`): "shade pump"
//! activated abilities (+X/+X effects whose X and affected set are locked in as they
//! resolve), "shakedown", and "shapechange" effects that set base power and toughness
//! (CR 613.4b) or copy objects. (The helpers of batches S01–S30 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_s01_common::supported;
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Casts the real card `name` (with lands for its cost) with the given targets and
/// resolves the stack.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    cast_new(t, p, name, targets);
    t.resolve_all();
}

/// Activates the `index`th activated ability of `source` and resolves the stack.
pub fn activate_resolve(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    index: usize,
    targets: &[Entity],
) {
    t.activate(p, source, index, targets)
        .unwrap_or_else(|e| panic!("activation failed: {e:?}"));
    t.resolve_all();
}

/// Puts `n` +1/+1 counters on the object.
pub fn plus_counters(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Queues a "yes" for `p`'s next yes/no decision.
pub fn yes(t: &mut TestGame, p: PlayerId) {
    t.answer(p, DecisionKind::YesNo, Answer::Bool(true));
}

/// The creature tokens `p` controls.
pub fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.id)
        .collect()
}

/// The base power and toughness that a setting spell used as an "earlier" or "later"
/// effect gives its target.
pub fn setter_pt(name: &str) -> (i32, i32) {
    match name {
        "Square Up" => (4, 4),
        "Relic's Roar" => (4, 3),
        "Mind Transfer Protocol" => (4, 5),
        "Suit Up" => (4, 5),
        "Majestic Metamorphosis" => (4, 4),
        "Humble" => (0, 1),
        _ => panic!("unknown setter {name}"),
    }
}

/// The standard layer-7 scenario on `target` (P0's creature): it has a +1/+1 counter, an
/// earlier base-setting effect (`earlier`), a +3/+3 effect (Giant Growth) and, if
/// `switch`, a switch effect (Twisted Image) — all before the tested effect. `apply` then
/// applies the tested effect, which should set its base power and toughness to `set`: the
/// counter, the pump and the switch still apply on top. Finally the setting spell `later`
/// overwrites the tested effect.
pub fn layer7(
    t: &mut TestGame,
    target: ObjectId,
    earlier: &str,
    switch: bool,
    apply: impl FnOnce(&mut TestGame),
    set: (i32, i32),
    later: &str,
) {
    layer7_by(t, P0, target, earlier, switch, apply, set, later)
}

/// [`layer7`], with the setup spells (the earlier and later setters, the pump and the
/// switch) cast by `caster` (an opponent when the tested ability triggers on its
/// controller's spells).
#[allow(clippy::too_many_arguments)]
pub fn layer7_by(
    t: &mut TestGame,
    caster: PlayerId,
    target: ObjectId,
    earlier: &str,
    switch: bool,
    apply: impl FnOnce(&mut TestGame),
    set: (i32, i32),
    later: &str,
) {
    let target = t.g.current(target);
    plus_counters(t, target, 1);
    cast_resolve(t, caster, earlier, &[Entity::Object(target)]);
    cast_resolve(t, caster, "Giant Growth", &[Entity::Object(target)]);
    if switch {
        cast_resolve(t, caster, "Twisted Image", &[Entity::Object(target)]);
    }
    let sw = |(p, q): (i32, i32)| if switch { (q, p) } else { (p, q) };
    let e = setter_pt(earlier);
    assert_eq!(t.pt(target), sw((e.0 + 4, e.1 + 4)), "after {earlier}");
    apply(t);
    t.g.recompute();
    assert_eq!(
        t.pt(target),
        sw((set.0 + 4, set.1 + 4)),
        "after the tested effect"
    );
    cast_resolve(t, caster, later, &[Entity::Object(target)]);
    let l = setter_pt(later);
    assert_eq!(t.pt(target), sw((l.0 + 4, l.1 + 4)), "after {later}");
}

pub fn targeted_spell(name: &'static str, target: ObjectId) -> impl FnOnce(&mut TestGame) {
    move |t: &mut TestGame| cast_resolve(t, P0, name, &[Entity::Object(target)])
}

pub fn helpers_supported() {
    for name in [
        "Square Up",
        "Relic's Roar",
        "Mind Transfer Protocol",
        "Giant Growth",
        "Twisted Image",
    ] {
        supported(name);
    }
}
