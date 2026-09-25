//! Oracle patterns for the turn structure (CR 500–505):
//!
//! * additional phases and steps (CR 500.8–500.10a): "After this phase, there is an
//!   additional combat phase followed by an additional main phase", "there is an
//!   additional beginning phase after this phase", "you get an additional upkeep step
//!   after this step", "you get that many additional upkeep steps after this phase";
//! * extra turns (CR 500.7): "target player takes two extra turns after this one";
//! * skipping (CR 500.11, 614.10): "Skip your draw step.", "Players skip their upkeep
//!   steps.", "You skip your next turn.", "target player skips their next untap step";
//! * "at the beginning of your second main phase" (CR 505.1b: main phases are counted
//!   within the turn).

use super::{AbilityPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;
use crate::turn_structure::{AFTER_UPKEEP, MAIN_PHASE, UPKEEP};

// ---------------------------------------------------------------------------
// Additional phases and steps (CR 500.8–500.10)
// ---------------------------------------------------------------------------

/// "an additional combat phase", "two additional combat phases", "that many additional
/// upkeep steps" → (count, part).
fn additional(s: &str) -> Option<(Value, TurnPart)> {
    let s = s.trim();
    let (n, rest) = if let Some(r) = s.strip_prefix("an additional ") {
        (Value::c(1), r)
    } else if let Some(r) = s.strip_prefix("that many additional ") {
        (Value::EventAmount, r)
    } else {
        let (n, r) = parse_number(s)?;
        (n, r.trim_start().strip_prefix("additional ")?)
    };
    let part = match rest.trim() {
        "combat phase" | "combat phases" => TurnPart::CombatPhase,
        "main phase" | "main phases" => TurnPart::MainPhase,
        "beginning phase" | "beginning phases" => TurnPart::BeginningPhase,
        "untap step" | "untap steps" => TurnPart::Step(TriggerStep::Untap),
        "upkeep step" | "upkeep steps" => TurnPart::Step(TriggerStep::Upkeep),
        "draw step" | "draw steps" => TurnPart::Step(TriggerStep::Draw),
        "end step" | "end steps" => TurnPart::Step(TriggerStep::End),
        _ => return None,
    };
    Some((n, part))
}

/// "an additional combat phase followed by an additional main phase" → parts, count.
fn parts_phrase(s: &str) -> Option<(Value, Vec<TurnPart>)> {
    let (first, then) = match s.split_once(" followed by ") {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    let (n, part) = additional(first)?;
    let mut parts = vec![part];
    if let Some(t) = then {
        let (m, p2) = additional(t)?;
        if !matches!(m, Value::Const(1)) || !matches!(n, Value::Const(1)) {
            return None;
        }
        parts.push(p2);
    }
    Some((n, parts))
}

/// "after this phase" / "after this main phase" / "after this combat phase" → true;
/// "after this step" → false. "After this main phase" adds nothing unless it's a main
/// phase (and likewise for combat): returns that condition.
fn after_this(s: &str) -> Option<(bool, Option<Condition>)> {
    match s.trim() {
        "after this phase" => Some((true, None)),
        "after this main phase" => Some((true, Some(Condition::Phase(PhaseCond::MainPhase)))),
        "after this combat phase" => Some((true, Some(Condition::Phase(PhaseCond::Combat)))),
        "after this step" => Some((false, None)),
        _ => None,
    }
}

fn add_parts(
    parts: Vec<TurnPart>,
    (after_phase, cond): (bool, Option<Condition>),
    n: Value,
    who: Option<PlayerRef>,
) -> Effect {
    let e = Effect::AddTurnParts {
        parts,
        after_phase,
        n,
        who,
    };
    match cond {
        Some(cond) => Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
        None => e,
    }
}

fn additional_parts(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "After this phase, there is an additional combat phase ..."
    if let Some(i) = l.find(", there ") {
        let after = after_this(&l[..i])?;
        let rest = &l[i + ", there ".len()..];
        let rest = rest
            .strip_prefix("is ")
            .or_else(|| rest.strip_prefix("are "))?;
        let (n, parts) = parts_phrase(rest)?;
        return Some(add_parts(parts, after, n, None));
    }
    // "there is an additional beginning phase after this phase", "you get an additional
    // upkeep step after this step", "that player gets ...".
    let (who, rest) = if let Some(r) = l
        .strip_prefix("there is ")
        .or_else(|| l.strip_prefix("there are "))
    {
        (None, r)
    } else if let Some(i) = l.find(" get ").or_else(|| l.find(" gets ")) {
        let (p, r) = player_ref(&l[..i], b)?;
        if !r.trim().is_empty() {
            return None;
        }
        let r = &l[i..];
        let r = r
            .strip_prefix(" gets ")
            .or_else(|| r.strip_prefix(" get "))?;
        (Some(p), r)
    } else {
        return None;
    };
    let i = rest.find(" after this ")?;
    let after = after_this(&rest[i + 1..])?;
    let (n, parts) = parts_phrase(&rest[..i])?;
    Some(add_parts(parts, after, n, who))
}

inventory::submit! { EffectPattern { name: "r500 additional phases and steps", priority: 100, parse: additional_parts } }

// ---------------------------------------------------------------------------
// Extra turns (CR 500.7)
// ---------------------------------------------------------------------------

/// "[player] takes an extra turn after this one", "target player takes two extra turns
/// after this one".
fn extra_turns(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" take")?;
    let (who, r) = player_ref(&l[..i], b)?;
    if !r.trim().is_empty() {
        return None;
    }
    let rest = &l[i + " take".len()..];
    let rest = rest.strip_prefix("s ").or_else(|| rest.strip_prefix(" "))?;
    let (n, rest) = if let Some(r) = rest.strip_prefix("an ") {
        (1, r)
    } else {
        match parse_number(rest)? {
            (Value::Const(n), r) => (n, r),
            _ => return None,
        }
    };
    let rest = rest.trim();
    if rest != "extra turn after this one" && rest != "extra turns after this one" {
        return None;
    }
    // CR 500.7: multiple extra turns are added one at a time.
    Some(Effect::seq(
        (0..n.max(0))
            .map(|_| Effect::ExtraTurn { who: who.clone() })
            .collect(),
    ))
}

inventory::submit! { EffectPattern { name: "r500 extra turns", priority: 100, parse: extra_turns } }

// ---------------------------------------------------------------------------
// Skipping (CR 500.11, 614.10)
// ---------------------------------------------------------------------------

fn step_kind(s: &str) -> Option<StepKind> {
    Some(match s {
        "untap step" | "untap steps" => StepKind::Untap,
        "upkeep step" | "upkeep steps" | "upkeep" | "upkeeps" => StepKind::Upkeep,
        "draw step" | "draw steps" => StepKind::Draw,
        "combat phase" | "combat phases" => StepKind::Combat,
        "end step" | "end steps" => StepKind::End,
        "turn" | "turns" => StepKind::Turn,
        _ => return None,
    })
}

/// "You skip your next turn.", "target player skips their next untap step", "you skip
/// your next two turns".
fn skip_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" skip")?;
    let (who, r) = player_ref(&l[..i], b)?;
    if !r.trim().is_empty() {
        return None;
    }
    let rest = &l[i + " skip".len()..];
    let rest = rest.strip_prefix("s ").or_else(|| rest.strip_prefix(" "))?;
    let rest = rest
        .strip_prefix("their next ")
        .or_else(|| rest.strip_prefix("your next "))
        .or_else(|| rest.strip_prefix("his or her next "))?;
    let (n, rest) = match parse_number(rest) {
        Some((Value::Const(n), r)) if r.trim_start().starts_with(|c: char| c.is_alphabetic()) => {
            (n, r)
        }
        _ => (1, rest),
    };
    let step = step_kind(rest.trim())?;
    Some(Effect::seq(
        (0..n.max(0))
            .map(|_| Effect::Skip {
                who: who.clone(),
                step,
            })
            .collect(),
    ))
}

