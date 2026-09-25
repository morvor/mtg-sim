//! Replacement effects that modify how much life is gained or how many counters are put
//! (CR 614.1a):
//!
//! * "If you would gain life, you gain that much life plus 1 instead." / "... twice that
//!   much life instead." / "If a player would gain life, that player gains no life
//!   instead.";
//! * "If one or more +1/+1 counters would be put on a creature you control, that many
//!   plus one +1/+1 counters are put on it instead." / "... twice that many ..." / "that
//!   many -1/-1 counters minus one ..."; "If one or more counters would be put on [X],
//!   twice that many of each of those kinds of counters are put on it instead.";
//! * "If you would get one or more {E}, you get twice that many {E} instead.", "If you
//!   would get one or more counters, you get that many plus one of each of those kinds of
//!   counters instead."

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::counters;

fn replacement(event: ReplacementEvent, action: ReplacementAction, text: &str) -> Vec<Ability> {
    vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event,
                action,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )]
}

/// "that many plus one", "that many plus two", "twice that many", "three times that many".
fn amount_change(s: &str) -> Option<(ReplacementAction, &str)> {
    if let Some(r) = s.strip_prefix("twice that many") {
        return Some((ReplacementAction::Multiply(2), r));
    }
    if let Some(r) = s.strip_prefix("three times that many") {
        return Some((ReplacementAction::Multiply(3), r));
    }
    let r = s.strip_prefix("that many plus ")?;
    let (n, r) = parse_number(r)?;
    if matches!(n, Value::X) {
        return None;
    }
    Some((ReplacementAction::Add(n), r))
}

/// "If [you | a player | an opponent] would gain life, [you gain | that player gains]
/// [that much life plus N | twice that much life | no life] instead."
fn life_gain_replacement(l: &str) -> Option<(ReplacementEvent, ReplacementAction)> {
    let r = l.strip_prefix("if ")?;
    let (who, r) = if let Some(r) = r.strip_prefix("you would gain life, you gain ") {
        (PlayerFilter::You, r)
    } else if let Some(r) = r.strip_prefix("a player would gain life, that player gains ") {
        (PlayerFilter::Any, r)
    } else if let Some(r) = r.strip_prefix("an opponent would gain life, that player gains ") {
        (PlayerFilter::Opponent, r)
    } else {
        return None;
    };
    let r = r.strip_suffix(" instead")?;
    let action = match r {
        "no life" => ReplacementAction::Prevent,
        "twice that much life" => ReplacementAction::Multiply(2),
        "three times that much life" => ReplacementAction::Multiply(3),
        _ => {
            let n = r.strip_prefix("that much life plus ")?;
            let (n, tail) = parse_number(n)?;
            if !tail.trim().is_empty() || matches!(n, Value::X) {
                return None;
            }
            ReplacementAction::Add(n)
        }
    };
    Some((ReplacementEvent::GainLife(who), action))
}

/// The permanents counters would be put on: "a creature you control", "~", "an artifact
/// or creature you control", "another creature you control", "a creature".
fn counter_recipients(s: &str) -> Option<Filter> {
    if s == "~" {
        return Some(Filter::Source);
    }
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let (f, plural, tail) = parse_object_phrase(s)?;
    if plural || !tail.trim().is_empty() || f.zone().is_some() {
        return None;
    }
    Some(f)
}

/// "If one or more [kind] counters would be put on [permanent], [change] [kind] counters
/// are put on [it] instead", and the kind-less "of each of those kinds of counters" form.
fn counters_replacement(l: &str) -> Option<(ReplacementEvent, ReplacementAction)> {
    let r = l.strip_prefix("if one or more ")?;
    let (kind, r) = if let Some(r) = r.strip_prefix("counters would be put on ") {
        (None, r)
    } else {
        let (k, r) = counter_kind(r)?;
        (Some(k), r.strip_prefix("counters would be put on ")?)
    };
    let (who, action) = r.split_once(", ")?;
    let on = counter_recipients(who)?;
    let action = action.strip_suffix(" instead")?;
    // The pronoun at the end refers back to the permanent.
    let action = ["it", "that creature", "that permanent"]
        .iter()
        .find_map(|p| action.strip_suffix(&format!(" are put on {p}")))?;
    let act = match &kind {
        Some(k) => {
            // "that many -1/-1 counters minus one"
            if let Some(n) = action
                .strip_prefix(&format!("that many {k} counters minus "))
                .and_then(parse_number)
                .filter(|(n, t)| t.trim().is_empty() && !matches!(n, Value::X))
                .map(|(n, _)| n)
            {
                ReplacementAction::Subtract(n)
            } else {
                let (act, rest) = amount_change(action)?;
                if rest.trim() != format!("{k} counters") {
                    return None;
                }
                act
            }
        }
        None => {
            let (act, rest) = amount_change(action)?;
            if rest.trim() != "of each of those kinds of counters" {
                return None;
            }
            act
        }
    };
    Some((
        ReplacementEvent::PutCounters {
            on_objects: Some(on),
            on_players: None,
            kind,
        },
        act,
    ))
}

/// "If you would get one or more {E}, you get twice that many {E} instead." / "... that
/// many plus one {E} instead." / "If you would get one or more counters, you get that
/// many plus one of each of those kinds of counters instead."
fn player_counters_replacement(l: &str) -> Option<(ReplacementEvent, ReplacementAction)> {
    let r = l.strip_prefix("if you would get one or more ")?;
    let (kind, what, r): (Option<CounterKind>, &str, &str) =
        if let Some(r) = r.strip_prefix("{e}, you get ") {
            (Some(counters::ENERGY.into()), "{e}", r)
        } else if let Some(r) = r.strip_prefix("counters, you get ") {
            (None, "of each of those kinds of counters", r)
        } else {
            return None;
        };
    let r = r.strip_suffix(" instead")?;
    let (act, rest) = amount_change(r)?;
    if rest.trim() != what {
        return None;
    }
    Some((
        ReplacementEvent::PutCounters {
            on_objects: None,
            on_players: Some(PlayerFilter::You),
            kind,
        },
        act,
    ))
}

use crate::types::CounterKind;

fn amount_replacements(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    // Reminder-text removal can leave "{E} ," behind.
    let l = end(l).replace(" ,", ",");
    let (event, action) = life_gain_replacement(&l)
        .or_else(|| counters_replacement(&l))
        .or_else(|| player_counters_replacement(&l))?;
    Some(replacement(event, action, text))
}

inventory::submit! { StaticPattern { name: "counters_resources: life and counter amount replacements", priority: 100, parse: amount_replacements } }
