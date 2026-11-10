//! Shared helpers for the tests of rulings batch P155 (`r_p155_*.rs`): abilities that
//! scale with a mana value or a power: the value as it last existed on the battlefield,
//! in a graveyard or in a hand (CR 608.2h), the mana values of double-faced, split,
//! Adventure and prototype cards, tokens and copies (CR 202.3), reflexive triggers,
//! values locked in on resolution and spells cast during resolution. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p154_common::*;
pub use crate::r_p016_common::{cast_at, lots, stack_targets, target_choices};
pub use crate::r_p153_common::{answer_x, cast_blaze, cast_simple, mana_for, mv, trigger_sources, x_asked};
pub use crate::r_p195_common::{on_stack, top_is_trigger_of, top_name};
pub use crate::r_s04_common::{graveyard_names, hand_names, stack_items, top_of_stack};
pub use crate::r_s05_common::move_to;

/// `p`'s copy card `copier` (Mirror Image, Clone, ...) enters the battlefield as a copy
/// of `of` (through a real zone change, choosing to copy): the copy's id.
pub fn enter_copy_of(t: &mut TestGame, p: PlayerId, copier: &str, of: ObjectId) -> ObjectId {
    supported(copier);
    let of = t.g.current(of);
    t.answer_yes(p, true);
    t.answer_choose(p, &[obj(of)]);
    let c = t.enter(p, copier);
    t.settle();
    assert_eq!(
        t.obj_now(c).chars.name,
        t.obj_now(of).chars.name,
        "{copier} didn't copy: {}",
        t.dump_log()
    );
    c
}

/// A Mirror Image ({2}{U}, mana value 3) of `p`'s copying `of` (a creature `p` controls):
/// a permanent whose mana value on the battlefield (the copied creature's) differs from
/// the mana value of the card it is anywhere else (3).
pub fn mirror_image_of(t: &mut TestGame, p: PlayerId, of: ObjectId) -> ObjectId {
    enter_copy_of(t, p, "Mirror Image", of)
}

/// A Sculpting Steel ({3}, mana value 3) of `p`'s copying the artifact `of`.
pub fn sculpting_steel_of(t: &mut TestGame, p: PlayerId, of: ObjectId) -> ObjectId {
    enter_copy_of(t, p, "Sculpting Steel", of)
}

/// The names of the targets a spell or ability on the stack has.
pub fn target_names(t: &TestGame, id: ObjectId) -> Vec<String> {
    stack_targets(t, id)
        .iter()
        .filter_map(|e| match e {
            Entity::Object(o) => Some(t.g.obj(*o).chars.name.to_string()),
            Entity::Player(p) => Some(format!("P{}", p.0)),
        })
        .collect()
}

/// The yes/no questions asked of `p` since decision `from`.
pub fn yes_nos_asked(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

/// Puts the card into its owner's graveyard from wherever it is (as an effect would)
/// and settles; the new object.
pub fn to_graveyard(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    move_to(t, id, Zone::Graveyard(owner)).expect("moved")
}

/// The number of creature tokens `p` controls.
pub fn creature_tokens(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s25_common::creature_tokens(t, p)
}

pub use crate::r_s17_common::token_copy;

/// Transforms the double-faced permanent (as an effect would) and settles.
pub fn transform(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    assert!(mtg_engine::dfc::transform(&mut t.g, id), "didn't transform");
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// `p`'s Thraben Sentry ({3}{W}, 4) transformed into Thraben Militia (a 5/4 with trample
/// and no mana cost): a back-face-up nonmodal double-faced permanent.
pub fn militia(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Thraben Sentry // Thraben Militia");
    let h = t.battlefield(p, "Thraben Sentry // Thraben Militia");
    transform(t, h);
    assert_eq!(t.obj_now(h).chars.name, "Thraben Militia");
    assert!(t.obj_now(h).chars.mana_cost.is_none());
    h
}

/// Bruna, the Fading Light ({5}{W}{W}) and Gisela, the Broken Blade ({2}{W}{W}) melded
/// into Brisela, Voice of Nightmares under `p`'s control: the melded permanent.
pub fn brisela(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let bruna = t.exile(p, "Bruna, the Fading Light");
    let gisela = t.exile(p, "Gisela, the Broken Blade");
    let b = mtg_engine::merge::meld(
        &mut t.g,
        bruna,
        gisela,
        mtg_engine::card::card("Brisela, Voice of Nightmares"),
        p,
    )
    .expect("melded");
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.obj_now(b).chars.name, "Brisela, Voice of Nightmares");
    b
}

pub use crate::r_p003_common::activate;
pub use crate::r_s02_common::can_play_land;
pub use crate::r_s01_common::with_subtype;
