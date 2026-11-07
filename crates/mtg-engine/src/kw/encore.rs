//! CR 702.141 Encore: an activated ability that functions while the card is in a
//! graveyard. "Encore [cost]" means "[Cost], Exile this card from your graveyard: For each
//! opponent, create a token that's a copy of this card that attacks that opponent this
//! turn if able. The tokens gain haste. Sacrifice them at the beginning of the next end
//! step. Activate only as a sorcery." (CR 702.141a).
//!
//! The tokens copy the card as it last existed in the graveyard. Each token's requirement
//! names the opponent it was created for; the haste and the sacrifice come from the
//! resolving ability, so they keep applying if the tokens lose their abilities.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

/// The tokens created so far by the resolving encore ability.
const TOKENS: Var = vars::USER + 1410;

pub struct Encore;

impl KeywordRules for Encore {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Encore]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let created = || Sel::Var(vars::CREATED);
        let tokens = || Sel::Var(TOKENS);
        let per_opponent = Effect::Seq(vec![
            // "create a token that's a copy of this card ..."
            Effect::CreateTokenCopy {
                of: Sel::This,
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
                mods: vec![],
            },
            // "... that attacks that opponent this turn if able."
            Effect::AddRestriction {
                restriction: Restriction::MustAttackPlayer {
                    attackers: Filter::In(Box::new(created())),
                    defender: PlayerFilter::Ref(Box::new(PlayerRef::Iterated)),
                },
                duration: Duration::EndOfTurn,
            },
            Effect::Store {
                var: TOKENS,
                sel: Sel::Union(vec![tokens(), created()]),
            },
        ]);
        let effect = Effect::Seq(vec![
            Effect::Store {
                var: TOKENS,
                sel: Sel::None,
            },
            Effect::ForEachPlayer {
                who: PlayerRef::EachOpponent,
                effect: Box::new(per_opponent),
            },
            // "The tokens gain haste."
            Effect::Modify {
                what: tokens(),
                mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
                duration: Duration::Permanent,
            },
            // "Sacrifice them at the beginning of the next end step": only those its
            // controller still controls (CR 701.21a).
            Effect::AtNext {
                step: TriggerStep::End,
                effect: Box::new(Effect::SacrificeObjects {
                    what: Sel::All(Filter::And(vec![
                        Filter::In(Box::new(tokens())),
                        Filter::ControlledBy(PlayerRel::You),
                    ])),
                }),
            },
        ]);
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::ExileSelf);
        let mut act = ActivatedAbility::new(cost, Body::effect(effect));
        act.timing = ActivationTiming::Sorcery;
        act.zone = FunctionZone::Graveyard;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Encore.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Encore) }
