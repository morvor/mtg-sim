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
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;

/// `Filter::Custom`: the second spell cast this turn.
pub const SECOND_SPELL_CAST_THIS_TURN: &str = "basic_effects:second spell cast this turn";

/// `Filter::Custom`: a source that dealt damage this turn.
pub const DEALT_DAMAGE_THIS_TURN: &str = "basic_effects:dealt damage this turn";
/// `Filter::Custom`: a creature that blocked this turn.
pub const BLOCKED_THIS_TURN: &str = "basic_effects:blocked this turn";
const BLOCKED_OR_WAS_BLOCKED_BY: &str = "basic_effects:blocked or was blocked by:";

/// `Filter::Custom` name: a creature that blocked, or was blocked by, a creature matching
/// `by` this turn.
pub fn blocked_or_was_blocked_by(by: &crate::ability::Filter) -> String {
    format!(
        "{BLOCKED_OR_WAS_BLOCKED_BY}{}",
        serde_json::to_string(by).unwrap_or_default()
    )
}

/// The (blocker, attacker) pairs of blocks made this turn.
fn blocks_this_turn(g: &Game) -> Vec<(ObjectId, ObjectId)> {
    let mut v = Vec::new();
    for e in &g.turn_events {
        match e {
            Event::BlockersDeclared { blocks } => v.extend(blocks.iter().copied()),
            Event::BlockAdded {
                blocker, attacker, ..
            } => v.push((*blocker, *attacker)),
            _ => {}
        }
    }
    v
}

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
        if name == DEALT_DAMAGE_THIS_TURN {
            return Some(g.turn_events.iter().any(|e| {
                matches!(e, Event::Damage { source, amount, .. } if *source == id && *amount > 0)
            }));
        }
        if name == BLOCKED_THIS_TURN {
            return Some(blocks_this_turn(g).iter().any(|(b, _)| *b == id));
        }
        if let Some(json) = name.strip_prefix(BLOCKED_OR_WAS_BLOCKED_BY) {
            let by: crate::ability::Filter = serde_json::from_str(json).ok()?;
            let other_ok = |o: ObjectId| {
                g.matches_view(&crate::eval::Current, o, &by, ctx)
            };
            return Some(blocks_this_turn(g).iter().any(|(b, a)| {
                (*b == id && other_ok(*a)) || (*a == id && other_ok(*b))
            }));
        }
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
