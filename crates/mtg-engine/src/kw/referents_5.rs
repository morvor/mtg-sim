//! Engine rules behind `oracle/patterns/referents_values_5.rs` that need code: named
//! filters over what an earlier step did.
//!
//! * [`crate::kwa::manifest::PUT_IN_GRAVEYARD_BY_THAT_DREAD`]: a card the triggering
//!   manifest dread put into a graveyard (CR 701.62a), still there (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: one of the cards the latest instruction acted on ("it"/"those cards",
/// `vars::IT`) that has the same mana value as another of them ("For each of those cards
/// that has the same mana value as another card revealed this way", Vault 21: House
/// Gambit).
pub const SAME_MANA_VALUE_AS_ANOTHER_OF_IT: &str = "same mana value as another of those cards";

/// `Filter::Custom`: an activated or triggered ability on the stack other than the one
/// being resolved ("copy all other activated and triggered abilities you control", Ulalek,
/// Fused Atrocity).
pub const OTHER_ABILITY_ON_STACK: &str = "other activated or triggered ability on the stack";

pub struct Referents5;

impl KeywordRules for Referents5 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == crate::kwa::manifest::PUT_IN_GRAVEYARD_BY_THAT_DREAD {
            return Some(
                g.is_live(id)
                    && crate::kwa::manifest::dread_graveyard_cards(g, ctx).contains(&id),
            );
        }
        if name == OTHER_ABILITY_ON_STACK {
            let o = g.obj(id);
            return Some(
                g.is_live(id)
                    && o.zone == crate::object::Zone::Stack
                    && Some(id) != ctx.stack_obj
                    && o.stack.as_deref().is_some_and(|si| {
                        !matches!(si.kind, crate::object::StackKind::Spell)
                    }),
            );
        }
        if name == SAME_MANA_VALUE_AS_ANOTHER_OF_IT {
            let group: Vec<ObjectId> = ctx
                .vars
                .get(&crate::ability::vars::IT)
                .map(|v| v.iter().filter_map(|e| e.object()).collect())
                .unwrap_or_default();
            if !group.contains(&id) {
                return Some(false);
            }
            let mv = |o: ObjectId| g.mana_value_of(o);
            return Some(group.iter().any(|o| *o != id && mv(*o) == mv(id)));
        }
        None
    }
}

inventory::submit! { KeywordRegistration(&Referents5) }
