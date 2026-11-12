//! "[instruction] at the beginning of the next [step] if [condition]" (Angrath, the
//! Flame-Chained: "Sacrifice it at the beginning of the next end step if it has mana value
//! 3 or less."): the condition is the delayed instruction's, checked as the delayed
//! triggered ability resolves (CR 603.7, 608.2c; Angrath's ruling: "Whether the target
//! creature has mana value 3 or less is checked only as the delayed triggered ability ...
//! resolves during the end step"), not as the delayed ability is created.

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
