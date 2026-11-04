//! River Song: "Meet in Reverse — You draw cards from the bottom of your library rather
//! than the top." Every card its controller draws while it's on the battlefield is the
//! bottom card of their library (CR 121.1; ruling: not just the draw for the turn). It's
//! still a draw, so draw replacements and draw triggers apply as usual.
//!
//! The rule is the `draws_from_bottom` hook, keyed to the marker [`FROM_BOTTOM`].

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::PlayerId;

/// The marker of "You draw cards from the bottom of your library rather than the top."
pub const FROM_BOTTOM: &str = "card:River Song:you draw from the bottom of your library";

const TEXT: &str =
    "Meet in Reverse — You draw cards from the bottom of your library rather than the top.";

inventory::submit! { ManualAbility {
    card: "River Song",
    face: 0,
    text: TEXT,
    build: |_ctx| vec![marker(FROM_BOTTOM, TEXT)],
    reason: "draws from the bottom of the library instead of the top: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn draws_from_bottom(&self, g: &Game, p: PlayerId) -> bool {
        active(g, FROM_BOTTOM).iter().any(|(_, ctl)| *ctl == p)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
