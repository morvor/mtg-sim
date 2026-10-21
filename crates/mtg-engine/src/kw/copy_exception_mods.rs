//! Copy exceptions written as characteristics the copy has instead (CR 707.9b): they're
//! applied to the copiable values as part of the copy effect (layer 1), so copies of the
//! copy have them too (Jace, Mirror Mage's rulings).
//!
//! * "its starting loyalty is 1" (CR 306.5b): the loyalty a planeswalker enters with.
//! * "it's a legendary Alien named Prisoner Zero" (The Eleventh Hour): these subtypes
//!   instead of its other subtypes; its card types stay.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Layer, Modification};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Characteristics;
use crate::types::*;

/// `Modification::Custom` prefix: the starting loyalty is the number that follows.
pub const STARTING_LOYALTY: &str = "copy exception:starting loyalty:";

/// `Modification::Custom` prefix: its subtypes are only those listed (JSON) afterwards.
pub const ONLY_SUBTYPES: &str = "copy exception:only subtypes:";

/// The modification giving a copy a starting loyalty.
pub fn starting_loyalty(n: i32) -> Modification {
    Modification::Custom {
        name: format!("{STARTING_LOYALTY}{n}").into(),
        layer: Layer::L1aCopy,
    }
}

/// The modification leaving a copy only these subtypes.
pub fn only_subtypes(subtypes: &[Subtype]) -> Option<Modification> {
    Some(Modification::Custom {
        name: format!("{ONLY_SUBTYPES}{}", serde_json::to_string(subtypes).ok()?).into(),
        layer: Layer::L1aCopy,
    })
}

struct CopyExceptionMods;

impl KeywordRules for CopyExceptionMods {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_modification(
        &self,
        _g: &Game,
        name: &str,
        chars: &mut Characteristics,
        _ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        if let Some(n) = name.strip_prefix(STARTING_LOYALTY) {
            if let Ok(n) = n.parse::<i32>() {
                chars.loyalty = Some(n);
            }
            return true;
        }
        if let Some(json) = name.strip_prefix(ONLY_SUBTYPES) {
            if let Ok(v) = serde_json::from_str::<Vec<Subtype>>(json) {
                chars.subtypes = v.into_iter().collect();
                chars.all_creature_types = false;
            }
            return true;
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&CopyExceptionMods) }
