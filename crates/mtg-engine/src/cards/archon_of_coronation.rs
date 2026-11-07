//! Archon of Coronation: "As long as you're the monarch, damage doesn't cause you to lose
//! life." A replacement effect on the life loss that results from damage (CR 120.3a,
//! 614.1a): the damage is still dealt.

use super::ManualAbility;
use crate::ability::*;

const TEXT: &str = "As long as you're the monarch, damage doesn't cause you to lose life.";

inventory::submit! { ManualAbility {
    card: "Archon of Coronation",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::LifeLossFromDamage(PlayerFilter::You),
            action: ReplacementAction::Prevent,
            self_replacement: false,
            optional: false,
        }));
        s.condition = Some(
            crate::oracle::statics::parse_condition("you're the monarch", ctx)
                .unwrap_or(Condition::Custom("never".into())),
        );
        vec![AbilityDef::new(AbilityKind::Static(s), TEXT)]
    },
    reason: "while you're the monarch, damage doesn't cause you to lose life: unique",
} }
