//! Suppressor Skyguard: "Whenever a player attacks you, if that player has another opponent
//! who isn't being attacked, prevent all combat damage that would be dealt to you this
//! combat." Attacking that opponent's planeswalkers or battles doesn't count as attacking
//! them (ruling); the intervening "if" is checked again on resolution (CR 603.4).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const SPARED: &str = "card:Suppressor Skyguard:the attacking player has another opponent who isn't being attacked";
const TEXT: &str = "Whenever a player attacks you, if that player has another opponent who isn't being attacked, prevent all combat damage that would be dealt to you this combat.";

inventory::submit! { ManualAbility {
    card: "Suppressor Skyguard",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever a player attacks you, prevent all combat damage that would be dealt to you this combat.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.intervening_if = Some(Condition::Custom(SPARED.into()));
                    if let Effect::AddReplacement { duration, .. } = &mut t.body.effect {
                        *duration = Duration::EndOfCombat;
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "an intervening \"if\" on an opponent of the attacking player not being attacked: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != SPARED {
            return None;
        }
        // The player attacking you: the controller of a creature attacking you.
        let Some(combat) = g.combat.as_ref() else {
            return Some(false);
        };
        let Some(attacker) = combat
            .attackers
            .iter()
            .find(|a| a.target == Some(Entity::Player(ctx.controller)))
            .map(|a| g.obj(a.id).controller)
        else {
            return Some(false);
        };
        let attacked: Vec<Entity> = combat.attackers.iter().filter_map(|a| a.target).collect();
        Some(g.opponents(attacker).into_iter().any(|o| {
            o != ctx.controller && g.player(o).in_game() && !attacked.contains(&Entity::Player(o))
        }))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
