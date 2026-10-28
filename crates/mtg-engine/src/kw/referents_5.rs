//! Engine rules behind `oracle/patterns/referents_values_5.rs` that need code: named
//! filters over what an earlier step did.
//!
//! * [`crate::kwa::manifest::PUT_IN_GRAVEYARD_BY_THAT_DREAD`]: a card the triggering
//!   manifest dread put into a graveyard (CR 701.62a), still there (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub struct Referents5;

impl KeywordRules for Referents5 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == crate::kwa::manifest::PUT_IN_GRAVEYARD_BY_THAT_DREAD {
            return Some(
                g.is_live(id)
                    && crate::kwa::manifest::dread_graveyard_cards(g, ctx).contains(&id),
            );
        }
        None
    }
}

inventory::submit! { KeywordRegistration(&Referents5) }
