//! Ancient Adamantoise: "Damage isn't removed from ~ during cleanup steps." (an exception
//! to CR 514.2).

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const KEEPS: &str = "card:Ancient Adamantoise:damage isn't removed during cleanup";

inventory::submit! { ManualAbility {
    card: "Ancient Adamantoise",
    face: 0,
    text: "Damage isn't removed from ~ during cleanup steps.",
    build: |_| vec![marker(KEEPS, "Damage isn't removed from ~ during cleanup steps.")],
    reason: "damage isn't removed from it during cleanup (an exception to CR 514.2): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Only while the ability functions: a phased-out Adamantoise loses its damage.
    fn keeps_damage_in_cleanup(&self, g: &Game, id: ObjectId) -> bool {
        active(g, KEEPS).iter().any(|(s, _)| *s == id)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
