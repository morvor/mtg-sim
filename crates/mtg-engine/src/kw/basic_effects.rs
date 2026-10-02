//! Engine support for basic effect verbs with qualified objects (see
//! `oracle/patterns/basic_effects_*.rs`):
//!
//! * "Counter target spell that's the second spell cast this turn." (Second Guess): the
//!   second spell any player cast this turn, by cast order (copies aren't cast, CR 707.10).
//! * The permanent whose ability is in a target slot ("If a permanent's ability is
//!   countered this way, destroy that permanent."): the object that was the ability's
//!   source, if it's still that object (CR 400.7); a stack ability's source is the object
//!   it came from (CR 113.7, 113.7a).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;

/// `Filter::Custom`: the second spell cast this turn.
pub const SECOND_SPELL_CAST_THIS_TURN: &str = "basic_effects:second spell cast this turn";

const SOURCE_OF_SLOT: &str = "basic_effects:source of the ability in target slot ";

/// `Filter::Custom` name: the source of the ability chosen in target slot `slot`.
pub fn source_of_slot(slot: u8) -> String {
    format!("{SOURCE_OF_SLOT}{slot}")
}

pub struct BasicEffects;

impl KeywordRules for BasicEffects {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == SECOND_SPELL_CAST_THIS_TURN {
            return Some(g.history.spells_cast.get(1).is_some_and(|(_, s)| *s == id));
        }
        if let Some(slot) = name.strip_prefix(SOURCE_OF_SLOT) {
            let slot: usize = slot.parse().ok()?;
            let targets = ctx.targets.get(slot)?;
            return Some(targets.iter().any(|t| {
                let Entity::Object(a) = t else {
                    return false;
                };
                matches!(
                    g.obj(*a).stack.as_deref().map(|si| &si.kind),
                    Some(StackKind::Activated { source, .. } | StackKind::Triggered { source, .. })
                        if *source == id
                )
            }));
        }
        None
    }
}

inventory::submit! { KeywordRegistration(&BasicEffects) }
