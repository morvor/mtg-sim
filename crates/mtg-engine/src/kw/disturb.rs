//! CR 702.146 Disturb: "You may cast this card transformed from your graveyard by paying
//! [cost] rather than its mana cost" (CR 702.146a). The spell has its back face up on the
//! stack (CR 712.8c), and a resolving disturb spell enters the battlefield with its back
//! face up (CR 702.146b). The back faces of disturb cards carry their own "If this would
//! be put into a graveyard from anywhere, exile it instead." (see
//! `oracle/patterns/control_exile_graveyard.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast with disturb.
pub const DISTURB: &str = "disturb";

pub struct Disturb;

impl KeywordRules for Disturb {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Disturb]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        // Only a double-faced card can be cast transformed.
        let Some(def) = g.obj(card).card.as_ref() else {
            return vec![];
        };
        if def.back().is_none() {
            return vec![];
        }
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Back);
        opt.method = CastMethod::Keyword(KeywordKind::Disturb);
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Disturb,
            &cost,
        ));
        opt.tag = Some(DISTURB);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Disturb) }
