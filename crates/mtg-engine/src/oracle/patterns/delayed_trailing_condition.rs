//! "[instruction] at the beginning of the next [step] if [condition]" (Angrath, the
//! Flame-Chained: "Sacrifice it at the beginning of the next end step if it has mana value
//! 3 or less."): the condition is the delayed instruction's, checked as the delayed
//! triggered ability resolves (CR 603.7, 608.2c; Angrath's ruling: "Whether the target
//! creature has mana value 3 or less is checked only as the delayed triggered ability ...
//! resolves during the end step"), not as the delayed ability is created.
//!
//! The same with the time first: "At the beginning of the next end step, exile those tokens
//! unless you have the city's blessing." (Tilonalli's Summoner: "Whether you have the city’s
//! blessing is checked only as the delayed triggered ability resolves during the end
//! step."), "At the beginning of the next end step, destroy that creature if it attacked
//! this turn." (Berserk).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn delayed_with_trailing_condition(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (x, c) = l.rsplit_once(" if ")?;
    if x.contains(" if ") || x.contains(" unless ") || c.contains(',') || c.is_empty() {
        return None;
    }
    // "... if ~ is still on the battlefield" (Shirei): the delayed grammar's own pattern.
    if c.contains('~') {
        return None;
    }
    let (instruction, delay) = x.split_once(" at the beginning of the next ")?;
    if instruction.is_empty() || instruction.contains(" at ") || delay.contains(" or ") {
        return None;
    }
    // The condition moves into the delayed instruction, which the delayed grammar reads
    // with the creating ability's references captured.
    super::delayed_grammar::delayed_instruction(
        &format!("{instruction} if {c} at the beginning of the next {delay}"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "[instruction] at the beginning of the next [step] if [condition]", priority: 240, parse: delayed_with_trailing_condition } }

/// "At the beginning of the next [step], [instruction] unless/if [condition]": the whole
/// sentence is the delayed instruction, condition included (the general "[effect]
/// unless/if [condition]" pattern would check it as the delayed ability is created).
fn delayed_first_with_trailing_condition(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("at the beginning of the next ")?;
    let (delay, instruction) = r.split_once(", ")?;
    let (x, c) = instruction
        .rsplit_once(" unless ")
        .or_else(|| instruction.rsplit_once(" if "))?;
    if delay.is_empty()
        || delay.contains(" or ")
        || x.is_empty()
        || x.contains(" if ")
        || x.contains(" unless ")
        || x.contains(',')
        || c.contains(',')
        || c.is_empty()
        || c.contains('~')
    {
        return None;
    }
    // "unless you pay ...": a payment, not a condition (the payment patterns' own).
    if [
        "you pay ",
        "they pay ",
        "that player pays ",
        "its controller pays ",
        "their controller pays ",
        "any player pays ",
    ]
    .iter()
    .any(|p| c.starts_with(p))
    {
        return None;
    }
    super::delayed_grammar::delayed_instruction(l, b)
}

inventory::submit! { EffectPattern { name: "at the beginning of the next [step], [instruction] if/unless [condition]", priority: 240, parse: delayed_first_with_trailing_condition } }
