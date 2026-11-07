//! Auras and Equipment attached to what "you" control, for the phrases "Curses attached to
//! you" (Witchbane Orb) and "an Equipment named [name] is attached to a creature you
//! control" (Bride's Gown, Groom's Finery). "You" is the controller of the ability that
//! refers to them, not the controller of the object they're attached to.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Filter::Custom`: attached to the player who controls the ability being evaluated.
pub const ATTACHED_TO_YOU: &str = "attached to you";

/// `Filter::Custom`: attached to a creature controlled by the controller of the ability
/// being evaluated.
pub const ATTACHED_TO_CREATURE_YOU_CONTROL: &str = "attached to a creature you control";

pub struct AttachedToYours;

impl KeywordRules for AttachedToYours {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let attached = g.obj(id).attached_to;
        match name {
            ATTACHED_TO_YOU => Some(attached == Some(Entity::Player(ctx.controller))),
            ATTACHED_TO_CREATURE_YOU_CONTROL => Some(match attached {
                Some(Entity::Object(host)) => {
                    let h = g.obj(host);
                    h.zone == Zone::Battlefield
                        && h.controller == ctx.controller
                        && h.is(CardType::Creature)
                }
                _ => false,
            }),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&AttachedToYours) }
