//! Tyvar the Bellicose: "Each creature you control has "Whenever a mana ability of ~
//! resolves, put a number of +1/+1 counters on it equal to the amount of mana ~ produced.
//! This ability triggers only once each turn."" The granted ability triggers once each
//! turn for each creature (ruling); it isn't a mana ability (it adds no mana), so it uses
//! the stack, resolving before a spell being cast with that mana (ruling).

use super::ManualAbility;
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::EventInfo;
use crate::types::{CardType, ObjectId, PlayerId};

const RESOLVED: &str = "card:Tyvar the Bellicose:a mana ability of this creature resolves";
const TEXT: &str = "Each creature you control has \"Whenever a mana ability of ~ resolves, put a number of +1/+1 counters on it equal to the amount of mana ~ produced. This ability triggers only once each turn.\"";
const GRANTED: &str = "Whenever a mana ability of this creature resolves, put a number of +1/+1 counters on it equal to the amount of mana this creature produced. This ability triggers only once each turn.";

inventory::submit! { ManualAbility {
    card: "Tyvar the Bellicose",
    face: 0,
    text: TEXT,
    build: |_| {
        let mut t = TriggeredAbility::new(
            TriggerCond::Custom(RESOLVED.into()),
            Body::effect(Effect::AddCounters {
                what: Sel::This,
                kind: "+1/+1".into(),
                n: Value::EventAmount,
            }),
        );
        t.once_per_turn = true;
        let granted = AbilityDef::new(AbilityKind::Triggered(t), GRANTED);
        vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::And(vec![
                    Filter::Type(CardType::Creature),
                    Filter::ControlledBy(PlayerRel::You),
                ]),
                mods: vec![Modification::AddAbility(granted)],
            })),
            TEXT,
        )]
    },
    reason: "a granted trigger on its mana abilities resolving, with the amount of mana produced: only this card",
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
        if name != RESOLVED {
            return None;
        }
        Some(match ev {
            Event::Custom {
                name,
                obj: Some(o),
                amount,
                player,
            } if name == crate::casting::MANA_ABILITY_RESOLVED && *o == src => vec![EventInfo {
                object: Some(src),
                player: *player,
                amount: *amount,
                ..Default::default()
            }],
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
