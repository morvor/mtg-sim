//! "if it isn't being declared as an attacker" (Rhoda, Geist Avenger, Verity Circle):
//! the trigger event's permanent didn't become tapped by being declared as an attacker
//! (CR 508.1f). See `oracle/patterns/trigger_grammar_causes.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::trigger_grammar_causes::NOT_TAPPED_ATTACKING;
use crate::types::ObjectId;

struct TapCauses;

impl KeywordRules for TapCauses {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
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
