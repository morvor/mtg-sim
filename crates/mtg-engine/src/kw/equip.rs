//! CR 702.6 Equip.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::CardType;

pub struct Equip;

/// Whether an equip quality names planeswalkers ("Equip planeswalker", "Equip creature or
/// planeswalker").
fn mentions_planeswalker(f: &Filter) -> bool {
    match f {
        Filter::Type(CardType::Planeswalker) => true,
        Filter::Or(v) | Filter::And(v) => v.iter().any(mentions_planeswalker),
        _ => false,
    }
}

impl KeywordRules for Equip {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Equip]
    }

    /// CR 702.6a: "Equip [cost]" means "[Cost]: Attach this permanent to target creature
    /// you control. Activate only as a sorcery."
    /// CR 702.6c: "Equip [quality] [cost]" may target only a creature you control with that
    /// quality ("Equip legendary creature", "Equip Knight", "Equip commander").
    /// CR 702.6e: "Equip planeswalker [cost]" attaches it to target planeswalker you control
    /// as though that planeswalker were a creature.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = kw.cost.clone().unwrap_or_default();
        let (quality, effect, text) = match &kw.filter {
            Some(f) if mentions_planeswalker(f) => (
                f.clone(),
                Effect::AttachAsCreature {
                    what: Sel::This,
                    to: Sel::Target(0),
                },
                "target planeswalker you control",
            ),
            other => (
                Filter::and(vec![
                    Filter::creature(),
                    other.clone().unwrap_or(Filter::Any),
                ]),
                Effect::Attach {
                    what: Sel::This,
                    to: Sel::Target(0),
                },
                "target creature you control",
            ),
        };
        let mut act = ActivatedAbility::new(
            cost,
            Body::simple(
                vec![TargetSpec::object(
                    Filter::and(vec![
                        quality,
                        Filter::ControlledBy(PlayerRel::You),
                        // CR 301.5c: an Equipment can't equip itself.
                        Filter::Other,
                    ]),
                    text,
                )],
                effect,
            ),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(AbilityKind::Activated(act), "Equip")])
    }
}

inventory::submit! { KeywordRegistration(&Equip) }
