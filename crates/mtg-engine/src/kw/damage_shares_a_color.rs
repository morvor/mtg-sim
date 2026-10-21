//! `Condition::Custom` [`SOURCE_AND_RECIPIENT_SHARE_A_COLOR`]: "Prevent all damage that
//! would be dealt to a creature by another creature if they share a color." (Well-Laid
//! Plans): the damage event's source and the object it would be dealt to are different
//! objects with at least one color in common (CR 105.2; colorless isn't a color, CR
//! 105.2c).

use crate::eval::Ctx;
use crate::game::Game;
use crate::types::ColorSet;

/// The `Condition::Custom` name.
pub const SOURCE_AND_RECIPIENT_SHARE_A_COLOR: &str = "damage source and recipient share a color";

pub struct DamageSharesAColor;

impl super::KeywordRules for DamageSharesAColor {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
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
