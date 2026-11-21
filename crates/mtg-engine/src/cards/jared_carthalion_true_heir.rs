//! Jared Carthalion, True Heir: "When ~ enters, target opponent becomes the monarch. You
//! can't become the monarch this turn." The second sentence is a player effect until end
//! of turn (the current monarch stays the monarch, ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "When ~ enters, target opponent becomes the monarch. You can't become the monarch this turn.";

inventory::submit! { ManualAbility {
    card: "Jared Carthalion, True Heir",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ enters, target opponent becomes the monarch.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| {
                    Effect::Seq(vec![
                        e,
                        Effect::AddPlayerEffect {
                            who: PlayerRef::You,
                            effect: PlayerModification::Custom(
                                crate::designations::CANT_BECOME_MONARCH.into(),
                            ),
                            duration: Duration::EndOfTurn,
                        },
                    ])
                })
            })
            .collect()
    },
    reason: "\"you can't become the monarch this turn\": a restriction on a designation; only this card in paper",
} }
