//! CR 702.72 Champion: "Champion an [object]" means "When this permanent enters, sacrifice
//! it unless you exile another [object] you control" and "When this permanent leaves the
//! battlefield, return the exiled card to the battlefield under its owner's control". The
//! two abilities are linked (CR 607.2k, 702.72b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Champion;

impl KeywordRules for Champion {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Champion]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let what = kw.filter.clone().unwrap_or(Filter::Permanent);
        const V: Var = vars::USER + 70;
        let enters = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::Seq(vec![
                Effect::Exile {
                    what: Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::And(vec![
                            what,
                            Filter::Other,
                            Filter::ControlledBy(PlayerRel::You),
                        ]),
                        count: Value::c(1),
                        up_to: true,
                        store: None,
                    },
                    face_down: false,
                    link: true,
                },
                Effect::If {
                    cond: Condition::Not(Box::new(Condition::PrevAffectedAny)),
                    then: Box::new(Effect::SacrificeObjects { what: Sel::This }),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        );
        let leaves = TriggeredAbility::new(
            TriggerCond::LeavesBattlefield(Filter::Source),
            Body::effect(Effect::ForEach {
                sel: Sel::Linked,
                var: V,
                effect: Box::new(Effect::Move {
                    what: Sel::Var(V),
                    to: {
                        let mut d = Destination::battlefield();
                        d.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(V))));
                        d
                    },
                }),
            }),
        );
        Some(vec![
            AbilityDef::new(AbilityKind::Triggered(enters), "Champion"),
            AbilityDef::new(AbilityKind::Triggered(leaves), "Champion"),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Champion) }
