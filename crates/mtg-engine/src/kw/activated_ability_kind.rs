//! Conditions on the activated ability an "activates an ability" trigger event is about
//! (CR 602.2), for triggers qualified by its cost or kind: "Whenever a player activates an
//! ability of enchanted creature with {T} in its activation cost" (Imprison), "… an
//! artifact's ability without {T} in its activation cost" (Haunting Wind, CR 602.1a),
//! "Whenever you activate a ninjutsu ability" (Satoru Umezawa, CR 702.49).
//!
//! The ability is the one on the stack (`EventInfo::spell`), or, for a mana ability, which
//! never goes on the stack (CR 605.3a), the one its source just recorded as activated.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;

/// `Condition::Custom`: the activated ability has {T} in its activation cost.
pub const TAP_IN_COST: &str = "activated ability: {T} in its activation cost";
/// `Condition::Custom`: the activated ability is a ninjutsu ability (CR 702.49a).
pub const NINJUTSU: &str = "activated ability: ninjutsu";

/// The activated ability of the trigger event being evaluated.
fn event_ability(g: &Game, ctx: &Ctx) -> Option<ActivatedAbility> {
    let info = ctx.event.as_ref()?;
    if let Some(ab) = info.spell {
        return match g.obj(ab).stack.as_deref().map(|s| &s.kind) {
            Some(StackKind::Activated { ability, .. }) => match &ability.kind {
                AbilityKind::Activated(a) => Some(a.clone()),
                _ => None,
            },
            _ => None,
        };
    }
    // A mana ability: the last one recorded for its source.
    let src = info.object?;
    let (_, _, uid) = g.history.activated.iter().rev().find(|(_, s, _)| *s == src)?;
    g.obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| a.uid == *uid)
        .and_then(|a| match &a.kind {
            AbilityKind::Activated(act) => Some(act.clone()),
            _ => None,
        })
}

/// Whether the event's activated ability is a ninjutsu ability (its keyword's expansion).
fn is_ninjutsu(g: &Game, ctx: &Ctx) -> bool {
    let Some(info) = ctx.event.as_ref() else {
        return false;
    };
    let Some(ab) = info.spell else {
        return false;
    };
    matches!(
        g.obj(ab).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Activated { ability, .. }) if ability.text == KeywordKind::Ninjutsu.name()
    )
}

pub struct ActivatedAbilityKind;

impl KeywordRules for ActivatedAbilityKind {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        match name {
            TAP_IN_COST => Some(event_ability(g, ctx).is_some_and(|a| a.cost.has_tap())),
            NINJUTSU => Some(is_ninjutsu(g, ctx)),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&ActivatedAbilityKind) }
