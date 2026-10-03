//! CR 702.40 Storm: "When you cast this spell, copy it for each other spell that was cast
//! before it this turn. If the spell has any targets, you may choose new targets for any
//! of the copies." A triggered ability that functions on the stack (CR 702.40a); each
//! instance triggers separately (CR 702.40b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};

/// The number of spells cast this turn before the storm spell (the source).
pub const STORM_COUNT: &str = "storm:spells cast before this";

pub struct Storm;

impl KeywordRules for Storm {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Storm]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Body::effect(Effect::CopySpell {
                what: Sel::This,
                count: Value::Custom(STORM_COUNT.into()),
                new_targets: true,
            }),
        );
        t.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(AbilityKind::Triggered(t), "Storm")])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != STORM_COUNT {
            return None;
        }
        // Every spell cast before it counts, whoever cast it, even if it was countered or
        // cast from outside a hand; copies aren't cast and don't count. Spells cast after
        // it (in response to the trigger) don't count.
        let cast = &g.history.spells_cast;
        let n = ctx
            .source
            .and_then(|s| cast.iter().position(|(_, x)| *x == s))
            .unwrap_or(cast.len());
        Some(n as i64)
    }
}

inventory::submit! { KeywordRegistration(&Storm) }
