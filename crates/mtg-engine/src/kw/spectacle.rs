//! CR 702.137 Spectacle: a static ability that functions on the stack. "Spectacle [cost]"
//! means "You may pay [cost] rather than pay this spell's mana cost if an opponent lost
//! life this turn." (CR 702.137a). Casting a spell for its spectacle cost follows the
//! rules for alternative costs (CR 601.2b, 601.2f–h): its mana value doesn't change, and
//! it doesn't change when it can be cast.
//!
//! An opponent who lost life this turn and then left the game still counts (the life was
//! lost this turn); how much life was lost, or by how many opponents, doesn't matter.

use super::{KeywordRegistration, KeywordRules};
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its spectacle cost.
pub const SPECTACLE: &str = "spectacle";

/// Whether an opponent of `p` (including one who has since left the game) lost life this
/// turn.
pub fn opponent_lost_life(g: &Game, p: PlayerId) -> bool {
    g.players
        .iter()
        .any(|q| g.are_opponents(p, q.id) && g.history.life_lost.get(&q.id).is_some_and(|n| *n > 0))
}

pub struct Spectacle;

impl KeywordRules for Spectacle {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Spectacle]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        if !opponent_lost_life(g, p) {
            return vec![];
        }
        let o = g.obj(card);
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Spectacle);
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
            KeywordKind::Spectacle,
            &cost,
        ));
        opt.tag = Some(SPECTACLE);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Spectacle) }
