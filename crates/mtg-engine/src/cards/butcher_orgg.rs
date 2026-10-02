//! Butcher Orgg: "You may assign ~'s combat damage divided as you choose among defending
//! player and/or any number of creatures they control." (an exception to CR 510.1b–c).

use super::{has_marker, marker, ManualAbility};
use crate::decision::{Answer, Decision};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const DIVIDE: &str = "card:Butcher Orgg:divide combat damage among defending player and their creatures";
const TEXT: &str = "You may assign ~'s combat damage divided as you choose among defending player and/or any number of creatures they control.";

inventory::submit! { ManualAbility {
    card: "Butcher Orgg",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(DIVIDE, TEXT)],
    reason: "divides its combat damage among defending player and their creatures (CR 510.1 exception): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Blocked or not (ruling); recipients assigned 0 aren't chosen.
    fn assign_combat_damage(
        &self,
        g: &mut Game,
        attacker: ObjectId,
        power: u32,
    ) -> Option<Vec<(ObjectId, Entity, u32)>> {
        if !has_marker(g, attacker, DIVIDE) {
            return None;
        }
        let dp = g
            .combat
            .as_ref()?
            .attackers
            .iter()
            .find(|a| a.id == attacker)?
            .defending_player?;
        let controller = g.obj(attacker).controller;
        if !g.ask_yes_no(
            controller,
            Some(attacker),
            "Divide its combat damage among defending player and their creatures?",
            false,
        ) {
            return None;
        }
        let mut recipients = vec![Entity::Player(dp)];
        recipients.extend(
            g.permanents_controlled_by(dp)
                .into_iter()
                .filter(|o| g.obj(*o).is_creature())
                .map(Entity::Object),
        );
        let n = recipients.len();
        let ans = g.ask(
            controller,
            Decision::AssignCombatDamage {
                creature: attacker,
                amount: power,
                recipients: recipients.clone(),
                lethal: vec![0; n],
                trample: false,
            },
        );
        let split: Vec<u32> = match ans {
            Answer::Numbers(v)
                if v.len() == n
                    && v.iter().all(|x| *x >= 0)
                    && v.iter().sum::<i64>() == power as i64 =>
            {
                v.into_iter().map(|x| x as u32).collect()
            }
            _ => {
                let mut d = vec![0; n];
                d[0] = power;
                d
            }
        };
        Some(
            recipients
                .into_iter()
                .zip(split)
                .filter(|(_, k)| *k > 0)
                .map(|(r, k)| (attacker, r, k))
                .collect(),
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
