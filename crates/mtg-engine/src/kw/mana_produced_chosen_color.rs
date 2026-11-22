//! `Condition::Custom` [`PRODUCED_CHOSEN_COLOR`]: "Whenever a basic land is tapped for mana
//! of the chosen color" (Gauntlet of Power): the mana the triggering mana ability produced
//! (CR 106.12a) includes mana of the color chosen for the trigger's source (CR 607.2d). No
//! color chosen, no match (CR 607.5a).

use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::ManaType;

/// The `Condition::Custom` name.
pub const PRODUCED_CHOSEN_COLOR: &str = "tapped for mana of the chosen color";

pub struct ProducedChosenColor;

impl super::KeywordRules for ProducedChosenColor {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != PRODUCED_CHOSEN_COLOR {
            return None;
        }
        let Some(c) = ctx
            .source
            .and_then(|s| g.try_obj(s))
            .and_then(|o| o.choices.color)
        else {
            return Some(false);
        };
        let t = ManaType::from_color(c);
        Some(ctx.event.as_ref().is_some_and(|e| e.mana.contains(&t)))
    }
}

inventory::submit! { super::KeywordRegistration(&ProducedChosenColor) }
