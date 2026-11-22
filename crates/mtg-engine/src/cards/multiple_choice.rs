//! Multiple Choice: "If X is 4 or more, do all of the above." — the effects of its first
//! three paragraphs (each of which compiles on its own), in order.

use super::{parse, spell, ManualAbility};
use crate::ability::*;

const TEXT: &str = "If X is 4 or more, do all of the above.";

/// The instructions of its other paragraphs, without their "If X is N," conditions.
const ABOVE: [&str; 3] = [
    "Scry 1, then draw a card.",
    "You may choose a player. They return a creature they control to its owner's hand.",
    "Create a 4/4 blue and red Elemental creature token.",
];

inventory::submit! { ManualAbility {
    card: "Multiple Choice",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut all = Vec::new();
        for t in ABOVE {
            for a in parse(ctx, t) {
                match &a.kind {
                    AbilityKind::Spell(s) => all.push(s.body.effect.clone()),
                    // Reported by manual-check through the card's unsupported text.
                    _ => return vec![a],
                }
            }
        }
        vec![spell(
            vec![],
            Effect::If {
                cond: Condition::Compare(Value::X, Cmp::Ge, Value::Const(4)),
                then: Box::new(Effect::Seq(all)),
                otherwise: Box::new(Effect::Noop),
            },
            TEXT,
        )]
    },
    reason: "\"do all of the above\": refers to the card's other paragraphs; only this card",
} }
