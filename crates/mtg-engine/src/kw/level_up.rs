//! CR 702.87 Level up. A card printed with a level up ability is a leveler card (CR 711):
//! its level symbols ("LEVEL N1-N2", "LEVEL N3+") are static abilities compiled by the
//! oracle pattern in `oracle/patterns/r107_symbols.rs`, which give the permanent its
//! striation's power, toughness, and abilities while it has that many level counters.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::counters;

pub struct LevelUp;

impl KeywordRules for LevelUp {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::LevelUp]
    }

    /// CR 702.87a: "Level up [cost]" means "[Cost]: Put a level counter on this permanent.
    /// Activate only as a sorcery."
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let mut act = ActivatedAbility::new(
            kw.cost.clone().unwrap_or_default(),
            Body::effect(Effect::AddCounters {
                what: Sel::This,
                kind: counters::LEVEL.into(),
                n: Value::c(1),
            }),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::LevelUp.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&LevelUp) }
