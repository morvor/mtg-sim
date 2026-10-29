//! "Creatures dealt damage by [a particular creature] this turn" (CR 120: the turn's
//! damage history) in an effect created by a resolving ability, after the creature was
//! locked in (`prevention::lock_damage_history`): Runesword's "If a creature dealt damage
//! by the targeted creature would die this turn, exile that creature instead".

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom` prefix: dealt damage this turn by one of the listed objects (ids).
pub const DEALT_DAMAGE_BY: &str = "damage history:dealt damage this turn by:";

/// The `Filter::Custom` name for objects dealt damage this turn by one of `ids`.
pub fn dealt_damage_by(ids: &[ObjectId]) -> String {
    let v: Vec<String> = ids.iter().map(|o| o.0.to_string()).collect();
    format!("{DEALT_DAMAGE_BY}{}", v.join(","))
}

struct DamageHistoryBy;

impl KeywordRules for DamageHistoryBy {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let list = name.strip_prefix(DEALT_DAMAGE_BY)?;
        let sources: Vec<u32> = list.split(',').filter_map(|x| x.parse().ok()).collect();
        Some(
            g.history
                .damage_by_source
                .iter()
                .any(|(s, t)| *t == id && sources.contains(&s.0)),
        )
    }
}

inventory::submit! { KeywordRegistration(&DamageHistoryBy) }
