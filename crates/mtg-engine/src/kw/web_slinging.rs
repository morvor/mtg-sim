//! CR 702.188 Web-slinging: "Web-slinging [cost]" means "You may cast this spell by
//! paying [cost] and returning a tapped creature you control to its owner's hand rather
//! than paying its mana cost." (CR 702.188a). It's an alternative cost (CR 601.2b,
//! 601.2f–h): timing rules apply, it can't be combined with another alternative cost,
//! and additional costs are still paid. The creature is returned as the total cost is
//! paid, after mana abilities are activated (CR 601.2g–h): a creature tapped for mana
//! while casting the spell may be returned.
//!
//! "If [this spell] was cast using web-slinging" is `Condition::CostPaid(WEB_SLINGING)`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast using web-slinging.
pub const WEB_SLINGING: &str = "web-slinging";

pub struct WebSlinging;

impl KeywordRules for WebSlinging {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::WebSlinging]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::WebSlinging);
        // From its owner's hand, or a zone an effect lets it be cast from.
        if g.obj(card).zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        let mut cost = super::modified_keyword_cost(g, p, KeywordKind::WebSlinging, &cost);
        cost.parts.push(CostPart::ReturnToHand {
            filter: Filter::and(vec![
                Filter::Type(CardType::Creature),
                Filter::Tapped,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            count: Value::c(1),
        });
        opt.alt_cost = Some(cost);
        opt.tag = Some(WEB_SLINGING);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&WebSlinging) }
