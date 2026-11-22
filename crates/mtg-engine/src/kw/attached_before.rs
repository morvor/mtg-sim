//! "Aura cards that were attached to it" (Cass, Hand of Vengeance): cards in a graveyard
//! whose last object on the battlefield was attached to the object a triggered ability is
//! about, as it last existed there (CR 608.2h). The text is parsed in
//! `oracle/patterns/attach_moves.rs`. Also: an object whose previous object is one of
//! some objects an instruction moved (`WAS_IN_VAR`).

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

/// `Filter::Custom` name prefix, followed by a variable number: an object whose previous
/// object (before its last zone change) is in that variable ("Exile enchanted creature and
/// all Auras attached to it. ... return that card": the card the creature became,
/// Flickerform).
pub const WAS_IN_VAR: &str = "an object whose previous object is in variable ";

/// The filter for an object whose previous object is in `var`.
pub fn was_in_var(var: crate::ability::Var) -> crate::ability::Filter {
    crate::ability::Filter::Custom(format!("{WAS_IN_VAR}{var}").into())
}

pub struct AttachedBefore;

impl KeywordRules for AttachedBefore {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if let Some(v) = name.strip_prefix(WAS_IN_VAR) {
            let v: crate::ability::Var = v.parse().ok()?;
            let prev = g.obj(id).prev;
            return Some(prev.is_some_and(|p| {
                ctx.vars
                    .get(&v)
                    .is_some_and(|es| es.contains(&Entity::Object(p)))
            }));
        }
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
