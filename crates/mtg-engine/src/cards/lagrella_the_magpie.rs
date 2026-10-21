//! Lagrella, the Magpie: "When ~ enters, exile any number of other target creatures
//! controlled by different players until ~ leaves the battlefield. When an exiled card
//! enters under your control this way, put two +1/+1 counters on it." The second sentence
//! is a delayed triggered ability set up as the first resolves; it triggers for the cards
//! returning because Lagrella left (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{EventInfo, Zone};
use crate::types::{Entity, ObjectId, PlayerId};

const EXILED: &str = "card:Lagrella, the Magpie:cards exiled until it leaves";
const RETURNS: &str = "card:Lagrella, the Magpie:an exiled card enters under your control this way";
const TEXT: &str = "When ~ enters, exile any number of other target creatures controlled by different players until ~ leaves the battlefield. When an exiled card enters under your control this way, put two +1/+1 counters on it.";

inventory::submit! { ManualAbility {
    card: "Lagrella, the Magpie",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ enters, exile any number of other target creatures controlled by different players until ~ leaves the battlefield.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| {
                    Effect::Seq(vec![
                        e,
                        Effect::Custom(EXILED.into()),
                        Effect::DelayedTrigger {
                            trigger: TriggerCond::Custom(RETURNS.into()),
                            body: Box::new(Body::effect(Effect::AddCounters {
                                what: Sel::TriggerObject,
                                kind: "+1/+1".into(),
                                n: Value::Const(2),
                            })),
                            once: false,
                        },
                    ])
                })
            })
            .collect()
    },
    reason: "a delayed trigger for each card returning from its exile under your control: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (Lagrella, exiled card).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EXILED {
            return false;
        }
        let src = ctx.source.map_or(-1, |s| i64::from(s.0));
        let exiled: Vec<ObjectId> = g
            .exile
            .iter()
            .copied()
            .filter(|c| {
                ctx.targets.iter().flatten().any(|t| match t {
                    Entity::Object(o) => g.current(*o) == *c,
                    _ => false,
                })
            })
            .collect();
        for c in exiled {
            g.cards.push(EXILED, vec![src, i64::from(c.0)]);
        }
        true
    }
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != RETURNS {
            return None;
        }
        let Event::ZoneChange {
            old,
            new,
            from: Zone::Exile,
            to: Zone::Battlefield,
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        let ours = g
            .cards
            .get(EXILED)
            .iter()
            .any(|r| r[0] == i64::from(src.0) && r[1] == i64::from(old.0));
        Some(if ours && g.obj(*new).controller == ctl {
            vec![EventInfo {
                object: Some(*new),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
