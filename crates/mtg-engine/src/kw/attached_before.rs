//! "Aura cards that were attached to it" (Cass, Hand of Vengeance): cards in a graveyard
//! whose last object on the battlefield was attached to the object a triggered ability is
//! about, as it last existed there (CR 608.2h). The text is parsed in
//! `oracle/patterns/attach_moves.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Sel;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Filter::Custom`: a card whose previous object (on the battlefield) was attached to the
/// triggering object.
pub const WAS_ATTACHED_TO_TRIGGER_OBJECT: &str = "a card that was attached to the triggering object";

pub struct AttachedBefore;

impl KeywordRules for AttachedBefore {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != WAS_ATTACHED_TO_TRIGGER_OBJECT {
            return None;
        }
        let Some(prev) = g.obj(id).prev else {
            return Some(false);
        };
        let p = g.obj(prev);
        if p.zone != Zone::Battlefield {
            return Some(false);
        }
        let mut hosts = g.eval_sel(&Sel::TriggerObject, ctx);
        hosts.extend(g.eval_sel(&Sel::TriggerLki, ctx));
        Some(
            [p.attached_to, p.last_attached_to]
                .into_iter()
                .flatten()
                .any(|h| hosts.contains(&h)),
        )
    }
}

inventory::submit! { KeywordRegistration(&AttachedBefore) }
