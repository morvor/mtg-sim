//! Shared helpers for the tests of rulings batch S06 (`r_s06_*.rs`): enchant (Auras and
//! Curses), encore, endure, enlist, enrage, equip. (The helpers of batches S01–S05 are
//! used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the real Aura (or Equipment) `name` onto the battlefield under `p`'s control
/// attached to `to` (no enters abilities trigger), and returns it.
pub fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: impl Into<Entity>) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(t.g.attach(id, to.into()), "{name} couldn't be attached");
    t.g.recompute();
    id
}

/// What the object (followed across zone changes) is attached to now.
pub fn attached_to(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.obj_now(id).attached_to
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

/// Gives `p` control of `id` (as a resolving "gain control of" effect with no duration
/// would), and settles state-based actions.
pub fn give_control(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    use mtg_engine::ability::{Duration, Effect, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: mtg_engine::ability::PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Activates the first activated ability of `source` whose text contains `needle`.
pub fn activate_containing(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    use mtg_engine::ability::AbilityKind;
    t.g.recompute();
    let s = t.g.current(source);
    let uid = t
        .g
        .obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such activated ability");
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, s, uid);
    t.g.flush_events();
    r
}

/// Marks `n` damage on the permanent from `source` (as a resolving effect would), then
/// settles state-based actions and triggers.
pub fn damage(t: &mut TestGame, source: ObjectId, n: i32, to: ObjectId) {
    use mtg_engine::ability::{Effect, Sel, Value};
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), t.obj_now(source).controller);
    ctx.targets = vec![vec![Entity::Object(t.g.current(to))]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}
