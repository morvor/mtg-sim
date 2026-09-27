//! CR 702.162 More Than Meets the Eye: "More Than Meets the Eye [cost]" means "You may
//! cast this card converted by paying [cost] rather than its mana cost." (CR 702.162a).
//! It functions in any zone from which the spell may be cast; the cost is an alternative
//! cost (CR 601.2b, 601.2f–h).
//!
//! A spell cast converted is put on the stack with its back face up and has only the
//! characteristics of its back face, but its mana value is that of its front face
//! (CR 712.8c, 712.11a); it enters the battlefield with that face up (CR 712.13).

use super::{KeywordRegistration, KeywordRules};
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast converted with More Than
/// Meets the Eye.
pub const MORE_THAN_MEETS_THE_EYE: &str = "more than meets the eye";

pub struct MoreThanMeetsTheEye;

impl KeywordRules for MoreThanMeetsTheEye {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::MoreThanMeetsTheEye]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        // Only a double-faced card can be cast converted.
        let o = g.obj(card);
        if o.card.as_ref().and_then(|d| d.back()).is_none() {
            return vec![];
        }
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Back);
        opt.method = CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye);
        // From any zone the card could be cast from: its owner's hand, or a zone an
        // effect lets its controller cast it from.
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
            KeywordKind::MoreThanMeetsTheEye,
            &cost,
        ));
        opt.tag = Some(MORE_THAN_MEETS_THE_EYE);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&MoreThanMeetsTheEye) }
