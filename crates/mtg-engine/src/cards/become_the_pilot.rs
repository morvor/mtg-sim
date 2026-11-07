//! Become the Pilot: "Enchanted creature gets +2/+2 and can't be blocked unless it's
//! attacking its owner or a permanent its owner controls." (Its "You control enchanted
//! creature." compiles.)

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{Entity, ObjectId};

const EVASION: &str = "card:Become the Pilot:can't be blocked unless attacking its owner";
const TEXT: &str = "Enchanted creature gets +2/+2 and can't be blocked unless it's attacking its owner or a permanent its owner controls.";

inventory::submit! { ManualAbility {
    card: "Become the Pilot",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut v: Vec<Ability> = parse(ctx, "Enchanted creature gets +2/+2.")
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect();
        v.push(super::marker(EVASION, TEXT));
        v
    },
    reason: "can't be blocked unless attacking its owner or a permanent its owner controls: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn block_allowed(&self, g: &Game, _blocker: ObjectId, attacker: ObjectId) -> bool {
        let piloted = super::active(g, EVASION).into_iter().any(|(aura, _)| {
            g.obj(aura).zone == Zone::Battlefield
                && g.obj(aura).attached_to == Some(Entity::Object(attacker))
        });
        if !piloted {
            return true;
        }
        let owner = g.obj(attacker).owner;
        match g.combat.as_ref().and_then(|c| c.attack_target(attacker)) {
            Some(Entity::Player(p)) => p == owner,
            Some(Entity::Object(o)) => g.obj(o).controller == owner,
            None => true,
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
