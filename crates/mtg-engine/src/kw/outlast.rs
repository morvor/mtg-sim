//! CR 702.107 Outlast: "Outlast [cost]" means "[Cost], {T}: Put a +1/+1 counter on this
//! creature. Activate only as a sorcery." (CR 702.107a). Its {T} means it can't be
//! activated unless the creature has been under its controller's control continuously
//! since their most recent turn began (CR 302.6).
//!
//! "Whenever you activate ~'s outlast ability" (Herald of Anafenza) is an
//! [`TriggerCond::AbilityActivated`] trigger qualified by [`OUTLAST_ACTIVATED`].

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::StackKind;
use crate::types::counters;

/// `Condition::Custom`: the activated ability of the trigger event is an outlast ability.
pub const OUTLAST_ACTIVATED: &str = "outlast:the activated ability is an outlast ability";

pub struct Outlast;

impl KeywordRules for Outlast {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Outlast]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = kw.cost.clone().unwrap_or_default().with(CostPart::Tap);
        let mut act = ActivatedAbility::new(
            cost,
            Body::effect(Effect::AddCounters {
                what: Sel::This,
                kind: counters::PLUS1.into(),
                n: Value::c(1),
            }),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Outlast.name(),
        )])
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != OUTLAST_ACTIVATED {
            return None;
        }
        let Some(ab) = ctx.event.as_ref().and_then(|e| e.spell) else {
            return Some(false);
        };
        Some(g.obj(ab).stack.as_deref().is_some_and(|si| {
            matches!(&si.kind, StackKind::Activated { ability, .. }
                if crate::keyword_impls::ability_from_keyword(ability) == Some(KeywordKind::Outlast))
        }))
    }
}

inventory::submit! { KeywordRegistration(&Outlast) }
