//! Quakebringer: "At the beginning of your upkeep, ~ deals 2 damage to each opponent. This
//! ability triggers only if ~ is on the battlefield or if ~ is in your graveyard and you
//! control a Giant." The ability functions on the battlefield and, while you control a
//! Giant, in your graveyard (CR 113.6, 603.3): there it triggers only if you control a
//! Giant as your upkeep begins (not an intervening "if" checked again).

use super::{parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "At the beginning of your upkeep, ~ deals 2 damage to each opponent. This ability triggers only if ~ is on the battlefield or if ~ is in your graveyard and you control a Giant.";

inventory::submit! { ManualAbility {
    card: "Quakebringer",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let base = parse(ctx, "At the beginning of your upkeep, ~ deals 2 damage to each opponent.");
        let giant = parse(ctx, "At the beginning of your upkeep, if you control a Giant, ~ deals 2 damage to each opponent.");
        let mut out: Vec<Ability> = base
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect();
        for a in giant {
            let mut kind = a.kind.clone();
            if let AbilityKind::Triggered(t) = &mut kind {
                if let Some(cond) = t.intervening_if.take() {
                    t.trigger = TriggerCond::Where {
                        trigger: Box::new(t.trigger.clone()),
                        cond,
                    };
                }
                t.zone = FunctionZone::Graveyard;
            }
            out.push(AbilityDef::new(kind, TEXT));
        }
        out
    },
    reason: "a triggered ability that functions on the battlefield or, while you control a Giant, in your graveyard: only this card",
} }