inventory::submit! { EffectPattern { name: "r500 skip next step or turn", priority: 100, parse: skip_next } }

/// "Skip your draw step.", "Players skip their upkeep steps.", "Each player skips their
/// untap step." — static effects that replace those steps with nothing (CR 614.1b).
fn skip_steps(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (whose, rest) = if let Some(r) = l
        .strip_prefix("skip your ")
        .or_else(|| l.strip_prefix("you skip your "))
    {
        (PlayerRel::You, r)
    } else if let Some(r) = l
        .strip_prefix("players skip their ")
        .or_else(|| l.strip_prefix("each player skips their "))
    {
        (PlayerRel::Any, r)
    } else if let Some(r) = l
        .strip_prefix("your opponents skip their ")
        .or_else(|| l.strip_prefix("each opponent skips their "))
    {
        (PlayerRel::Opponent, r)
    } else {
        return None;
    };
    let step = step_kind(rest)?;
    if matches!(step, StepKind::Turn | StepKind::Combat) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::SkipStep { step, whose },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r500 skip steps", priority: 100, parse: skip_steps } }

// ---------------------------------------------------------------------------
// "At the beginning of your second main phase" (CR 505.1b)
// ---------------------------------------------------------------------------

fn nth_main_phase(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let x = r.strip_prefix("at the beginning of ")?;
    let (whose, rest) = [
        ("each of your ", PlayerRel::You),
        ("your ", PlayerRel::You),
        ("each player's ", PlayerRel::Any),
        ("each opponent's ", PlayerRel::Opponent),
        ("each ", PlayerRel::Any),
    ]
    .into_iter()
    .find_map(|(p, w)| x.strip_prefix(p).map(|r| (w, r)))?;
    let n = match rest {
        "second main phase" | "second main phases" => 2,
        "third main phase" | "third main phases" => 3,
        _ => return None,
    };
    // Every main phase after the first is a postcombat main phase (CR 505.1a); which one
    // it is counts the main phases of this turn.
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::BeginningOf {
                step: TriggerStep::PostcombatMain,
                whose,
            }),
            cond: Condition::Custom(format!("{MAIN_PHASE}{n}").into()),
        },
        Sel::This,
        PlayerRef::ActivePlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "r505 nth main phase", priority: 50, parse: nth_main_phase } }

