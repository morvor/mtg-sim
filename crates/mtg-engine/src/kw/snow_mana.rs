//! Snow mana spent to cast a spell (CR 107.4h): "{S}" also refers to mana of any type
//! produced by a snow source spent to pay a cost ("X is the amount of {S} spent to cast
//! this spell"). The amount is recorded in the spell's `CastInfo::mana_spent_snow` as its
//! total cost is paid; a copy of a spell, or a permanent that wasn't cast, sees zero
//! (CR 707.10).

use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Value::Custom` name: the amount of mana from snow sources spent to cast the spell
/// (or the permanent it became).
pub const SNOW_MANA_SPENT: &str = "snow mana spent to cast it";

/// Records that `n` mana produced by snow sources was spent to pay the total cost of the
/// spell `spell` (CR 601.2h).
pub fn record(g: &mut Game, spell: ObjectId, n: u32) {
    if n == 0 {
        return;
    }
    let o = &mut g.objects[spell.0 as usize];
    if o.zone != Zone::Stack {
        return;
    }
    if let Some(si) = o.stack.as_mut() {
        si.cast.mana_spent_snow += n;
    }
}

struct SnowManaSpent;

impl super::KeywordRules for SnowManaSpent {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == SNOW_MANA_SPENT).then(|| {
            g.cast_info(ctx)
                .filter(|c| c.was_cast)
                .map_or(0, |c| c.mana_spent_snow as i64)
        })
    }
}

inventory::submit! { super::KeywordRegistration(&SnowManaSpent) }
