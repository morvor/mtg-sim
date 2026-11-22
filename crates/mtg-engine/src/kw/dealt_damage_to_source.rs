//! `Filter::Custom` "dealt damage to the source this turn": "each creature that dealt
//! damage to it this turn" in the source's own ability (Brine Hag: "When ~ dies, the base
//! power and toughness of each creature that dealt damage to it this turn become 0/2.").

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub const DEALT_DAMAGE_TO_SOURCE: &str = "dealt damage to the source this turn";

pub struct DealtDamageToSource;

impl KeywordRules for DealtDamageToSource {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != DEALT_DAMAGE_TO_SOURCE {
            return None;
        }
        let Some(src) = ctx.source else {
            return Some(false);
        };
        // The source as the object that was dealt the damage: the permanent it was before
        // it left the battlefield, if it just did ("When ~ dies, ... dealt damage to it").
        let before = g.obj(src).prev;
        Some(
            g.history.damage_by_source.contains(&(id, src))
                || before.is_some_and(|b| g.history.damage_by_source.contains(&(id, b))),
        )
    }
}

inventory::submit! { KeywordRegistration(&DealtDamageToSource) }
