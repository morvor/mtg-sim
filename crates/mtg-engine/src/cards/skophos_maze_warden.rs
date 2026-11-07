//! Skophos Maze-Warden: "Whenever another creature becomes the target of an ability of a
//! land you control named Labyrinth of Skophos, you may have ~ fight that creature." The
//! trigger resolves first; if either creature has left the battlefield, no damage is dealt
//! (rulings, CR 701.14b).

use super::ManualAbility;
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, StackKind};
use crate::types::{CardType, Entity, ObjectId, PlayerId};

const TARGETED: &str = "card:Skophos Maze-Warden:a creature becomes the target of your Labyrinth of Skophos";
const LABYRINTH: &str = "Labyrinth of Skophos";
const TEXT: &str = "Whenever another creature becomes the target of an ability of a land you control named Labyrinth of Skophos, you may have ~ fight that creature.";

inventory::submit! { ManualAbility {
    card: "Skophos Maze-Warden",
    face: 0,
    text: TEXT,
    build: |_| {
        let body = Body::effect(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::Fight {
                a: Sel::This,
                b: Sel::TriggerObject,
            }),
        });
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(TriggerCond::Custom(TARGETED.into()), body)),
            TEXT,
        )]
    },
    reason: "triggers on a creature becoming the target of an ability of a land named Labyrinth of Skophos: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != TARGETED {
            return None;
        }
        let Event::BecameTarget {
            target: Entity::Object(c),
            by,
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        let from_labyrinth = match g.obj(*by).stack.as_deref().map(|s| &s.kind) {
            Some(StackKind::Activated { source, .. } | StackKind::Triggered { source, .. }) => {
                let s = g.obj(*source);
                s.is(CardType::Land) && s.controller == ctl && s.chars.name == LABYRINTH
            }
            _ => false,
        };
        Some(
            if from_labyrinth && *c != src && g.obj(*c).is_creature() {
                vec![EventInfo {
                    object: Some(*c),
                    ..Default::default()
                }]
            } else {
                vec![]
            },
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
