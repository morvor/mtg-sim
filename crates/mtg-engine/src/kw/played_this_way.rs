//! `Filter::Custom(PLAYED_THIS_WAY)`: a spell cast or a land played with the permission
//! "this way" refers to (CR 400.7h, 400.7i; see `permissions::PermissionUse`).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub struct PlayedThisWay;

impl KeywordRules for PlayedThisWay {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == crate::permissions::PLAYED_THIS_WAY)
            .then(|| crate::permissions::played_this_way(g, id, ctx))
    }
}

inventory::submit! { KeywordRegistration(&PlayedThisWay) }
