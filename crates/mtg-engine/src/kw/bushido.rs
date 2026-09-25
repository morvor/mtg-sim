//! CR 702.45 Bushido. Template for keyword implementations: a `KeywordRules` impl
//! registered with `inventory::submit!`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Bushido;

impl KeywordRules for Bushido {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Bushido]
    }

    /// CR 702.45a: "Bushido N" means "Whenever this creature blocks or becomes blocked,
    /// it gets +N/+N until end of turn."
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BlocksOrBecomesBlocked(Filter::Source),
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(Value::c(n), Value::c(n))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            format!("Bushido {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Bushido) }
