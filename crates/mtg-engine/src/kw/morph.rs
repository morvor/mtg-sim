//! CR 702.37 Morph and megamorph: casting a card face down.
//!
//! "Morph [cost]" means "You may cast this card as a 2/2 face-down creature with no text,
//! no name, no subtypes, and no mana cost by paying {3} rather than paying its mana cost"
//! (CR 702.37a; megamorph likewise, CR 702.37b). The card is turned face down and put onto
//! the stack as a face-down spell; effects and prohibitions that apply to casting it look
//! at those face-down characteristics (CR 702.37c, see `Game::option_characteristics`).
//! It can be cast this way from any zone from which it could normally be cast. Turning it
//! face up is a special action: see `kw/morph_face_up.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::object::*;
use crate::types::*;

pub struct Morph;

/// The cost of casting a card face down with a morph ability: {3} (CR 702.37a).
pub fn face_down_cost() -> Cost {
    Cost::mana(ManaCost::generic(3))
}

impl KeywordRules for Morph {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        // CR 702.37d: only a morph ability lets a card be cast face down.
        if o.face_down || !o.chars.has_keyword(KeywordKind::Morph) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::FaceDown(KeywordKind::Morph);
        opt.alt_cost = Some(face_down_cost());
        // CR 702.37c: from any zone from which it could normally be cast, with the
        // permission judged by the face-down characteristics (CR 601.3e).
        let allowed = o.zone == Zone::Hand(p)
            || (g.permitted_cards(p).contains(&card)
                && g.permission_allows(p, card, &g.option_characteristics(card, &opt), false));
        if allowed {
            vec![opt]
        } else {
            vec![]
        }
    }
}

inventory::submit! { KeywordRegistration(&Morph) }
