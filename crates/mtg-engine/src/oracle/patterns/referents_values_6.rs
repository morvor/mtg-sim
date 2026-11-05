//! Referent and value grammar VI (CR 107.3, 608.2h, 611.2a).
//!
//! * "count the number of permanents. If the number is odd, ... If the number is even,
//!   ..." (Chaos Moon): "the number" is the number counted once, as the instruction was
//!   carried out (CR 608.2h), and each "if" checks its parity.
//! * "until end of turn, red creatures get +1/+1 and whenever a player taps a Mountain for
//!   mana, that player adds an additional {R}": two continuous effects (a pump and a
//!   delayed mana trigger, or a mana replacement) under one duration (CR 611.2a).
//! * "if a player taps a Mountain for mana, that Mountain produces colorless mana instead
//!   of any other type": "that Mountain" is the tapped land, as "it" is (CR 106.12a).

use super::basic_effects_misc::COUNTED;
use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;

/// "If the number is odd, ..." after "count the number of [objects]": the counted number
/// (in place of "the number") in a condition about it; the rest of the sentence is parsed
/// as written.
fn if_the_number(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let counted = match &*prev {
        Effect::Seq(v) => v
            .iter()
            .any(|e| matches!(e, Effect::StoreValue { var, .. } if *var == COUNTED)),
        Effect::StoreValue { var, .. } => *var == COUNTED,
        _ => false,
    };
    if !counted {
        return false;
    }
    let lower = end(l).to_lowercase();
    let Some(r) = lower
        .strip_prefix("if the number is ")
        .or_else(|| lower.strip_prefix("if that number is "))
    else {
        return false;
    };
    // The body must not have an X of its own.
    if r.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") || r.contains("the number") {
        return false;
    }
    let text = format!("if x is {r}");
    let first_new = b.targets.len();
    let Some(e) = parse_sentence(&text, b) else {
        b.targets.truncate(first_new);
        return false;
    };
    if b.targets.len() != first_new || !matches!(e, Effect::If { .. }) {
        b.targets.truncate(first_new);
        return false;
    }
    let Some(e) = super::r107_numbers::substitute_x(&e, &Value::Var(COUNTED)) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = match old {
        Effect::Seq(mut v) => {
            v.push(e);
            Effect::Seq(v)
        }
        o => Effect::Seq(vec![o, e]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "referents values 6: if the number (counted) is odd/even", priority: 70, apply: if_the_number } }

/// "if a player taps a Mountain for mana, that Mountain produces ...": "that Mountain" is
/// the tapped permanent, "it".
fn tapped_referent(s: &str) -> Option<String> {
    let r = s.strip_prefix("if a player taps ")?;
    let (noun, rest) = r.split_once(" for mana, ")?;
    let noun = noun
        .strip_prefix("a ")
        .or_else(|| noun.strip_prefix("an "))?;
    let after = rest.strip_prefix(&format!("that {noun} "))?;
    Some(format!(
        "if a player taps a{} {noun} for mana, it {after}",
        if noun.starts_with(['a', 'e', 'i', 'o', 'u']) {
            "n"
        } else {
            ""
        }
    ))
}

/// "until end of turn, [effect] and [whenever ... | if ...]": two continuous effects
/// with the same duration; also "until end of turn, if a player taps a Mountain for mana,
/// that Mountain produces ..." alone.
fn until_eot_two_effects(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("until end of turn, ")?;
    if let Some(n) = tapped_referent(r) {
        return parse_sentence(&format!("until end of turn, {n}"), b);
    }
    let (first, second) = [" and whenever ", " and if "].iter().find_map(|sep| {
        r.split_once(sep)
            .map(|(a, c)| (a, format!("{}{c}", &sep[" and ".len()..])))
    })?;
    // One effect before "and": not a list of its own.
    if first.contains(" and ") || first.contains(", ") || first.starts_with("if ") {
        return None;
    }
    let second = tapped_referent(&second).unwrap_or(second);
    let saved = b.targets.len();
    let parts = (|| {
        let a = parse_sentence(&format!("until end of turn, {first}"), b)?;
        let c = parse_sentence(&format!("until end of turn, {second}"), b)?;
        Some((a, c))
    })();
    let Some((a, c)) = parts else {
        b.targets.truncate(saved);
        return None;
    };
    Some(Effect::Seq(vec![a, c]))
}

inventory::submit! { super::EffectPattern { name: "referents values 6: until end of turn, [effect] and [whenever/if ...]", priority: 90, parse: until_eot_two_effects } }
