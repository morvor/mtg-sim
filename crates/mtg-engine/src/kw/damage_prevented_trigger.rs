//! "Whenever damage that would be dealt to you is prevented" (Selfless Squire): a trigger
//! on a prevention effect preventing damage to the player (CR 615.13: once each time a
//! prevention effect is applied to one or more simultaneous damage events). "That many" is
//! the damage prevented. See `oracle/patterns/trigger_grammar_causes.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::EventInfo;
use crate::types::{Entity, ObjectId, PlayerId};

/// `TriggerCond::Custom` name: damage that would be dealt to the ability's controller is
/// prevented.
pub const DAMAGE_TO_YOU_PREVENTED: &str = "damage that would be dealt to you is prevented";

struct DamagePreventedTrigger;

impl KeywordRules for DamagePreventedTrigger {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != DAMAGE_TO_YOU_PREVENTED {
            return None;
        }
        Some(match ev {
            Event::DamagePrevented {
                source,
                target: Entity::Player(p),
                amount,
                ..
            } if *p == ctl => vec![EventInfo {
                player: Some(*p),
                other: Some(*source),
                amount: *amount as i32,
                ..Default::default()
            }],
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&DamagePreventedTrigger) }
