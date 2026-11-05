//! Shared helpers for rulings batch P024 (`r_p024_*.rs`): effects that copy creatures —
//! copying a token or something that's copying something else (CR 707.2, 707.3, 111.3),
//! {X} in a copied mana cost (CR 107.3, 202.3), copy-making abilities whose targets
//! became illegal (CR 608.2b), last known information (CR 608.2h) and the "enters"
//! abilities of token copies (CR 707.5, 603.6a, 614.1c).

#![allow(dead_code)]

pub use crate::r_p023_common::{
    assert_angel, assert_wolf, cloned_angel, clone_of, dressed_wolf, enter_copying, is_wolf,
    mv_now, token_of, unsick, wolf_token,
};
pub use crate::r_p025_common::{creature_spec, is_creature, plus1};
pub use crate::r_p027_common::{abilities_with, has_subtype, kill, name_of, obj};
pub use crate::r_s01_common::supported;
pub use crate::r_s06_common::{activate_containing, has_kw};
pub use crate::r_s18_common::lands_for;
pub use crate::r_s26_common::new_tokens;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts `p`'s `spell` (paying `cost` with basic lands, Wastes for generic) with one
/// target per entry of `targets`, optional `modes` and `x`, resolves everything, and
/// returns the tokens `p` got.
pub fn cast_spell(
    t: &mut TestGame,
    p: PlayerId,
    spell: &str,
    cost: &str,
    targets: &[Entity],
    modes: Option<&[usize]>,
    x: Option<i64>,
) -> Vec<ObjectId> {
    supported(spell);
    lands_for(t, p, cost);
    let card = t.hand(p, spell);
    let before = t.g.battlefield.clone();
    let mut b = t.cast(p, card);
    for e in targets {
        b = b.target(*e);
    }
    if let Some(m) = modes {
        b = b.modes(m);
    }
    if let Some(x) = x {
        b = b.x(x);
    }
    b.go();
    t.resolve_all();
    new_tokens(t, p, &before)
}

/// The creature tokens `p` controls that entered after the battlefield snapshot `before`.
pub fn tokens_since(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    new_tokens(t, p, before)
}

/// `p`'s creature token named and typed `name` (a `color` `n`/`n`), with its summoning
/// sickness cleared.
pub fn typed_token(t: &mut TestGame, p: PlayerId, name: &str, color: Color, n: i32) -> ObjectId {
    token_of(t, p, creature_spec(name, color, n))
}

/// Whether the object is an artifact now.
pub fn is_artifact(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Artifact)
}
