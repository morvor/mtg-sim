//! Rules behind the trigger modifiers of `oracle/patterns/trigger_modifiers_4.rs`:
//! "an emblem you own" as a source of triggered abilities (CR 114.4: abilities of
//! emblems function in the command zone).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::ObjKind;
use crate::types::ObjectId;

/// `Filter::Custom`: an emblem (CR 114.1).
pub const EMBLEM: &str = "emblem";

struct TriggerModifiers;

impl KeywordRules for TriggerModifiers {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == EMBLEM).then(|| g.obj(id).kind == ObjKind::Emblem)
    }
}

inventory::submit! { KeywordRegistration(&TriggerModifiers) }
