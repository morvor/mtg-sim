//! `Filter::Custom` "attached to a creature": an Aura or Equipment attached to a creature
//! permanent, whatever its enchant ability allows ("each Aura you control that's attached
//! to a creature", Sage's Reverie; CR 303.4).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

pub const ATTACHED_TO_A_CREATURE: &str = "attached to a creature";

pub struct AttachedToCreature;

impl KeywordRules for AttachedToCreature {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name != ATTACHED_TO_A_CREATURE {
            return None;
        }
        Some(match g.obj(id).attached_to {
            Some(Entity::Object(host)) => {
                let h = g.obj(host);
                g.is_live(host) && h.zone == Zone::Battlefield && h.is(CardType::Creature)
            }
            _ => false,
        })
    }
}

inventory::submit! { KeywordRegistration(&AttachedToCreature) }
