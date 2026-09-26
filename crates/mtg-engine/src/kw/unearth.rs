//! CR 702.84 Unearth.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Unearth;

impl KeywordRules for Unearth {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Unearth]
    }

    /// CR 702.84a: "Unearth [cost]" means "[Cost]: Return this card from your graveyard to
    /// the battlefield. It gains haste. Exile it at the beginning of the next end step. If
    /// it would leave the battlefield, exile it instead of putting it anywhere else.
    /// Activate only as a sorcery." The ability functions while the card is in a
    /// graveyard.
    ///
    /// The haste, the delayed triggered ability, and the replacement effect all come from
    /// the resolving ability, not from abilities the permanent has: they keep applying if
    /// it loses its abilities, and they don't apply to a new object it becomes after a
    /// zone change (CR 400.7). If the card left the graveyard before the ability resolves,
    /// nothing happens.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let it = || Sel::Var(vars::IT);
        let exile = Destination::zone(ZoneKind::Exile);
        let mut act = ActivatedAbility::new(
            kw.cost.clone().unwrap_or_default(),
            Body::effect(Effect::Seq(vec![
                Effect::Move {
                    what: Sel::This,
                    to: Destination::battlefield(),
                },
                Effect::If {
                    cond: Condition::SelNonEmpty(it()),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Modify {
                            what: it(),
                            mods: vec![Modification::AddKeyword(Keyword::new(
                                KeywordKind::Haste,
                            ))],
                            duration: Duration::Permanent,
                        },
                        Effect::AtNext {
                            step: TriggerStep::End,
                            effect: Box::new(Effect::Exile {
                                what: it(),
                                face_down: false,
                                link: false,
                            }),
                        },
                        Effect::AddReplacement {
                            def: ReplacementDef {
                                event: ReplacementEvent::ZoneChange {
                                    filter: Filter::In(Box::new(it())),
                                    from: Some(ZoneKind::Battlefield),
                                    to: None,
                                },
                                action: ReplacementAction::MoveInstead(exile),
                                self_replacement: false,
                                optional: false,
                            },
                            duration: Duration::Permanent,
                            uses: None,
                        },
                    ])),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        );
        act.timing = ActivationTiming::Sorcery;
        act.zone = FunctionZone::Graveyard;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Unearth.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Unearth) }
