//! Cunning Giant: "If ~ is unblocked, you may have it assign its combat damage to a creature
//! defending player controls." (An exception to CR 510.1a.)

use super::{has_marker, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const REDIRECT: &str = "card:Cunning Giant:may assign its combat damage to a creature defending player controls";
const TEXT: &str = "If ~ is unblocked, you may have it assign its combat damage to a creature defending player controls.";

inventory::submit! { ManualAbility {
    card: "Cunning Giant",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(REDIRECT, TEXT)],
    reason: "an unblocked attacker may assign its combat damage to a creature defending player controls: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn assign_combat_damage(
        &self,
        g: &mut Game,
        attacker: ObjectId,
        power: u32,
    ) -> Option<Vec<(ObjectId, Entity, u32)>> {
        if !has_marker(g, attacker, REDIRECT) {
            return None;
        }
        let ai = g.combat.as_ref()?.attackers.iter().find(|a| a.id == attacker)?.clone();
        if ai.blocked {
            return None;
        }
        let dp = ai.defending_player?;
        let creatures: Vec<ObjectId> = g
            .permanents_controlled_by(dp)
            .into_iter()
            .filter(|o| g.obj(*o).is_creature())
            .collect();
        if creatures.is_empty() {
            return None;
        }
        let you = g.obj(attacker).controller;
        let prompt = "Assign its combat damage to a creature defending player controls?";
        if !g.ask_yes_no(you, Some(attacker), prompt, false) {
            return None;
        }
        let c = g
            .ask_objects(you, Some(attacker), "Choose the creature", creatures.clone(), 1, 1)
            .first()
            .copied()
            .unwrap_or(creatures[0]);
        Some(vec![(attacker, Entity::Object(c), power)])
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
