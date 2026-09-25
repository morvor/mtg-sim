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
fn delayed_timing(l: &str) -> Option<(TriggerCond, String)> {
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
            return Some((TriggerCond::BeginningOf { step, whose }, r.to_string()));
        }
        // The trigger condition may also be written after the effect (CR 603.11):
        // "scry 3 at the beginning of your first upkeep."
        let suffix = format!(" {}.", prefix.trim_end_matches(", "));
        if let Some(r) = l.strip_suffix(&suffix) {
            return Some((TriggerCond::BeginningOf { step, whose }, format!("{r}.")));
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
            let effect = parse_effect_text(&effect_text, &mut b)?;
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

/// "Before you shuffle your deck to start the game, you may reveal this card from your deck
/// and exile [a card] you drafted that isn't in your deck." (CR 607.2n)
fn before_shuffle(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let mut r = None;
    for this in ["this card", "~"] {
        let prefix = format!(
            "before you shuffle your deck to start the game, you may reveal {this} from your deck and exile "
        );
        if let Some(x) = lower.strip_prefix(&prefix) {
            r = Some(x);
        }
    }
    let r = r?.strip_suffix(" you drafted that isn't in your deck.")?;
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))
        .unwrap_or(r);
    let (what, _, tail) = crate::oracle::phrases::parse_object_phrase(r)?;
    if !crate::oracle::phrases::end(tail).is_empty() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::BeforeShuffleExile { what });
    s.zone = FunctionZone::Library;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
}

inventory::submit! { AbilityPattern { name: "before you shuffle your deck", priority: 0, parse: before_shuffle } }
