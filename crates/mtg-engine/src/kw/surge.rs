//! CR 702.117 Surge: "You may pay [cost] rather than pay this spell's mana cost as you
//! cast this spell if you or one of your teammates has cast another spell this turn."
//! (CR 702.117a). A static ability that functions on the stack; casting a spell for its
//! surge cost follows the rules for alternative costs (CR 601.2b, 601.2f–h), so the
//! spell's mana value is still that of its mana cost.
//!
//! "Another spell" is any spell cast earlier this turn (`TurnHistory::spells_cast`),
//! whether or not it resolved; a spell with surge being cast isn't in that list yet.

use super::{KeywordRegistration, KeywordRules};
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its surge cost.
pub const SURGE: &str = "surge";

/// Whether `p` or one of their teammates has cast a spell this turn.
pub fn surge_condition(g: &Game, p: PlayerId) -> bool {
    let team = g.teammates(p);
    g.history
        .spells_cast
        .iter()
        .any(|(q, _)| *q == p || team.contains(q))
}

pub struct Surge;

impl KeywordRules for Surge {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Surge]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        if !surge_condition(g, p) {
            return vec![];
        }
        let o = g.obj(card);
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Surge);
        // Only from a zone the card could be cast from.
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
            KeywordKind::Surge,
            &cost,
        ));
        opt.tag = Some(SURGE);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Surge) }
