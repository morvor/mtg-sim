//! The Temporal Anchor: "Whenever you choose to put one or more cards on the bottom of your
//! library while scrying, exile that many cards from the bottom of your library." (Scrying
//! reports how many cards went to the bottom, `scry_rules.rs`; its other abilities compile.)

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::EventInfo;
use crate::types::{ObjectId, PlayerId};

const BOTTOMED: &str = "card:The Temporal Anchor:you put cards on the bottom while scrying";
const TEXT: &str = "Whenever you choose to put one or more cards on the bottom of your library while scrying, exile that many cards from the bottom of your library.";

inventory::submit! { ManualAbility {
    card: "The Temporal Anchor",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of your upkeep, exile the bottom card of your library.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.trigger = TriggerCond::Custom(BOTTOMED.into());
                    if let Effect::Exile { what: Sel::BottomOfLibrary(_, n), .. } = &mut t.body.effect {
                        *n = Value::EventAmount;
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "triggers on choosing to put cards on the bottom while scrying: only this card",
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
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != BOTTOMED {
            return None;
        }
        Some(match ev {
            Event::Custom {
                name,
                player: Some(p),
                amount,
                ..
            } if name == crate::scry_rules::SCRIED_TO_BOTTOM && *p == ctl && *amount > 0 => {
                vec![EventInfo {
                    player: Some(*p),
                    amount: *amount,
                    ..Default::default()
                }]
            }
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
