//! Choices IV (`oracle/patterns/choices_4.rs`): filters reading a choice made by the
//! instruction's own ability.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom`: an object whose power is of the quality (odd or even) chosen by the
/// linked ability (CR 208, 607.2d). An object with no power has none of either quality;
/// an undefined choice matches nothing (CR 607.5a).
pub const POWER_OF_CHOSEN_QUALITY: &str = "choices 4: power of the chosen quality";

pub struct Choices4;

impl KeywordRules for Choices4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != POWER_OF_CHOSEN_QUALITY {
            return None;
        }
        let Some(p) = g.obj(id).chars.power else {
            return Some(false);
        };
        let q = g.linked_choice(ctx).and_then(|ch| ch.text.clone());
        Some(match q.as_deref() {
            Some("odd") => p.rem_euclid(2) == 1,
            Some("even") => p.rem_euclid(2) == 0,
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&Choices4) }
