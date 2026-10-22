//! CR 702.168 Disguise: casting a card face down.
//!
//! "Disguise [cost]" means "You may cast this card as a 2/2 face-down creature with ward
//! {2}, no name, no subtypes, and no mana cost by paying {3} rather than paying its mana
//! cost" (CR 702.168a); the face-down characteristics come from `facedown.rs` (CR 708.2).
//! Turning it face up for its disguise cost is a special action: see
//! `kw/morph_face_up.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::object::*;
use crate::types::*;

pub struct Disguise;

/// The cost of casting a card face down with a disguise ability: {3} (CR 702.168a).
pub fn face_down_cost() -> Cost {
    Cost::mana(ManaCost::generic(3))
}

impl KeywordRules for Disguise {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.face_down || !o.chars.has_keyword(KeywordKind::Disguise) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::FaceDown(KeywordKind::Disguise);
        opt.alt_cost = Some(face_down_cost());
        // From any zone it could normally be cast from, judged by the characteristics it
        // would have as a face-down spell (CR 601.3e, 708.4).
        let fd = crate::facedown::face_down_spell_characteristics(KeywordKind::Disguise);
        let allowed = o.zone == Zone::Hand(p)
            || (g.permitted_cards(p).contains(&card) && g.permission_allows(p, card, &fd, false));
        if allowed {
            vec![opt]
        } else {
            vec![]
        }
    }
}

inventory::submit! { KeywordRegistration(&Disguise) }
