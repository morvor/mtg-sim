//! CR 702.64 Absorb. "Absorb N" means "If a source would deal damage to this creature,
//! prevent N of that damage." (CR 702.64a). It's a prevention effect of a static ability
//! (CR 615): it applies once to each event of damage from one source, preventing up to N
//! of it (CR 702.64b; damage from several sources at once is several events, and each
//! later event is a new one). Each instance applies separately (CR 702.64c): each is its
//! own ability, and each applies once to an event (CR 614.5).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Absorb;

impl KeywordRules for Absorb {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Absorb]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Damage {
                source: Filter::Any,
                to_players: None,
                to_objects: Some(Filter::Source),
                combat_only: false,
            },
            action: ReplacementAction::PreventAmount(Value::c(n)),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Absorb.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Absorb) }
