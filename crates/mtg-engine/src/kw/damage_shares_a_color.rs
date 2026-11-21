//! `Condition::Custom` [`SOURCE_AND_RECIPIENT_SHARE_A_COLOR`]: "Prevent all damage that
//! would be dealt to a creature by another creature if they share a color." (Well-Laid
//! Plans): the damage event's source and the object it would be dealt to are different
//! objects with at least one color in common (CR 105.2; colorless isn't a color, CR
//! 105.2c).
//!
//! `Filter::Custom` [`SHARES_COLOR_WITH_PRODUCED_MANA`]: "If that creature shares a color
//! with the mana that land produced" (C.A.M.P.): the object has a color of the mana the
//! triggering mana ability produced (CR 106.12a; colorless mana has no color).

use crate::eval::Ctx;
use crate::game::Game;
use crate::types::ColorSet;

/// The `Condition::Custom` name.
pub const SOURCE_AND_RECIPIENT_SHARE_A_COLOR: &str = "damage source and recipient share a color";

/// The `Filter::Custom` name.
pub const SHARES_COLOR_WITH_PRODUCED_MANA: &str = "shares a color with the mana produced";

pub struct DamageSharesAColor;

impl super::KeywordRules for DamageSharesAColor {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: crate::types::ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != SHARES_COLOR_WITH_PRODUCED_MANA {
            return None;
        }
        let colors = g.obj(id).chars.colors;
        Some(ctx.event.as_ref().is_some_and(|e| {
            e.mana
                .iter()
                .filter_map(|t| t.color())
                .any(|c| colors.contains(c))
        }))
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != SOURCE_AND_RECIPIENT_SHARE_A_COLOR {
            return None;
        }
        let e = ctx.event.as_ref()?;
        let (Some(src), Some(to)) = (e.other, e.object) else {
            return Some(false);
        };
        if src == to {
            return Some(false);
        }
        let colors = |id| g.try_obj(id).map_or(ColorSet::NONE, |o| o.chars.colors);
        Some(colors(src).intersects(colors(to)))
    }
}

inventory::submit! { super::KeywordRegistration(&DamageSharesAColor) }
