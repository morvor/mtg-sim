//! Phyrexian Splicer: "{2}, {T}, Choose flying, first strike, trample, or shadow: Until end
//! of turn, target creature with the chosen ability loses it and another target creature
//! gains it." The keyword is chosen as the ability is activated, before targets: modelled as
//! four modes. The creature gains it even if the first target lost it (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "{2}, {T}, Choose flying, first strike, trample, or shadow: Until end of turn, target creature with the chosen ability loses it and another target creature gains it.";
const KEYWORDS: [&str; 4] = ["flying", "first strike", "trample", "shadow"];

inventory::submit! { ManualAbility {
    card: "Phyrexian Splicer",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut cost = None;
        let mut modes = Vec::new();
        for k in KEYWORDS {
            let text = format!("{{2}}, {{T}}: Until end of turn, target creature with {k} loses {k} and another target creature gains {k}.");
            let parsed = parse(ctx, &text);
            let Some(AbilityKind::Activated(act)) = parsed.first().map(|a| a.kind.clone()) else {
                return parsed;
            };
            cost.get_or_insert(act.cost.clone());
            modes.push(Mode {
                text: format!("Choose {k}."),
                targets: act.body.targets.clone(),
                effect: act.body.effect.clone(),
                cost: None,
            });
        }
        let body = Body {
            targets: vec![],
            effect: Effect::Noop,
            modal: Some(Modal {
                min: Value::Const(1),
                max: Value::Const(1),
                allow_repeat: false,
                modes,
                per_mode_cost: false,
                chooser: ModeChooser::Controller,
                different_players: false,
                optional: false,
            }),
        };
        vec![AbilityDef::new(
            AbilityKind::Activated(ActivatedAbility::new(cost.unwrap_or_default(), body)),
            TEXT,
        )]
    },
    reason: "a keyword chosen as part of the activation cost, moved from one target to another: only this card",
} }
