//! Shared helpers for rulings batch P025 (`r_p025_*.rs`): effects that copy creatures —
//! what a copy copies (CR 707.2), the exceptions it makes and their copiability (CR
//! 707.9), copy effects in the layer system (CR 613.1a, 613.2) and the timing of the
//! abilities that make copies.

#![allow(dead_code)]

pub use crate::r_p021_common::{
    abilities_with, animate_land, assert_printed, assert_printed_giant, dress, dressed_giant,
    giant_growth, has_subtype, legendary, modify, obj, red, untap,
};
pub use crate::r_p023_common::{
    clone_of, cloned_angel, enter_copying, mv_now, token_of, unsick, wolf_token,
};
pub use crate::r_s01_common::supported;
pub use crate::r_s06_common::{activate_containing, has_kw};
pub use crate::r_s18_common::lands_for;
pub use crate::r_s26_common::{dress_up, fresh, new_tokens};
use mtg_engine::ability::*;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts `p`'s `spell` (paying `cost` with basic lands) with one target per entry of
/// `targets`, resolves everything, and returns the tokens `p` got.
pub fn cast_for_tokens(
    t: &mut TestGame,
    p: PlayerId,
    spell: &str,
    cost: &str,
    targets: &[Entity],
) -> Vec<ObjectId> {
    supported(spell);
    lands_for(t, p, cost);
    let card = t.hand(p, spell);
    let before = t.g.battlefield.clone();
    let mut b = t.cast(p, card);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    t.resolve_all();
    new_tokens(t, p, &before)
}

/// The triggered abilities on the stack, bottom first.
pub fn triggers_on_stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|s| {
            matches!(
                t.g.obj(*s).stack.as_ref().map(|i| &i.kind),
                Some(StackKind::Triggered { .. })
            )
        })
        .collect()
}

/// A plain creature token spec: `name` (also its one creature type), `color`, `n`/`n`.
pub fn creature_spec(name: &str, color: Color, n: i32) -> TokenSpec {
    TokenSpec {
        name: name.into(),
        colors: ColorSet::single(color),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![name.into()],
        power: Some(n),
        toughness: Some(n),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    }
}

/// Puts `n` +1/+1 counters on the permanent (as a resolving effect would).
pub fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, n, None);
    t.g.recompute();
}

/// Whether the object (followed across zone changes) is a creature now.
pub fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// The name of the object (followed across zone changes) now.
pub fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}
