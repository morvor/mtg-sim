//! `Filter::Custom` [`CREATED_BY_SOURCE`]: an object one of the source's abilities created
//! (CR 607.1d; `Game::link_to_creator` records the creator), judged by the object as it is
//! or last existed: "When the token leaves the battlefield, sacrifice ~." (Dance of Many)
//! looks back at the token that left (CR 603.10a), which is no longer among the source's
//! live linked objects.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub const CREATED_BY_SOURCE: &str = "created by the source";

pub struct CreatedBySource;

impl KeywordRules for CreatedBySource {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != CREATED_BY_SOURCE {
            return None;
        }
        let src = ctx.source?;
        Some(g.obj(id).created_by.is_some_and(|(s, _)| s == src))
    }
}

inventory::submit! { KeywordRegistration(&CreatedBySource) }
