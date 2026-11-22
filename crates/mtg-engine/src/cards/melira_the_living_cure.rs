//! Melira, the Living Cure: "If you would get one or more poison counters, instead you get
//! one poison counter and you can't get additional poison counters this turn."
//! A replacement effect (CR 614.1a); once it has applied, further poison counters don't
//! happen whatever other replacement effects apply (ruling).

use super::ManualAbility;
use crate::ability::*;

const TEXT: &str = "If you would get one or more poison counters, instead you get one poison counter and you can't get additional poison counters this turn.";

inventory::submit! { ManualAbility {
    card: "Melira, the Living Cure",
    face: 0,
    text: TEXT,
    build: |_| {
        // The event is modified to one poison counter (rather than replaced by a new event
        // putting one, which replacement effects already applied to this event, such as
        // Vorinclex, Monstrous Raider's, would get another chance at: CR 614.5); the
        // restriction follows it.
        let action = ReplacementAction::Then(
            Box::new(ReplacementAction::Subtract(Value::Diff(
                Box::new(Value::EventAmount),
                Box::new(Value::Const(1)),
            ))),
            Box::new(Effect::AddPlayerEffect {
                who: PlayerRef::You,
                effect: PlayerModification::Custom(
                    crate::multiplayer::two_headed::CANT_GET_POISON.into(),
                ),
                duration: Duration::EndOfTurn,
            }),
        );
        vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::PutCounters {
                    on_objects: None,
                    on_players: Some(PlayerFilter::You),
                    kind: Some(crate::types::counters::POISON.into()),
                },
                action,
                self_replacement: false,
                optional: false,
            }))),
            TEXT,
        )]
    },
    reason: "poison counters become one counter, then none for the rest of the turn: only this card",
} }
