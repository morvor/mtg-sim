//! `Value::Custom` [`UNSPENT_MANA_YOU_HAVE`]: "the amount of unspent mana you have" (Glissa
//! Sunseeker) — the mana in the ability's controller's mana pool as it's read (CR 106.4).
//! Nothing lets a player activate mana abilities while an ability resolves (CR 608.2), so
//! only mana already in the pool counts.
//!
//! A static ability may count it too ("~ gets +1/+1 for each unspent green mana you have",
//! "as long as you have six or more unspent mana"): the pool changing marks the
//! characteristics for recomputation (`Game::add_mana`, spending and emptying mana).

use crate::eval::Ctx;
use crate::game::Game;

/// The `Value::Custom` name for "the amount of unspent mana you have".
pub const UNSPENT_MANA_YOU_HAVE: &str = "unspent mana you have";

/// `Value::Custom` prefix, followed by a mana type letter (`unspent mana you have:G`):
/// "for each unspent green mana you have" (Omnath, Locus of Mana).
pub const UNSPENT_MANA_OF_TYPE: &str = "unspent mana you have:";

/// The `Value::Custom` name for the unspent mana of one type.
pub fn of_type(t: crate::mana::ManaType) -> String {
    format!("{UNSPENT_MANA_OF_TYPE}{t:?}")
}

pub struct UnspentManaValue;

impl super::KeywordRules for UnspentManaValue {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if let Some(t) = name.strip_prefix(UNSPENT_MANA_OF_TYPE) {
            let t = crate::mana::ManaType::from_letter_str(t)?;
            return Some(g.player(ctx.controller).mana_pool.count(t) as i64);
        }
        (name == UNSPENT_MANA_YOU_HAVE)
            .then(|| g.player(ctx.controller).mana_pool.total() as i64)
    }
}

inventory::submit! { super::KeywordRegistration(&UnspentManaValue) }
