//! CR 702.43 Modular: "Modular N" means "This permanent enters with N +1/+1 counters on
//! it" and "When this permanent is put into a graveyard from the battlefield, you may put
//! a +1/+1 counter on target artifact creature for each +1/+1 counter on this permanent."
//! (CR 702.43a). Each instance works separately (CR 702.43b).
//!
//! "Modular—Sunburst" (CR 702.44c) enters with a +1/+1 counter for each color of mana
//! spent to cast it instead of N; it's marked by [`SUNBURST_TEXT`] in the keyword's text.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The text of the "Modular—Sunburst" variant.
pub const SUNBURST_TEXT: &str = "Modular—Sunburst";

/// Whether the modular keyword is "Modular—Sunburst".
pub fn is_sunburst(kw: &Keyword) -> bool {
    kw.text
        .as_deref()
        .is_some_and(|t| t.to_lowercase().contains("sunburst"))
}

pub struct Modular;

impl KeywordRules for Modular {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Modular]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let (enters, text) = if is_sunburst(kw) {
            // CR 702.44c: the number is set by sunburst, whatever the object's types.
            (
                ReplacementAction::AsEnters(Box::new(Effect::Custom(SmolStr::new(
                    super::sunburst::SUNBURST_PLUS1,
                )))),
                SUNBURST_TEXT.to_string(),
            )
        } else {
            let n = kw.n.unwrap_or(0);
            (
                ReplacementAction::EnterWithCounters(
                    SmolStr::new(counters::PLUS1),
                    Value::c(n),
                ),
                format!("Modular {n}"),
            )
        };
        let replacement = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: enters,
            self_replacement: false,
            optional: false,
        }));
        let target = TargetSpec::object(
            Filter::And(vec![
                Filter::Type(CardType::Artifact),
                Filter::Type(CardType::Creature),
            ]),
            "target artifact creature",
        );
        // "for each +1/+1 counter on this permanent": as it last existed on the
        // battlefield.
        let effect = Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::AddCounters {
                what: Sel::Target(0),
                kind: SmolStr::new(counters::PLUS1),
                n: Value::CountersOn(
                    Box::new(Sel::TriggerLki),
                    Some(SmolStr::new(counters::PLUS1)),
                ),
            }),
        };
        let dies = TriggeredAbility::new(
            TriggerCond::Dies(Filter::Source),
            Body::simple(vec![target], effect),
        );
        Some(vec![
            AbilityDef::new(AbilityKind::Static(replacement), text.clone()),
            AbilityDef::new(AbilityKind::Triggered(dies), text),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Modular) }
