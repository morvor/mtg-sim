//! Defensive Formation: "Rather than the attacking player, you assign the combat damage of
//! each creature attacking you. You can divide that creature's combat damage as you choose
//! among any of the creatures blocking it." (as banding does, CR 702.22j; an exception to
//! CR 510.1c).

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId, PlayerId};

const ASSIGN: &str = "card:Defensive Formation:you assign the combat damage of creatures attacking you";
const TEXT: &str = "Rather than the attacking player, you assign the combat damage of each creature attacking you. You can divide that creature's combat damage as you choose among any of the creatures blocking it.";

inventory::submit! { ManualAbility {
    card: "Defensive Formation",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(ASSIGN, TEXT)],
    reason: "the defending player assigns attacking creatures' combat damage: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn combat_damage_assigner(&self, g: &Game, creature: ObjectId) -> Option<PlayerId> {
        let target = g
            .combat
            .as_ref()?
            .attackers
            .iter()
            .find(|a| a.id == creature)?
            .target?;
        let Entity::Player(p) = target else {
            return None;
        };
        active(g, ASSIGN).iter().any(|(_, c)| *c == p).then_some(p)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
