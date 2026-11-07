//! CR 702.23 Rampage.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Rampage;

impl KeywordRules for Rampage {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Rampage]
    }

    /// CR 702.23a: "Rampage N" means "Whenever this creature becomes blocked, it gets
    /// +N/+N until end of turn for each creature blocking it beyond the first."
    /// CR 702.23b: the bonus is calculated once, as the ability resolves (the value is
    /// locked in then, CR 608.2h). CR 702.23c: each instance triggers separately.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        let beyond_first = Value::Max(
            Box::new(Value::Diff(
                Box::new(Value::Count(Filter::and(vec![
                    Filter::creature(),
                    Filter::BlockingSource,
                ]))),
                Box::new(Value::c(1)),
            )),
            Box::new(Value::c(0)),
        );
        let bonus = Value::Mul(Box::new(Value::c(n)), Box::new(beyond_first));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BecomesBlocked(Filter::Source),
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(bonus.clone(), bonus)],
                    duration: Duration::EndOfTurn,
                }),
            )),
            format!("Rampage {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Rampage) }
