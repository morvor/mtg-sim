//! CR 702.39 Provoke: "Whenever this creature attacks, you may choose to have target
//! creature defending player controls block this creature this combat if able. If you
//! do, untap that creature." (CR 702.39a). Each instance triggers separately
//! (CR 702.39b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

pub struct Provoke;

impl KeywordRules for Provoke {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Provoke]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let target = TargetSpec::object(
            Filter::And(vec![
                Filter::Type(CardType::Creature),
                Filter::ControlledBy(PlayerRel::Defending),
            ]),
            "target creature defending player controls",
        );
        let that = || Sel::Target(0);
        let effect = Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::Seq(vec![
                Effect::AddRestriction {
                    restriction: Restriction::MustBlockAttacker {
                        blocker: Filter::In(Box::new(that())),
                        attacker: Filter::Source,
                    },
                    duration: Duration::EndOfCombat,
                },
                Effect::Untap { what: that() },
            ])),
        };
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::simple(vec![target], effect),
            )),
            "Provoke",
        )])
    }
}

inventory::submit! { KeywordRegistration(&Provoke) }
