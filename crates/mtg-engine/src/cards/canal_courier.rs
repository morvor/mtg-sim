//! Canal Courier: "Whenever ~ and another creature attack different players, ~ can't be
//! blocked this combat." Attacking a planeswalker isn't attacking a player (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::EventInfo;
use crate::types::{Entity, ObjectId, PlayerId};

const SPLIT: &str = "card:Canal Courier:it and another creature attack different players";
const TEXT: &str = "Whenever ~ and another creature attack different players, ~ can't be blocked this combat.";

inventory::submit! { ManualAbility {
    card: "Canal Courier",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever ~ attacks, ~ can't be blocked this combat.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.trigger = TriggerCond::Custom(SPLIT.into());
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "triggers when it and another creature attack different players: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != SPLIT {
            return None;
        }
        let Event::AttackersDeclared { attackers, .. } = ev else {
            return Some(vec![]);
        };
        let Some(Entity::Player(mine)) = attackers.iter().find(|(a, _)| *a == src).map(|(_, t)| *t)
        else {
            return Some(vec![]);
        };
        let split = attackers
            .iter()
            .any(|(a, t)| *a != src && matches!(t, Entity::Player(p) if *p != mine));
        Some(if split {
            vec![EventInfo {
                object: Some(src),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
