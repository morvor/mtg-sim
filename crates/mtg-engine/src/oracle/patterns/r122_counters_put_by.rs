//! Replacement effects keyed to the player who puts counters (CR 122.6, 122.6a) or who
//! creates tokens (CR 111.2), changing how many (CR 614.1a):
//!
//! * "If you would put one or more counters on a permanent or player, put twice that many
//!   of each of those kinds of counters on that permanent or player instead." (Vorinclex,
//!   Monstrous Raider; Innkeeper's Talent);
//! * "If an opponent would put one or more counters on a permanent or player, they put
//!   half that many of each of those kinds of counters on that permanent or player
//!   instead, rounded down." (Halving Season; Vorinclex, Monstrous Raider);
//! * "If an opponent would create one or more tokens, they create half that many of each
//!   of those kinds of tokens instead, rounded down." (Halving Season).
//!
//! A permanent's counters it enters with are put on it by its controller unless the
//! effect says otherwise (CR 122.6a, `counter_rules::who_puts_counters`).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

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

/// "twice that many", "three times that many", "half that many ... rounded down/up": the
/// change to the amount, given the phrase and whether it ends with ", rounded down" or
/// ", rounded up". The half is taken away from the event's amount.
fn amount(r: &str, rounded: Option<bool>) -> Option<(ReplacementAction, &str)> {
    if rounded.is_none() {
        if let Some(r) = r.strip_prefix("twice that many") {
            return Some((ReplacementAction::Multiply(2), r));
        }
        if let Some(r) = r.strip_prefix("three times that many") {
            return Some((ReplacementAction::Multiply(3), r));
        }
        return None;
    }
    let r = r.strip_prefix("half that many")?;
    // Rounded down, half remains: take away the other half, rounded up (and vice versa).
    let take = Value::Div(Box::new(Value::EventAmount), 2, rounded == Some(false));
    Some((ReplacementAction::Subtract(take), r))
}

/// Splits off a trailing ", rounded down" / ", rounded up".
fn rounding(s: &str) -> (&str, Option<bool>) {
    if let Some(r) = s.strip_suffix(", rounded down") {
        (r, Some(false))
    } else if let Some(r) = s.strip_suffix(", rounded up") {
        (r, Some(true))
    } else {
        (s, None)
    }
}

fn put_counters_by(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (l, rounded) = rounding(end(l));
    let r = l.strip_prefix("if ")?;
    let (by, r) = if let Some(r) = r.strip_prefix("you would put one or more ") {
        (PlayerRel::You, r)
    } else if let Some(r) = r.strip_prefix("an opponent would put one or more ") {
        (PlayerRel::Opponent, r)
    } else {
        return None;
    };
    let r = r.strip_prefix("counters on a permanent or player, ")?;
    let r = match by {
        PlayerRel::You => r,
        _ => r.strip_prefix("they ")?,
    };
    let r = r.strip_prefix("put ")?;
    let (action, r) = amount(r, rounded)?;
    if r != " of each of those kinds of counters on that permanent or player instead" {
        return None;
    }
    Some(replacement(
        ReplacementEvent::PutCountersBy { by, kind: None },
        action,
        text,
    ))
}

fn opponent_creates_half(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (l, rounded) = rounding(end(l));
    let r = l.strip_prefix(
        "if an opponent would create one or more tokens, they create ",
    )?;
    let (action, r) = amount(r, rounded)?;
    if rounded.is_none() || r != " of each of those kinds of tokens instead" {
        return None;
    }
    Some(replacement(
        ReplacementEvent::CreateTokens(PlayerFilter::Opponent),
        action,
        text,
    ))
}

inventory::submit! { StaticPattern { name: "if you/an opponent would put counters, twice/half that many", priority: 100, parse: put_counters_by } }
inventory::submit! { StaticPattern { name: "if an opponent would create tokens, half that many", priority: 100, parse: opponent_creates_half } }
