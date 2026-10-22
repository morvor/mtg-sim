//! CR 702.187 Mayhem.
//!
//! * Mayhem is a static ability that functions while the card is in a player's graveyard
//!   (CR 702.187a).
//! * "Mayhem [cost]" means "As long as you discarded this card this turn, you may cast it
//!   from your graveyard by paying [cost] rather than paying its mana cost." (CR 702.187b):
//!   an alternative cost (CR 601.2b, 601.2f–h); timing rules still apply and additional
//!   costs must be paid.
//! * "Mayhem" without a cost means "You may play this card from your graveyard if you
//!   discarded it this turn." (CR 702.187c): a land is played (using a land play,
//!   [`KeywordRules::playable_lands`]); a spell is cast paying its costs.
//! * A card discarded to a graveyard is recorded with the turn it was discarded (a mark
//!   on the object it became there, see `special_actions.rs`): once it leaves that
//!   graveyard it's a new object that wasn't discarded (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::casting::CastOption;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast for its mayhem cost.
pub const MAYHEM: &str = "mayhem";

/// Whether `p` discarded `card` this turn and it's still in their graveyard.
pub fn discarded_this_turn(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    let o = g.obj(card);
    o.zone == Zone::Graveyard(p)
        && o.owner == p
        && crate::special_actions::marked(g, card, KeywordKind::Mayhem) == Some(g.turn.number)
}

fn mayhem(o: &GameObject) -> Option<&Keyword> {
    o.chars.keywords().find(|k| k.kind == KeywordKind::Mayhem)
}

pub struct Mayhem;

impl KeywordRules for Mayhem {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Mayhem]
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::Discarded { card, .. } = ev {
            if matches!(g.obj(*card).zone, Zone::Graveyard(_)) {
                crate::special_actions::mark(g, *card, KeywordKind::Mayhem);
            }
        }
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        if !discarded_this_turn(g, p, card) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Mayhem);
        if let Some(cost) = &kw.cost {
            opt.alt_cost = Some(super::modified_keyword_cost(
                g,
                p,
                KeywordKind::Mayhem,
                cost,
            ));
            opt.tag = Some(MAYHEM);
        }
        vec![opt]
    }

    fn playable_lands(&self, g: &Game, p: PlayerId) -> Vec<ObjectId> {
        g.player(p)
            .graveyard
            .iter()
            .copied()
            .filter(|c| {
                mayhem(g.obj(*c)).is_some_and(|k| k.cost.is_none())
                    && discarded_this_turn(g, p, *c)
            })
            .collect()
    }
}

inventory::submit! { KeywordRegistration(&Mayhem) }
