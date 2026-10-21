//! Market Gnome: "When ~ is exiled from the battlefield while you're activating a craft
//! ability, you gain 1 life and draw a card." It triggers when it's exiled as a material
//! to pay a craft ability's cost (CR 702.167a); the ability goes on the stack above the
//! craft ability (ruling). (Being exiled while activating mana abilities during that
//! payment isn't modelled.)

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
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
    /// It was exiled to pay a cost, as one of the materials a craft ability exiled.
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CRAFTING {
            return None;
        }
        let Some(ev) = &ctx.event else {
            return Some(false);
        };
        let Some(exiled) = ev.object else {
            return Some(false);
        };
        Some(
            ev.cause == Some(MoveCause::Cost)
                && g.objects.iter().any(|o| {
                    o.controller == ctx.controller
                        && o.linked
                            .get(&crate::kw::craft::CRAFT_LINK)
                            .is_some_and(|v| v.contains(&exiled))
                }),
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
