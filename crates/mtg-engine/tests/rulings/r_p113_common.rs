//! Shared helpers for the tests of rulings batch P113 (`r_p113_*.rs`): one-sided fights
//! ("target creature you control deals damage equal to its power to ..."), "becomes the
//! target" triggers, overrun-style pumps, opponents' choices, and partner commanders.

#![allow(dead_code)]

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::run_from;
use crate::r_s06_common::give_control;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::*;

/// How one target of a spell is made illegal before it resolves.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Spoil {
    /// The object is destroyed.
    Gone,
    /// The object changes control (to the other player of a two-player game).
    Stolen,
    /// The object gains hexproof (from its controller, if P1's) or shroud (if P0's).
    Hexproof,
}

/// Makes `id` an illegal target for P0's spells and abilities in the way `how` says.
pub fn spoil(t: &mut TestGame, id: ObjectId, how: Spoil) {
    match how {
        Spoil::Gone => destroy(t, id),
        Spoil::Stolen => {
            let c = t.obj_now(id).controller;
            give_control(t, id, if c == P0 { P1 } else { P0 })
        }
        Spoil::Hexproof => {
            let c = t.obj_now(id).controller;
            let kw = if c == P0 {
                KeywordKind::Shroud
            } else {
                KeywordKind::Hexproof
            };
            run_from(
                t,
                c,
                None,
                Effect::Modify {
                    what: Sel::Target(0),
                    mods: vec![Modification::AddKeyword(Keyword::new(kw))],
                    duration: Duration::EndOfTurn,
                },
                &[Entity::Object(t.g.current(id))],
            );
        }
    }
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost and casts it
/// with the given targets and modes. Returns the spell.
pub fn cast_modes(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    modes: &[usize],
    targets: &[Entity],
) -> ObjectId {
    supported(name);
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let mut b = t.cast(p, card);
    if !modes.is_empty() {
        b = b.modes(modes);
    }
    b.try_go()
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// The object as an entity.
pub fn ent(id: ObjectId) -> Entity {
    Entity::Object(id)
}
