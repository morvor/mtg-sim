//! CR 702.109 Dash: "Dash [cost]" means "You may cast this card by paying [cost] rather
//! than its mana cost," "If this spell's dash cost was paid, return the permanent this
//! spell becomes to its owner's hand at the beginning of the next end step," and "As long
//! as this permanent's dash cost was paid, it has haste." (CR 702.109a). The dash cost is
//! an alternative cost (CR 601.2b, 601.2f–h): the spell is cast only when it otherwise
//! could be, and its mana value is still that of its mana cost.
//!
//! The delayed triggered ability is created as the spell resolves and the permanent
//! enters (CR 608.3g); it returns only that permanent, if it's still on the battlefield.
//! A copy of a permanent whose dash cost was paid wasn't cast for its dash cost, so it
//! neither has haste nor returns.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its dash cost.
pub const DASH: &str = "dash";

pub struct Dash;

impl KeywordRules for Dash {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Dash]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Dash.name();
        let mut ret = StaticAbility::new(StaticEffect::DelayedTriggerAsEnters {
            condition: Some(Condition::CostPaid(DASH.into())),
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::Any,
            },
            body: Body::effect(Effect::Move {
                what: Sel::Var(vars::IT),
                to: Destination::zone(ZoneKind::Hand),
            }),
        });
        ret.zone = FunctionZone::Stack;
        // "As long as this permanent's dash cost was paid, it has haste": the permanent's
        // own static ability, an ability-adding effect (layer 6, CR 613.1f) with its
        // timestamp (CR 613.7a). An effect that removes the permanent's abilities removes
        // this one too, and a copy of the permanent (which wasn't cast for its dash cost)
        // doesn't have haste.
        let mut haste = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
        });
        haste.condition = Some(Condition::CostPaid(DASH.into()));
        Some(vec![
            AbilityDef::new(AbilityKind::Static(ret), text),
            AbilityDef::new(AbilityKind::Static(haste), text),
        ])
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Dash);
        // Only from a zone the card could be cast from (its owner's hand, or a zone an
        // effect lets them cast it from).
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        // "Dash costs you pay cost {1} less" (Warbringer).
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Dash,
            &cost,
        ));
        opt.tag = Some(DASH);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Dash) }
