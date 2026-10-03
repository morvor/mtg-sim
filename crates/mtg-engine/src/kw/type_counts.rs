//! Values counting an object's types: "+1/+1 for each of its creature types" (Diligent
//! Zookeeper: changeling counts every creature type, CR 702.73a), "+1/+1 for each
//! supertype, card type, and subtype it has" (Embiggen; CR 205). "It" is the object a
//! static ability is being applied to, or else the first target of a resolving ability.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Characteristics;
use crate::types::*;

/// The number of creature types of "it".
pub const CREATURE_TYPES: &str = "types: creature types it has";
/// The number of supertypes, card types and subtypes of "it".
pub const ALL_TYPES: &str = "types: supertypes, card types and subtypes it has";

pub struct TypeCounts;

fn creature_types(c: &Characteristics) -> i64 {
    if c.all_creature_types {
        return crate::types::subtype_lists().creature.len() as i64;
    }
    c.subtypes.iter().filter(|s| is_creature_type(s)).count() as i64
}

impl KeywordRules for TypeCounts {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != CREATURE_TYPES && name != ALL_TYPES {
            return None;
        }
        let it = ctx.var_objects(vars::AFFECTED).first().copied().or_else(|| {
            ctx.targets
                .first()
                .and_then(|t| t.first())
                .and_then(|e| e.object())
        });
        let Some(o) = it else {
            return Some(0);
        };
        let c = &g.obj(o).chars;
        let ct = creature_types(c);
        Some(if name == CREATURE_TYPES {
            ct
        } else {
            let other = c.subtypes.iter().filter(|s| !is_creature_type(s)).count() as i64;
            c.supertypes.iter().count() as i64 + c.card_types.count() as i64 + ct + other
        })
    }
}

inventory::submit! { KeywordRegistration(&TypeCounts) }
