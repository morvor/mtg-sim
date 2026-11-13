//! Trinisphere: "As long as ~ is untapped, each spell that would cost less than three
//! mana to cast costs three mana to cast." A minimum total cost, applied after every other
//! cost increase and reduction (CR 601.2f).

use super::{active, marker_if, ManualAbility};
use crate::ability::{Condition, Cost};
use crate::game::Game;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::types::{ObjectId, PlayerId};

const FLOOR: &str = "card:Trinisphere:spells cost at least three mana";

inventory::submit! { ManualAbility {
    card: "Trinisphere",
    face: 0,
    text: "As long as ~ is untapped, each spell that would cost less than three mana to cast costs three mana to cast.",
    build: |ctx| {
        let cond = crate::oracle::statics::parse_condition("~ is untapped", ctx)
            .unwrap_or(Condition::Custom("never".into()));
        vec![marker_if(
            FLOOR,
            cond,
            "As long as ~ is untapped, each spell that would cost less than three mana to cast costs three mana to cast.",
        )]
    },
    reason: "a minimum total cost of three mana for every spell (CR 601.2f, after reductions): unique cost rule",
} }

struct TrinisphereRules;

impl KeywordRules for TrinisphereRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// The mana in the total cost (X counted as chosen) is raised to three with generic
    /// mana; several Trinispheres don't add up.
    fn global_spell_cost(&self, g: &Game, _p: PlayerId, _card: ObjectId, cost: &mut Cost) {
        if active(g, FLOOR).is_empty() {
            return;
        }
        let have = cost.mana.as_ref().map_or(0, |m| m.mana_value());
        if have < 3 {
            cost.mana
                .get_or_insert_with(ManaCost::default)
                .add(&ManaCost::generic(3 - have));
        }
    }
}

inventory::submit! { KeywordRegistration(&TrinisphereRules) }
