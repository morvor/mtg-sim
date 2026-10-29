//! Deification: "As long as you control a creature, if damage dealt to a planeswalker you
//! control of the chosen type would result in all loyalty counters on it being removed,
//! instead all but one of those counters are removed."
//!
//! A replacement of one result of damage (CR 120.3c, 614.1), the
//! `loyalty_removed_by_damage` hook, keyed to the marker [`LAST_LOYALTY`] (with its
//! condition "as long as you control a creature", CR 604): the damage is still dealt with
//! all its other results (ruling). The card's [`super::ManualAbility`] registration waits
//! for its "As ~ enters, choose a planeswalker type." to compile (cards5-choices-players):
//! `manual-check` rejects a hand-written block on a card that still has unsupported text.

use super::active;
use crate::ability::Filter;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{counters, CardType, ObjectId};

/// The marker of the ability.
pub const LAST_LOYALTY: &str = "card:Deification:damage can't remove the last loyalty counter";

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn loyalty_removed_by_damage(&self, g: &Game, pw: ObjectId, n: u32) -> Option<u32> {
        let have = g.obj(pw).counter(counters::LOYALTY);
        if n == 0 || n < have {
            return None; // not all of them
        }
        let o = g.obj(pw);
        let applies = active(g, LAST_LOYALTY).into_iter().any(|(src, ctl)| {
            o.controller == ctl
                && o.is(CardType::Planeswalker)
                && g.matches(pw, &Filter::ChosenType, &Ctx::new(Some(src), ctl))
        });
        applies.then(|| have.saturating_sub(1))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
