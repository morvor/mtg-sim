//! Zabaz, the Glimmerwasp: "If a modular triggered ability would put one or more +1/+1
//! counters on a creature you control, that many plus one +1/+1 counters are put on it
//! instead." The counters are put by a resolving modular triggered ability (CR 702.43a);
//! Zabaz must be on the battlefield, so its own modular isn't affected (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::StackKind;

const MODULAR: &str = "card:Zabaz, the Glimmerwasp:put by a modular triggered ability";
const TEXT: &str = "If a modular triggered ability would put one or more +1/+1 counters on a creature you control, that many plus one +1/+1 counters are put on it instead.";

inventory::submit! { ManualAbility {
    card: "Zabaz, the Glimmerwasp",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(s) = &mut kind {
                    if let StaticEffect::Replacement(def) = &mut s.effect {
                        def.event = ReplacementEvent::Where {
                            event: Box::new(def.event.clone()),
                            cond: Condition::Custom(MODULAR.into()),
                        };
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "a counter replacement for counters put by modular triggered abilities: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// The resolving stack object is a modular keyword's triggered ability.
    fn custom_condition(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
        if name != MODULAR {
            return None;
        }
        Some(g.stack.last().is_some_and(|top| {
            matches!(
                g.obj(*top).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { ability, .. })
                    if ability.text.starts_with("Modular")
                        && matches!(ability.kind, AbilityKind::Triggered(_))
            )
        }))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
