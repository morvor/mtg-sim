//! "You may have ~ assign its combat damage as though it weren't blocked." (Thorn
//! Elemental, Pride of Lions): a global hook for the combat damage step. As its combat
//! damage is assigned (CR 510.1), a blocked attacking creature with this ability may have
//! its controller assign all of it to the player, planeswalker, or battle it's attacking
//! instead of to the blocking creatures — all of it one way or the other, not split. If
//! a creature with banding blocks it, the defending player makes that choice.
//! The ability is `StaticEffect::Custom(MAY_ASSIGN_UNBLOCKED)`, including when granted
//! ("creatures you control have \"You may have ~ assign ...\"").
//!
//! "You may have creatures you control assign their combat damage this turn as though
//! they weren't blocked." (Predatory Focus): the choice is made as the spell resolves; if
//! its controller chose to, the player gets `PlayerModification::Custom(ALL_ASSIGN_UNBLOCKED)`
//! for the turn, and every creature they control as its combat damage is assigned —
//! including creatures that weren't on the battlefield then — assigns all of it as though
//! it weren't blocked, with no further choice.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, PlayerModification, StaticEffect};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `StaticEffect::Custom` name of the ability.
pub const MAY_ASSIGN_UNBLOCKED: &str = "may assign combat damage as though unblocked";
/// `PlayerModification::Custom` name: creatures the player controls assign their combat
/// damage as though they weren't blocked.
pub const ALL_ASSIGN_UNBLOCKED: &str = "creatures assign combat damage as though unblocked";

pub struct AssignAsThoughUnblocked;

fn has_ability(g: &Game, id: ObjectId) -> bool {
    g.obj(id).chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if matches!(&s.effect, StaticEffect::Custom(n) if n == MAY_ASSIGN_UNBLOCKED))
    })
}

impl KeywordRules for AssignAsThoughUnblocked {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn assigns_as_though_unblocked(&self, g: &mut Game, creature: ObjectId) -> bool {
        let controller = g.obj(creature).controller;
        if g.player(controller).has_mod(
            |m| matches!(m, PlayerModification::Custom(n) if n == ALL_ASSIGN_UNBLOCKED),
        ) {
            return true;
        }
        if !has_ability(g, creature) {
            return false;
        }
        // Blocked by a creature with banding: the defending player decides, as they assign
        // its damage (CR 702.22j).
        let p = super::combat_damage_assigner(g, creature).unwrap_or(g.obj(creature).controller);
        g.ask_yes_no(
            p,
            Some(creature),
            "Assign its combat damage as though it weren't blocked?",
            false,
        )
    }
}

inventory::submit! { KeywordRegistration(&AssignAsThoughUnblocked) }
