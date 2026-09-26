//! CR 702.138 Escape: a static ability that functions while the card with escape is in a
//! player's graveyard. "Escape [cost]" means "You may cast this card from your graveyard
//! by paying [cost] rather than paying its mana cost." (CR 702.138a). Casting it follows
//! the rules for alternative costs (CR 601.2b, 601.2f–h): normal timing applies, and its
//! mana value doesn't change.
//!
//! A spell or permanent "escaped" if that spell, or the spell that became that permanent
//! as it resolved, was cast from a graveyard with an escape ability (CR 702.138b): the
//! [`ESCAPE`] name is recorded in its cast information (`CastInfo::paid`), which the
//! permanent keeps, and `Condition::CostPaid(ESCAPE)` asks whether it escaped. "[This
//! permanent] escapes with [counters]" (CR 702.138c) and "escapes with [ability]"
//! (CR 702.138d) are compiled from oracle text as abilities with that condition (see
//! `oracle/patterns/k702_125_139.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when a spell is cast with its escape ability.
pub const ESCAPE: &str = "escape";

/// [`Keyword::text`] of an escape ability granted with "The escape cost is equal to the
/// card's mana cost plus [cost]": its [`Keyword::cost`] holds only the added part.
pub const MANA_COST_PLUS: &str = "escape: mana cost plus";

/// "It escaped" (CR 702.138b).
pub fn escaped() -> Condition {
    Condition::CostPaid(SmolStr::new(ESCAPE))
}

pub struct Escape;

impl KeywordRules for Escape {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Escape]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Escape);
        let cost = if kw.text.as_deref() == Some(MANA_COST_PLUS) {
            // The card's mana cost (an unpayable cost if it has none, CR 118.6) plus the
            // rest.
            let chars = g.option_characteristics(card, &opt);
            Cost {
                mana: Some(
                    chars
                        .mana_cost
                        .clone()
                        .unwrap_or_else(crate::cost_rules::unpayable),
                ),
                parts: cost.parts,
            }
        } else {
            cost
        };
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Escape,
            &cost,
        ));
        opt.tag = Some(ESCAPE);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Escape) }
