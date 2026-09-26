//! Timing restrictions on casting and activating: "Activate only during your turn,
//! before attackers are declared." (CR 506.8, 506.8g, 602.5b) and "Cast ~ only during the
//! declare attackers step and only if you've been attacked this step." (CR 601.3).

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::game::Game;
use crate::oracle::CompileContext;
use crate::types::Entity;

/// `Condition::Custom`: it's the declare blockers step (CR 509).
pub const DECLARE_BLOCKERS_STEP: &str = "restrictions:declare_blockers_step";
/// `Condition::Custom`: a creature was declared as an attacker attacking you during the
/// current declare attackers step ("you've been attacked this step", CR 508.1). Creatures
/// put onto the battlefield attacking never attacked (CR 508.4).
pub const ATTACKED_THIS_STEP: &str = "restrictions:you_were_attacked_this_step";

/// Evaluates this module's custom conditions.
pub fn custom_condition(g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<bool> {
    use crate::turn::Step;
    match name {
        DECLARE_BLOCKERS_STEP => Some(g.turn.step == Step::DeclareBlockers),
        ATTACKED_THIS_STEP => Some(
            g.turn.step == Step::DeclareAttackers
                && g.combat.as_ref().is_some_and(|c| {
                    c.declared_attackers
                        .iter()
                        .any(|(_, t)| *t == Entity::Player(ctx.controller))
                }),
        ),
        _ => None,
    }
}

/// Case-insensitive `strip_suffix` on the original text (keeps its capitalization).
fn strip_suffix_ci<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    let n = s.len().checked_sub(suffix.len())?;
    let tail = s.get(n..)?;
    tail.eq_ignore_ascii_case(suffix).then(|| &s[..n])
}

/// "[cost]: [effect]. Activate only during your turn, before attackers are declared.":
/// both restrictions apply (CR 602.5b); "before attackers are declared" refers to the
/// turn's first combat phase (CR 506.8a, 506.8d, 506.8g).
fn activate_your_turn_before_attackers(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let head = strip_suffix_ci(
        text,
        " activate only during your turn, before attackers are declared.",
    )?;
    let new_block = format!("{head} Activate only before attackers are declared.");
    let mut abilities = crate::oracle::parse_ability(&new_block, ctx)?;
    if abilities.len() != 1 {
        return None;
    }
    let a = abilities.pop()?;
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    let mut act = act.clone();
    if !matches!(act.timing, ActivationTiming::CombatWindow(_)) {
        return None;
    }
    act.condition = Some(match act.condition.take() {
        Some(c) => Condition::And(vec![Condition::YourTurn, c]),
        None => Condition::YourTurn,
    });
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "restrictions: activate only during your turn, before attackers", priority: 100, parse: activate_your_turn_before_attackers } }

/// The condition of "Cast ~ only [timing]" for the step-based timings (CR 601.3).
fn cast_timing(r: &str) -> Option<Condition> {
    let custom = |s: &str| Condition::Custom(s.into());
    let step = |p: PhaseCond| Condition::Phase(p);
    Some(match r {
        "during the declare attackers step" => step(PhaseCond::DeclareAttackers),
        "during your declare attackers step" => {
            Condition::And(vec![Condition::YourTurn, step(PhaseCond::DeclareAttackers)])
        }
        // Only a declared attacker "attacked" (CR 508.1, 508.4).
        "during the declare attackers step and only if you've been attacked this step" => {
            Condition::And(vec![
                step(PhaseCond::DeclareAttackers),
                custom(ATTACKED_THIS_STEP),
            ])
        }
        "during the declare blockers step" => custom(DECLARE_BLOCKERS_STEP),
        "during the declare blockers step on an opponent's turn" => {
            Condition::And(vec![Condition::NotYourTurn, custom(DECLARE_BLOCKERS_STEP)])
        }
        "during your end step" => {
            Condition::And(vec![Condition::YourTurn, step(PhaseCond::EndStep)])
        }
        "during an opponent's upkeep" => {
            Condition::And(vec![Condition::NotYourTurn, step(PhaseCond::Upkeep)])
        }
        // CR 506.8a, 506.8d: before the declare attackers step of the turn's first combat.
        "during an opponent's turn, before attackers are declared" => Condition::And(vec![
            Condition::NotYourTurn,
            Condition::CombatTiming(CombatTiming {
                point: CombatPoint::AttackersDeclared,
                after: false,
                during_combat: false,
            }),
        ]),
        _ => return None,
    })
}

fn cast_only(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l
        .strip_prefix("cast ~ only ")
        .or_else(|| l.strip_prefix("cast this spell only "))?;
    let cond = cast_timing(r)?;
    // CR 601.3: a restriction on casting the card itself, which functions wherever the
    // card could be cast from.
    let mut s = StaticAbility::new(StaticEffect::CastOnlyIf(cond));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// The same on instants and sorceries, whose lines aren't parsed as static abilities.
fn cast_only_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    cast_only(lower.strip_suffix('.')?, t, ctx)
}

inventory::submit! { StaticPattern { name: "restrictions: cast only during a step", priority: 100, parse: cast_only } }
inventory::submit! { AbilityPattern { name: "restrictions: cast only during a step", priority: 100, parse: cast_only_block } }
