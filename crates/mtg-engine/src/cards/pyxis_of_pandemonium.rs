//! Pyxis of Pandemonium: "{7}, {T}, Sacrifice ~: Each player turns face up all cards they
//! own exiled with ~, then puts all permanent cards among them onto the battlefield."
//!
//! The cards its first ability exiled face down are turned face up (CR 406.3) before the
//! permanent cards among them are put onto the battlefield under their owners' control.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "{7}, {T}, Sacrifice ~: Each player turns face up all cards they own exiled with ~, then puts all permanent cards among them onto the battlefield.";

inventory::submit! { ManualAbility {
    card: "Pyxis of Pandemonium",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{7}, {T}, Sacrifice ~: Put all permanent cards exiled with ~ onto the battlefield under their owners' control.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| {
                    Effect::Seq(vec![
                        Effect::TurnFaceUp {
                            what: Sel::All(Filter::And(vec![
                                Filter::Card,
                                Filter::In(Box::new(Sel::Linked)),
                                Filter::InZone(ZoneKind::Exile),
                            ])),
                        },
                        e,
                    ])
                })
            })
            .collect()
    },
    reason: "turning its face-down exiled cards face up before returning the permanents: only this card",
} }
