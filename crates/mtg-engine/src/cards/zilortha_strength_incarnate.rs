//! Zilortha, Strength Incarnate: "Lethal damage dealt to creatures you control is
//! determined by their power rather than their toughness." (CR 704.5g, 702.19b, 120.4a).

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const BY_POWER: &str = "card:Zilortha, Strength Incarnate:lethal damage by power";

inventory::submit! { ManualAbility {
    card: "Zilortha, Strength Incarnate",
    face: 0,
    text: "Lethal damage dealt to creatures you control is determined by their power rather than their toughness.",
    build: |_| vec![marker(
        BY_POWER,
        "Lethal damage dealt to creatures you control is determined by their power rather than their toughness.",
    )],
    reason: "lethal damage determined by power rather than toughness (CR 704.5g): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn lethal_damage_basis(&self, g: &Game, creature: ObjectId) -> Option<i32> {
        let o = g.obj(creature);
        active(g, BY_POWER)
            .iter()
            .any(|(_, ctl)| *ctl == o.controller)
            .then(|| o.power())
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
