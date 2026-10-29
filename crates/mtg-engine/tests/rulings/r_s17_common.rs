//! Shared helpers for the tests of rulings batch S17 (`r_s17_*.rs`): trample, transform
//! (double-faced cards, battles, werewolves, disturb), tribute, triple, umbra armor,
//! undaunted, undergrowth. (The helpers of batches S01–S16 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Transforms the permanent (followed across zone changes) as a resolving effect would
/// ("Transform target permanent"), then settles.
pub fn transform(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let mut ctx = mtg_engine::eval::Ctx::new(None, t.obj(id).controller);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Transform {
            what: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Puts the card `id` onto the battlefield under `p`'s control as a resolving effect of
/// `p`'s would (transformed if `transformed`), then settles. Returns the new object if it
/// entered.
pub fn put_onto_battlefield(
    t: &mut TestGame,
    p: PlayerId,
    id: ObjectId,
    transformed: bool,
) -> Option<ObjectId> {
    let id = t.g.current(id);
    let mut to = Destination::battlefield().under_your_control();
    to.transformed = transformed;
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    let now = t.g.current(id);
    (now != id && t.zone(now) == Zone::Battlefield).then_some(now)
}

/// The real double-faced card `name` put onto the battlefield under `p`'s control with
/// its back face up (from `p`'s graveyard, by an effect), after settling.
pub fn enter_transformed(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.graveyard(p, name);
    put_onto_battlefield(t, p, card, true).expect("it didn't enter transformed")
}

/// Which face of the object (followed across zone changes) is up.
pub fn face(t: &TestGame, id: ObjectId) -> FaceState {
    t.obj_now(id).face
}

/// The object's (followed across zone changes) name now.
pub fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// A set of colors.
pub fn color_set(cs: &[mtg_engine::types::Color]) -> mtg_engine::types::ColorSet {
    let mut s = mtg_engine::types::ColorSet::NONE;
    for c in cs {
        s.insert(*c);
    }
    s
}

/// `id` becomes a copy of the permanent `of` (as a resolving effect of its own would,
/// permanently), then settles.
pub fn become_copy(t: &mut TestGame, id: ObjectId, of: ObjectId) {
    let id = t.g.current(id);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(id), t.obj(id).controller);
    ctx.targets = vec![vec![Entity::Object(t.g.current(of))]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::This,
            of: Sel::Target(0),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Creates a token that's a copy of `of` (any zone) under `p`'s control, then settles.
/// Returns the tokens `p` controls that were created.
pub fn token_copy(t: &mut TestGame, p: PlayerId, of: ObjectId) -> Vec<ObjectId> {
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(of))]];
    t.g.exec(
        &Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| !before.contains(o) && t.obj(*o).is_token())
        .collect()
}
