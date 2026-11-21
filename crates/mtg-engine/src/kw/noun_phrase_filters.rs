//! Object qualities read by the noun-phrase predicates (`oracle/patterns/
//! noun_phrase_predicates.rs`) that no other filter expresses: "that's attacking a
//! planeswalker you control" (a creature attacking a planeswalker, not a player, CR
//! 506.2, 508.1b).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::noun_phrase_predicates::ATTACKING_YOUR_PLANESWALKER;
use crate::types::*;

pub struct NounPhraseFilters;

impl KeywordRules for NounPhraseFilters {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != ATTACKING_YOUR_PLANESWALKER {
            return None;
        }
        Some(
            g.combat
                .as_ref()
                .and_then(|c| c.attack_target(id))
                .is_some_and(|t| match t {
                    Entity::Player(_) => false,
                    Entity::Object(o) => {
                        g.obj(o).is(CardType::Planeswalker) && g.obj(o).controller == ctx.controller
                    }
                }),
        )
    }
}

inventory::submit! { KeywordRegistration(&NounPhraseFilters) }
