//! "Activate only if you've controlled ~ continuously since the beginning of your most
//! recent turn." (Rocket Launcher): the condition CR 302.6 states for a creature's {T}
//! abilities, for any permanent — its controller has controlled it continuously since
//! their most recent turn began (it isn't "summoning sick").

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;

/// `Condition::Custom` name: the source's controller has controlled it continuously since
/// the beginning of their most recent turn.
pub const CONTROLLED_SINCE_YOUR_TURN: &str =
    "you've controlled ~ continuously since your most recent turn began";

pub struct ControlSinceYourTurn;

impl KeywordRules for ControlSinceYourTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CONTROLLED_SINCE_YOUR_TURN {
            return None;
        }
        Some(ctx.source.is_some_and(|s| {
            let o = g.obj(s);
            o.zone == Zone::Battlefield && o.controller == ctx.controller && !o.summoning_sick
        }))
    }
}

inventory::submit! { KeywordRegistration(&ControlSinceYourTurn) }
