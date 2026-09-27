//! CR 702.156 Ravenous: "This permanent enters with X +1/+1 counters on it" (a replacement
//! effect) and "When this permanent enters, if X is 5 or more, draw a card" (a triggered
//! ability) (CR 702.156a).
//!
//! X is the value chosen for the spell that became the permanent (CR 107.3m); a permanent
//! that wasn't a spell with a value of X (e.g. one put onto the battlefield, or entering as
//! a copy of a creature with ravenous) enters with no counters and draws nothing.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// `Value::Custom`: the value of X of the spell the ravenous permanent was (CR 107.3m),
/// for its enters ability, both as it triggers and as it resolves.
pub const SPELL_X: &str = "ravenous:X of its spell";

pub struct Ravenous;

impl KeywordRules for Ravenous {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Ravenous]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Ravenous.name();
        let counters = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::EnterWithCounters(counters::PLUS1.into(), Value::X),
            self_replacement: false,
            optional: false,
        }));
        let x = Value::Custom(SPELL_X.into());
        let mut draw = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(1),
            }),
        );
        draw.intervening_if = Some(Condition::Compare(x, Cmp::Ge, Value::c(5)));
        Some(vec![
            AbilityDef::new(AbilityKind::Static(counters), text),
            AbilityDef::new(AbilityKind::Triggered(draw), text),
        ])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == SPELL_X).then(|| {
            g.cast_info(ctx)
                .and_then(|c| c.x)
                .map_or(0, |x| x.max(0) as i64)
        })
    }
}

inventory::submit! { KeywordRegistration(&Ravenous) }
