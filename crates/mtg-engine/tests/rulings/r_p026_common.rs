//! Shared helpers for rulings batch P026 (`r_p026_*.rs`): Clone-style and token-copy
//! effects — what a copy copies (CR 707.2), the exceptions it makes and whether later
//! copies copy them (CR 707.9), and the durations and delayed triggers of copy effects.

#![allow(dead_code)]

pub use crate::r_p021_common::{
    abilities_with, animate_land, assert_printed, assert_printed_giant, dress, dressed_giant,
    has_subtype, legendary, modify, obj, red, untap,
};
pub use crate::r_p023_common::{clone_of, cloned_angel, enter_copying, unsick};
pub use crate::r_p025_common::{is_creature, name_of, plus1};
pub use crate::r_s01_common::supported;
pub use crate::r_s06_common::{activate_containing, attach_new, give_control, has_kw};
pub use crate::r_s18_common::lands_for;
pub use crate::r_s26_common::{dress_up, fresh, new_tokens};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s `name` on the battlefield, dressed up in every way a copy doesn't copy (see
/// [`dress`]: tapped, equipped with Bonesplitter, two +1/+1 counters, +3/+3 and green-blue
/// until end of turn).
pub fn dressed(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    dress(t, p, id);
    id
}

/// Casts `p`'s `spell` (paying `cost` with basic lands; `extra` more Wastes for X or
/// additional costs) with `targets` in order, optional `modes` and `x`; resolves
/// everything and returns the tokens `p` got.
#[allow(clippy::too_many_arguments)]
pub fn cast_for_tokens(
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

/// `p` activates the ability of `source` whose text contains `needle` with `targets`
/// (answered in order), resolves everything and returns the tokens `p` got.
pub fn activate_for_tokens(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    targets: &[Entity],
) -> Vec<ObjectId> {
    let before = t.g.battlefield.clone();
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    activate_containing(t, p, source, needle).expect("activation failed");
    t.resolve_all();
    new_tokens(t, p, &before)
}

/// Casts Cytoshape for P0: `target` becomes a copy of `what` until end of turn.
pub fn cytoshape(t: &mut TestGame, what: ObjectId, target: ObjectId) {
    supported("Cytoshape");
    lands_for(t, P0, "{1}{G}{U}");
    let c = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[obj(what)]);
    t.cast(P0, c).target(obj(target)).go();
    t.resolve_all();
}

/// Makes `p`'s `id` a copy of P1's new Grizzly Bears until end of turn (Cytoshape): a
/// copy effect that changed it, which a copy of its card after it changes zones mustn't
/// see (CR 400.7).
pub fn disguise_as_bears(t: &mut TestGame, id: ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    cytoshape(t, bears, id);
    assert_eq!(name_of(t, id), "Grizzly Bears");
}

/// The only element of `v`.
pub fn one(v: Vec<ObjectId>) -> ObjectId {
    assert_eq!(v.len(), 1, "expected one object, got {v:?}");
    v[0]
}

/// The permanents named `name` on the battlefield that are tokens.
pub fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj(*id).is_token())
        .collect()
}

/// Kills the creature (destroys it, as Murder would) and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    crate::r_s02_common::destroy(t, id);
}

/// P0's Fable of the Mirror-Breaker, transformed into Reflection of Kiki-Jiki.
pub fn reflection(t: &mut TestGame) -> ObjectId {
    let fable = t.battlefield(P0, "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki");
    mtg_engine::dfc::transform(&mut t.g, fable);
    t.g.recompute();
    let r = t.g.current(fable);
    assert_eq!(name_of(t, r), "Reflection of Kiki-Jiki");
    unsick(t, r);
    r
}
