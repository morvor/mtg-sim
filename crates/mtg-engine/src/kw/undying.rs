//! CR 702.93 Undying.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::counters;
use smol_str::SmolStr;

pub struct Undying;

impl KeywordRules for Undying {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Undying]
    }

    /// CR 702.93a: "Undying" means "When this permanent is put into a graveyard from the
    /// battlefield, if it had no +1/+1 counters on it, return it to the battlefield under
    /// its owner's control with a +1/+1 counter on it."
    ///
    /// Whether it had +1/+1 counters is checked with its last known information, both
    /// when it triggers and on resolution (an intervening "if" clause, CR 603.4). "It" is
    /// the card in the graveyard: if it left the graveyard (or it's a token, which ceases
    /// to exist there), it isn't returned (CR 400.7).
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let plus = SmolStr::new(counters::PLUS1);
        let mut t = TriggeredAbility::new(
            TriggerCond::Dies(Filter::Source),
            Body::effect(Effect::Move {
                what: Sel::TriggerObject,
                to: {
                    let mut d = Destination::battlefield();
                    d.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::TriggerObject)));
                    d.with_counters = vec![(plus.clone(), Value::c(1))];
                    d
                },
            }),
        );
        t.intervening_if = Some(Condition::Compare(
            Value::CountersOn(Box::new(Sel::TriggerLki), Some(plus)),
            Cmp::Eq,
            Value::c(0),
        ));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Undying.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Undying) }
