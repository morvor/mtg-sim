//! Shared helpers for the tests of rulings batch S10 (`r_s10_*.rs`): the Gods of Theros
//! (indestructible), infect, inspired, intimidate, investigate, islandwalk, jump-start,
//! kicker, kinship, landfall, learn, level up, lieutenant, madness. (The helpers of
//! batches S01–S07 are used too.)

#![allow(dead_code)]

use mtg_engine::card::CardDef;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Puts a vanilla enchantment with mana cost `cost` onto the battlefield under `p`'s
/// control (it adds the mana symbols of `cost` to `p`'s devotion, CR 700.5).
pub fn idol(t: &mut TestGame, p: PlayerId, cost: &str) -> ObjectId {
    let def = CardDef::custom(Characteristics {
        name: SmolStr::new("Devotion Idol"),
        mana_cost: mtg_engine::mana::ManaCost::parse(cost),
        card_types: CardTypeSet::single(CardType::Enchantment),
        rules_text: Arc::from(""),
        ..Default::default()
    });
    t.custom(p, def, Zone::Battlefield)
}

/// Whether the object (followed across zone changes) is attacking now.
pub fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.is_attacking(id)
}

/// Whether the object (followed across zone changes) is blocking now.
pub fn blocking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.is_blocking(id)
}

/// `p`'s poison counters.
pub fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}
