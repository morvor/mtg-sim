//! Eminence (an ability word, CR 207.2c): triggered abilities of commanders written
//! "..., if [this] is in the command zone or on the battlefield, ...". An ability that
//! states which zones it functions in functions only from those zones (CR 113.6b): the
//! pattern in `oracle/patterns/eminence.rs` makes the triggered ability function from
//! anywhere, with the intervening "if" clause [`IN_COMMAND_ZONE_OR_ON_BATTLEFIELD`]
//! (CR 603.4) checked as it would trigger and as it resolves. An object that changed
//! zones is a new object (CR 400.7): if the source left the zone it was in, the ability
//! does nothing as it resolves.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;

/// `Condition::Custom`: the source (the same object, not a new one it became) is in the
/// command zone or on the battlefield.
pub const IN_COMMAND_ZONE_OR_ON_BATTLEFIELD: &str = "eminence:in the command zone or on the battlefield";

pub struct Eminence;

impl KeywordRules for Eminence {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != IN_COMMAND_ZONE_OR_ON_BATTLEFIELD {
            return None;
        }
        Some(ctx.source.is_some_and(|s| {
            g.is_live(s) && matches!(g.obj(s).zone, Zone::Command | Zone::Battlefield)
        }))
    }
}

inventory::submit! { KeywordRegistration(&Eminence) }
