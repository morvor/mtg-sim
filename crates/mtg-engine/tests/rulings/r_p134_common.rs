//! Shared helpers for the tests of rulings batch P134 (`r_p134_*.rs`): fights, bounce,
//! and artifact/enchantment/creature removal.

#![allow(dead_code)]

use crate::r_p113_common::{spoil, Spoil};
use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Who is made an illegal target before a two-target fight resolves.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Bad {
    None,
    Mine(Spoil),
    Theirs(Spoil),
    Both,
}

/// Puts a fight spell or ability on the stack with (mine, theirs) as its targets.
pub type Start = fn(&mut TestGame, ObjectId, ObjectId);

/// P0's `mine` and P1's `theirs` (real cards); `start` puts the fight on the stack, `bad`
/// happens in response, then everything resolves. Returns (game, mine, theirs).
pub fn fight_scenario(
    mine: &str,
    theirs: &str,
    start: impl Fn(&mut TestGame, ObjectId, ObjectId),
    bad: Bad,
) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, mine);
    let o = t.battlefield(P1, theirs);
    start(&mut t, m, o);
    match bad {
        Bad::None => {}
        Bad::Mine(s) => spoil(&mut t, m, s),
        Bad::Theirs(s) => spoil(&mut t, o, s),
        Bad::Both => {
            spoil(&mut t, m, Spoil::Hexproof);
            spoil(&mut t, o, Spoil::Hexproof);
        }
    }
    t.resolve_all();
    (t, m, o)
}

/// Queues `p`'s choice of modes for the next modal choice (a triggered ability's).
pub fn answer_modes(t: &mut TestGame, p: PlayerId, modes: &[usize]) {
    t.answer(p, DecisionKind::Modes, Answer::Indices(modes.to_vec()));
}

/// The object as an entity.
pub fn ob(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Asserts the real card compiled completely.
pub fn sup(name: &str) {
    supported(name)
}

/// Applies `mods` to `id` until end of turn, as an effect controlled by `p`.
pub fn modify(
    t: &mut TestGame,
    p: PlayerId,
    id: ObjectId,
    mods: Vec<mtg_engine::ability::Modification>,
) {
    use mtg_engine::ability::*;
    crate::r_s05_common::run_from(
        t,
        p,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(t.g.current(id))],
    );
}

/// `id` gains indestructible until end of turn.
pub fn indestructible(t: &mut TestGame, id: ObjectId) {
    use mtg_engine::ability::Modification;
    use mtg_engine::keywords::{Keyword, KeywordKind};
    let c = t.obj_now(id).controller;
    modify(
        t,
        c,
        id,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Indestructible,
        ))],
    );
}

/// A regeneration shield for `id` (CR 701.19a).
pub fn regen_shield(t: &mut TestGame, id: ObjectId) {
    use mtg_engine::ability::*;
    let c = t.obj_now(id).controller;
    crate::r_s05_common::run_from(
        t,
        c,
        None,
        Effect::Regenerate {
            what: Sel::Target(0),
        },
        &[Entity::Object(t.g.current(id))],
    );
}

/// `id` becomes a 2/2 creature in addition to its other types until end of turn.
pub fn animate(t: &mut TestGame, id: ObjectId) {
    use mtg_engine::ability::*;
    use mtg_engine::types::CardType;
    let c = t.obj_now(id).controller;
    modify(
        t,
        c,
        id,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::Const(2)), Some(Value::Const(2))),
        ],
    );
}
