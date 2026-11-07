//! What caused a life gain (CR 119.9, 614.1a): "If a spell or ability would cause its
//! controller to gain life, ..." (Rain of Gore). The life gain is caused by the spell or
//! ability resolving ([`Game::resolving`]) when that spell or ability's controller is the
//! player gaining life. Lifelink (a static ability's effect of damage, CR 702.15b) and
//! other life gain outside a resolution isn't.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Condition::Custom`, evaluated with the life gain event: the resolving spell or ability
/// is controlled by the player who would gain life.
pub const RESOLVING_CONTROLLER: &str = "life gain caused by a spell or ability of its controller";

struct LifeGainCause;

impl KeywordRules for LifeGainCause {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != RESOLVING_CONTROLLER {
            return None;
        }
        let player = ctx.event.as_ref().and_then(|e| e.player);
        Some(match (g.resolving, player) {
            (Some(r), Some(p)) => g.try_obj(r).is_some_and(|o| o.controller == p),
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&LifeGainCause) }
