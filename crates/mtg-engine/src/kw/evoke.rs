//! CR 702.74 Evoke: "Evoke [cost]" means "You may cast this card by paying [cost] rather
//! than paying its mana cost" and "When this permanent enters, if its evoke cost was paid,
//! its controller sacrifices it." (CR 702.74a). The first is a static ability that
//! functions in any zone from which the card can be cast; the second is a triggered
//! ability that functions on the battlefield. Casting a spell for its evoke cost follows
//! the rules for alternative costs (CR 601.2b, 601.2f–h): its mana value is still that of
//! its mana cost.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its evoke cost.
pub const EVOKE: &str = "evoke";

pub struct Evoke;

impl KeywordRules for Evoke {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Evoke]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        // If it has left the battlefield by the time this resolves, "it" (this object)
        // is gone and nothing is sacrificed.
        let mut t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::SacrificeObjects { what: Sel::This }),
        );
        t.intervening_if = Some(Condition::CostPaid(EVOKE.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Evoke.name(),
        )])
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Evoke);
        // Only from a zone the card could be cast from (its owner's hand, or a zone an
        // effect lets them cast it from).
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Evoke,
            &cost,
        ));
        opt.tag = Some(EVOKE);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Evoke) }
