//! Coven (an ability word, CR 207.2c): "if you control three or more creatures with
//! different powers". Creatures have different powers from one another if each of their
//! powers is a different number, so the condition counts the distinct powers among the
//! creatures the player controls (`Value::Custom(DIFFERENT_POWERS)`; the condition phrase
//! is in `oracle/patterns/coven.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use std::collections::BTreeSet;

/// `Value::Custom` name: the greatest number of creatures the controller of the context
/// controls that all have different powers (the number of distinct powers among them).
pub const DIFFERENT_POWERS: &str = "coven:creatures you control with different powers";

/// The number of distinct powers among the creatures `p` controls.
pub fn different_powers(g: &Game, p: PlayerId) -> usize {
    g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Creature))
        .map(|o| o.power())
        .collect::<BTreeSet<_>>()
        .len()
}

pub struct Coven;

impl KeywordRules for Coven {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == DIFFERENT_POWERS).then(|| different_powers(g, ctx.controller) as i64)
    }
}

inventory::submit! { KeywordRegistration(&Coven) }
