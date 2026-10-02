//! Card descriptions the dig grammar (`oracle/patterns/dig_grammar.rs`) uses that the
//! shared filter vocabulary doesn't have: "a land or double-faced card from among them"
//! (a double-faced card, CR 712.1).

use super::{KeywordRegistration, KeywordRules};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// "double-faced card": a card (or token) with two faces (CR 712.1).
pub const DOUBLE_FACED: &str = "dig:double-faced card";

pub struct DigFilters;

impl KeywordRules for DigFilters {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(
        &self,
        g: &Game,
        name: &str,
        id: ObjectId,
        _ctx: &crate::eval::Ctx,
    ) -> Option<bool> {
        (name == DOUBLE_FACED).then(|| {
            g.obj(id)
                .card
                .as_ref()
                .is_some_and(|c| c.layout.is_double_faced())
        })
    }
}

inventory::submit! { KeywordRegistration(&DigFilters) }
