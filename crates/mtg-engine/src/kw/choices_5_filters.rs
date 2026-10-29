//! Object qualities for cards5-choices-players' choice grammar: "other spells and/or
//! abilities" (Boltbender): a spell or ability on the stack other than the one resolving.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom` name: not the resolving spell or ability itself.
pub const NOT_RESOLVING: &str = "choices5:not the resolving spell or ability";

pub struct Choices5Filters;

impl KeywordRules for Choices5Filters {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, _g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == NOT_RESOLVING).then(|| ctx.stack_obj != Some(id))
    }
}

inventory::submit! { KeywordRegistration(&Choices5Filters) }
