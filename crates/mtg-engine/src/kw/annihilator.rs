//! CR 702.86 Annihilator.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Annihilator;

impl KeywordRules for Annihilator {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Annihilator]
    }

    /// "Annihilator X, where X is ..." granted by an effect (Ulamog, the Defiler): X is
    /// determined as the ability resolves (CR 608.2h), see [`Keyword::x`].
    fn x_determined_on_resolution(&self) -> bool {
        true
    }

    /// CR 702.86a: "Annihilator N" means "Whenever this creature attacks, defending player
    /// sacrifices N permanents." The defending player is the player this creature is
    /// attacking (or who controls the planeswalker or protects the battle it's attacking,
    /// CR 506.2). Each instance triggers separately (CR 702.86b).
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::effect(Effect::Sacrifice {
                    who: PlayerRef::DefendingPlayer,
                    filter: Filter::Permanent,
                    count: kw.x.clone().unwrap_or(Value::c(n)),
                }),
            )),
            format!("Annihilator {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Annihilator) }
