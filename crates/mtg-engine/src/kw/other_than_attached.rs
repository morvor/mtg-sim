//! `Filter::Custom` "other than the attached object": what "other" means in "for each
//! other creature you control" on a noncreature Aura or Equipment (Bravado, Vampirism,
//! Sigil of Valor; see `oracle::patterns::other_than_attached`) — neither the source nor
//! the object it's attached to.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub const OTHER_THAN_ATTACHED: &str = "other than the attached object";

pub struct OtherThanAttached;

impl KeywordRules for OtherThanAttached {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != OTHER_THAN_ATTACHED {
            return None;
        }
        let Some(src) = ctx.source else {
            return Some(true);
        };
        let src = g.current(src);
        if src == id {
            return Some(false);
        }
        Some(match g.obj(src).attached_to {
            Some(Entity::Object(host)) => host != id && g.current(host) != id,
            _ => true,
        })
    }
}

inventory::submit! { KeywordRegistration(&OtherThanAttached) }
