//! Loot, the Anomaly: "If ~'s power is negative, he assigns combat damage as though his
//! power were positive." (CR 510.1a: a creature assigns combat damage equal to its power).

use super::{has_marker, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const ABS: &str = "card:Loot, the Anomaly:assigns combat damage as though negative power were positive";

inventory::submit! { ManualAbility {
    card: "Loot, the Anomaly",
    face: 0,
    text: "If ~'s power is negative, he assigns combat damage as though his power were positive.",
    build: |_| vec![marker(
        ABS,
        "If ~'s power is negative, he assigns combat damage as though his power were positive.",
    )],
    reason: "assigns combat damage as though negative power were positive (CR 510.1a): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn combat_damage_amount(&self, g: &Game, creature: ObjectId) -> Option<u32> {
        let p = g.obj(creature).power();
        (p < 0 && has_marker(g, creature, ABS)).then(|| p.unsigned_abs())
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
