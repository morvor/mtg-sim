//! Soul Echo: "At the beginning of your upkeep, sacrifice ~ if there are no echo counters
//! on it. Otherwise, target opponent may choose that for each 1 damage that would be dealt
//! to you until your next upkeep, you remove an echo counter from ~ instead."
//!
//! The opponent's choice creates a replacement effect lasting until your next upkeep. It
//! keeps replacing damage even after the counters run out (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "At the beginning of your upkeep, sacrifice ~ if there are no echo counters on it. Otherwise, target opponent may choose that for each 1 damage that would be dealt to you until your next upkeep, you remove an echo counter from ~ instead.";

inventory::submit! { ManualAbility {
    card: "Soul Echo",
    face: 0,
    text: TEXT,
    build: |ctx| {
        // The trigger targets an opponent; its effect is the sacrifice-or-choice.
        let targets = parse(ctx, "At the beginning of your upkeep, target opponent may draw a card.");
        parse(ctx, "At the beginning of your upkeep, sacrifice ~ if there are no echo counters on it.")
            .iter()
            .zip(targets.iter())
            .map(|(a, t)| {
                let mut a: AbilityDef = (*map_effect(a, TEXT, |e| match e {
                    Effect::If { cond, then, .. } => Effect::If {
                        cond,
                        then,
                        otherwise: Box::new(Effect::May {
                            who: PlayerRef::Target(0),
                            effect: Box::new(Effect::AddReplacement {
                                def: ReplacementDef {
                                    event: ReplacementEvent::Damage {
                                        source: Filter::Any,
                                        to_players: Some(PlayerFilter::You),
                                        to_objects: None,
                                        combat_only: false,
                                    },
                                    action: ReplacementAction::Instead(Box::new(
                                        Effect::RemoveCounters {
                                            what: Sel::This,
                                            kind: Some("echo".into()),
                                            n: Value::EventAmount,
                                        },
                                    )),
                                    self_replacement: false,
                                    optional: false,
                                },
                                duration: Duration::UntilYourNextStep(TriggerStep::Upkeep),
                                uses: None,
                            }),
                        }),
                    },
                    other => other,
                }))
                .clone();
                if let (AbilityKind::Triggered(ta), AbilityKind::Triggered(tt)) = (&mut a.kind, &t.kind) {
                    ta.body.targets = tt.body.targets.clone();
                }
                std::sync::Arc::new(a)
            })
            .collect()
    },
    reason: "an opponent's choice to turn damage to you into echo counter removal: only this card",
} }
