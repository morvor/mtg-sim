//! CR 702.81 Retrace: "You may cast this card from your graveyard by discarding a land
//! card as an additional cost to cast it." (CR 702.81a). A static ability that functions
//! while the card is in a player's graveyard; casting it follows the rules for additional
//! costs (CR 601.2b, 601.2f–h), and the spell's timing is the normal one for its type. It
//! goes back to the graveyard as usual when it resolves or is countered.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast with retrace.
pub const RETRACE: &str = "retrace";

pub struct Retrace;

impl KeywordRules for Retrace {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Retrace]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, _kw: &Keyword) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Retrace);
        opt.extra_cost = Some(Cost::free().with(CostPart::Discard {
            filter: Filter::and(vec![Filter::Type(CardType::Land), Filter::Card]),
            count: Value::c(1),
            random: false,
        }));
        opt.tag = Some(RETRACE);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Retrace) }
