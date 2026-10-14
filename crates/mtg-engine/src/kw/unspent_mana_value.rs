//! `Value::Custom` [`UNSPENT_MANA_YOU_HAVE`]: "the amount of unspent mana you have" (Glissa
//! Sunseeker) — the mana in the ability's controller's mana pool as it's read (CR 106.4).
//! Nothing lets a player activate mana abilities while an ability resolves (CR 608.2), so
//! only mana already in the pool counts.

use crate::eval::Ctx;
use crate::game::Game;

/// The `Value::Custom` name for "the amount of unspent mana you have".
pub const UNSPENT_MANA_YOU_HAVE: &str = "unspent mana you have";

pub struct UnspentManaValue;

impl super::KeywordRules for UnspentManaValue {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == UNSPENT_MANA_YOU_HAVE)
            .then(|| g.player(ctx.controller).mana_pool.total() as i64)
    }
}

inventory::submit! { super::KeywordRegistration(&UnspentManaValue) }
