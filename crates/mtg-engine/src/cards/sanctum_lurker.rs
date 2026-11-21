//! Sanctum Lurker: "Planeswalkers you control aren't put into their owners' graveyards for
//! having 0 loyalty." (An exception to the state-based action of CR 704.5i.)

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const KEEPS: &str = "card:Sanctum Lurker:planeswalkers you control survive 0 loyalty";
const TEXT: &str = "Planeswalkers you control aren't put into their owners' graveyards for having 0 loyalty.";

inventory::submit! { ManualAbility {
    card: "Sanctum Lurker",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(KEEPS, TEXT)],
    reason: "an exception to the 0-loyalty state-based action (CR 704.5i): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn keeps_zero_loyalty_planeswalker(&self, g: &Game, pw: ObjectId) -> bool {
        let ctl = g.obj(pw).controller;
        active(g, KEEPS).into_iter().any(|(_, you)| you == ctl)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
