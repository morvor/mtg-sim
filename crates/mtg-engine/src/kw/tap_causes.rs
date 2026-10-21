//! Conditions on what caused a trigger event (`oracle/patterns/trigger_grammar_causes.rs`):
//! "if it isn't being declared as an attacker" (Rhoda, Geist Avenger, Verity Circle: the
//! permanent didn't become tapped by being declared as an attacker, CR 508.1f), and "a
//! spell or ability an opponent controls causes ..." (the event's cause).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::trigger_grammar_causes::{CAUSED_BY_OPPONENT, NOT_TAPPED_ATTACKING};
use crate::types::ObjectId;

struct TapCauses;

impl KeywordRules for TapCauses {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == CAUSED_BY_OPPONENT).then(|| {
            ctx.event
                .as_ref()
                .and_then(|e| e.caused_by)
                .is_some_and(|q| g.are_opponents(ctx.controller, q))
        })
    }

    fn custom_filter(&self, _g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == NOT_TAPPED_ATTACKING).then(|| {
            ctx.event
                .as_ref()
                .is_some_and(|e| e.object == Some(id) && !e.attack_declaration)
        })
    }
}

inventory::submit! { KeywordRegistration(&TapCauses) }
