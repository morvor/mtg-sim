//! Lifeline: "Whenever a creature dies, if another creature is on the battlefield, return
//! the first card to the battlefield under its owner's control at the beginning of the next
//! end step." ("The first card" is the errata wording for the creature card that died.)

use super::{parse, ManualAbility};
use crate::ability::AbilityDef;

const TEXT: &str = "Whenever a creature dies, if another creature is on the battlefield, return the first card to the battlefield under its owner's control at the beginning of the next end step.";

inventory::submit! { ManualAbility {
    card: "Lifeline",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever a creature dies, if another creature is on the battlefield, return that card to the battlefield under its owner's control at the beginning of the next end step.")
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect()
    },
    reason: "\"the first card\" (Lifeline's errata wording for the card that died): no other card says it",
} }
