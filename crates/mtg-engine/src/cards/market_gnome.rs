//! Market Gnome: "When ~ is exiled from the battlefield while you're activating a craft
//! ability, you gain 1 life and draw a card." It triggers when it's exiled as a material
//! to pay a craft ability's cost (CR 702.167a); the ability goes on the stack above the
//! craft ability (ruling), or while a mana ability activated during that payment exiles
//! it (the game records which abilities are being activated).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const CRAFTING: &str = "card:Market Gnome:exiled while you're activating a craft ability";
const TEXT: &str = "When ~ is exiled from the battlefield while you're activating a craft ability, you gain 1 life and draw a card.";

inventory::submit! { ManualAbility {
    card: "Market Gnome",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ is exiled from the battlefield, you gain 1 life and draw a card.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.trigger = TriggerCond::Where {
                        trigger: Box::new(t.trigger.clone()),
                        cond: Condition::Custom(CRAFTING.into()),
                    };
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "triggers on being exiled while its controller activates a craft ability: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// It was exiled while its controller was activating a craft ability: as a material
    /// paying the craft cost, or by a mana ability activated during that payment.
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CRAFTING {
            return None;
        }
        Some(g.special.activating.iter().any(|(p, src, uid)| {
            *p == ctx.controller
                && g.obj(*src)
                    .chars
                    .abilities
                    .iter()
                    .any(|a| a.uid == *uid && crate::kw::craft::is_craft_ability(a))
        }))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
