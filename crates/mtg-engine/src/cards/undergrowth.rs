//! Undergrowth: "Prevent all combat damage that would be dealt this turn. If ~'s additional
//! cost was paid, this effect doesn't affect combat damage that would be dealt by red
//! creatures." (Its optional additional cost of {2}{R} compiles.)

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::types::{CardType, Color};

const TEXT: &str = "Prevent all combat damage that would be dealt this turn. If ~'s additional cost was paid, this effect doesn't affect combat damage that would be dealt by red creatures.";

/// The prevention shield, except for damage red creatures would deal.
fn except_red(e: &Effect) -> Effect {
    match e {
        Effect::AddReplacement {
            def,
            duration,
            uses,
        } => {
            let mut def = def.clone();
            if let ReplacementEvent::Damage { source, .. } = &mut def.event {
                *source = Filter::And(vec![
                    source.clone(),
                    Filter::Not(Box::new(Filter::And(vec![
                        Filter::Type(CardType::Creature),
                        Filter::Color(Color::Red),
                    ]))),
                ]);
            }
            Effect::AddReplacement {
                def,
                duration: duration.clone(),
                uses: uses.clone(),
            }
        }
        e => e.clone(),
    }
}

inventory::submit! { ManualAbility {
    card: "Undergrowth",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Prevent all combat damage that would be dealt this turn.")
            .iter()
            .map(|a| {
                super::map_effect(a, TEXT, |e| Effect::If {
                    cond: Condition::Custom("this spell's additional cost was paid".into()),
                    then: Box::new(except_red(&e)),
                    otherwise: Box::new(e),
                })
            })
            .collect()
    },
    reason: "a combat damage fog that, if its additional cost was paid, doesn't apply to red creatures' damage: only this card",
} }
