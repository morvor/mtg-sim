//! Card descriptions the dig grammar (`oracle/patterns/dig_grammar.rs`) uses that the
//! shared filter vocabulary doesn't have: "a land or double-faced card from among them"
//! (a double-faced card, CR 712.1), "a card with doctor's companion" (CR 702.124m).

use super::{KeywordRegistration, KeywordRules};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// "double-faced card": a card (or token) with two faces (CR 712.1).
pub const DOUBLE_FACED: &str = "dig:double-faced card";

/// "a card with doctor's companion": a card with that ability (CR 702.124m).
pub const DOCTORS_COMPANION: &str = "dig:card with doctor's companion";

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
        match name {
            DOUBLE_FACED => Some(
                g.obj(id)
                    .card
                    .as_ref()
                    .is_some_and(|c| c.layout.is_double_faced()),
            ),
            DOCTORS_COMPANION => Some(
                super::partner::partner_abilities(&g.obj(id).chars)
                    .contains(&super::partner::PartnerAbility::DoctorsCompanion),
            ),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&DigFilters) }
