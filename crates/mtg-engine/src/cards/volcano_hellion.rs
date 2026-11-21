//! Volcano Hellion: "When ~ enters, it deals an amount of damage of your choice to you and
//! target creature. The damage can't be prevented." The number is chosen as the ability
//! resolves; 0 may be chosen (ruling). Its echo {X} compiles.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "When ~ enters, it deals an amount of damage of your choice to you and target creature. The damage can't be prevented.";

inventory::submit! { ManualAbility {
    card: "Volcano Hellion",
    face: 0,
    text: TEXT,
    build: |ctx| {
        // The trigger and target as for a fixed amount; then the amount is chosen and the
        // damage is dealt to both at once, as unpreventable damage is for other cards
        // ("~ deals 4 damage to target creature. The damage can't be prevented.").
        parse(ctx, "When ~ enters, ~ deals 1 damage to target creature.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::DealDamage { source, .. } => Effect::Seq(vec![
                        Effect::Choose {
                            who: PlayerRef::You,
                            kind: ChoiceKind::Number { min: 0, max: 1000 },
                        },
                        Effect::AddRestriction {
                            restriction: Restriction::SourceDamageCantBePrevented(Filter::In(
                                Box::new(Sel::This),
                            )),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::DealDamage {
                            source,
                            amount: Value::Chosen,
                            to: Sel::Union(vec![Sel::Players(PlayerRef::You), Sel::Target(0)]),
                        },
                    ]),
                    e => e,
                })
            })
            .collect()
    },
    reason: "\"an amount of damage of your choice\", chosen as it resolves, to you and the target: only this card",
} }