/// "At the beginning of enchanted player's first upkeep each turn" (a turn can have
/// several upkeep steps, CR 500.9, 503.2).
fn first_upkeep(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let x = r.strip_prefix("at the beginning of ")?;
    let x = x.strip_suffix(" first upkeep each turn")?;
    let first = Condition::Custom(format!("{UPKEEP}1").into());
    let (whose, cond) = match x {
        "your" => (PlayerRel::You, first),
        "each player's" => (PlayerRel::Any, first),
        "each opponent's" => (PlayerRel::Opponent, first),
        "enchanted player's" => (
            PlayerRel::Any,
            Condition::And(vec![
                Condition::PlayerMatches(
                    PlayerRef::ControllerOf(Box::new(Sel::AttachedTo)),
                    PlayerFilter::Active,
                ),
                first,
            ]),
        ),
        _ => return None,
    };
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose,
            }),
            cond,
        },
        Sel::This,
        PlayerRef::ActivePlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "r503 first upkeep each turn", priority: 50, parse: first_upkeep } }

/// "Cast this spell only during an opponent's turn after their upkeep step." (CR 503.2:
/// with several upkeep steps, that's any time after the first one ends.)
fn cast_after_upkeep(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let l = end(lower.trim());
    let r = l
        .strip_prefix("cast ~ only ")
        .or_else(|| l.strip_prefix("cast this spell only "))?;
    let whose = match r {
        "during an opponent's turn after their upkeep step" => Condition::NotYourTurn,
        "during your turn after your upkeep step" => Condition::YourTurn,
        _ => return None,
    };
    let cond = Condition::And(vec![whose, Condition::Custom(AFTER_UPKEEP.into())]);
    let mut s = StaticAbility::new(StaticEffect::CastOnlyIf(cond));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block)])
}

inventory::submit! { AbilityPattern { name: "r503 cast only after upkeep", priority: 100, parse: cast_after_upkeep } }

/// "Untap all creatures that attacked this turn." (the usual companion of an additional
/// combat phase).
fn untap_attackers(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "untap all creatures that attacked this turn" => Some(Effect::Untap {
            what: Sel::All(Filter::and(vec![
                Filter::creature(),
                Filter::AttackedThisTurn,
            ])),
        }),
        "untap all creatures you control that attacked this turn" => Some(Effect::Untap {
            what: Sel::All(Filter::and(vec![
                Filter::creature().you_control(),
                Filter::AttackedThisTurn,
            ])),
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r500 untap attackers", priority: 100, parse: untap_attackers } }
