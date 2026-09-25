//! Oracle patterns for opening-hand actions (CR 103.6): "If this card is in your opening
//! hand, you may begin the game with it on the battlefield." and "You may reveal this card
//! from your opening hand. If you do, [delayed triggered ability]." (CR 603.7g).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::CompileContext;

fn opening_hand_ability(effect: StaticEffect, text: &str) -> Vec<Ability> {
    let mut s = StaticAbility::new(effect);
    s.zone = FunctionZone::Hand;
    vec![AbilityDef::new(AbilityKind::Static(s), text)]
}

/// The delayed trigger's timing: "at the beginning of the first upkeep, ...".
fn delayed_timing(l: &str) -> Option<(TriggerCond, &str)> {
    let table: [(&str, TriggerStep, PlayerRel); 5] = [
        (
            "at the beginning of the first upkeep, ",
            TriggerStep::Upkeep,
            PlayerRel::Any,
        ),
        (
            "at the beginning of your first upkeep, ",
            TriggerStep::Upkeep,
            PlayerRel::You,
        ),
        (
            "at the beginning of your first main phase of the game, ",
            TriggerStep::PrecombatMain,
            PlayerRel::You,
        ),
        (
            "at the beginning of your first main phase, ",
            TriggerStep::PrecombatMain,
            PlayerRel::You,
        ),
        (
            "at the beginning of the first end step, ",
            TriggerStep::End,
            PlayerRel::Any,
        ),
    ];
    for (prefix, step, whose) in table {
        if let Some(r) = l.strip_prefix(prefix) {
            return Some((TriggerCond::BeginningOf { step, whose }, r));
        }
    }
    None
}

fn opening_hand(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    for this in ["this card", "~"] {
        if lower == format!("if {this} is in your opening hand, you may begin the game with it on the battlefield.") {
            return Some(opening_hand_ability(
                StaticEffect::OpeningHand { delayed: None },
                t,
            ));
        }
        let prefix = format!("you may reveal {this} from your opening hand. if you do, ");
        if let Some(r) = lower.strip_prefix(&prefix) {
            let (trigger, effect_text) = delayed_timing(r)?;
            let mut b = Builder::new(ctx);
            b.in_trigger = true;
            let effect = parse_effect_text(effect_text, &mut b)?;
            let body = Body {
                targets: b.targets,
                effect,
                modal: None,
            };
            return Some(opening_hand_ability(
                StaticEffect::OpeningHand {
                    delayed: Some(Box::new((trigger, body))),
                },
                t,
            ));
        }
    }
    None
}

inventory::submit! { AbilityPattern { name: "opening hand actions", priority: 0, parse: opening_hand } }
