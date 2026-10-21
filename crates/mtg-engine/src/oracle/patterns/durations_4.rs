//! Durations, grammar IV (cards3-triggers-durations): "for as long as" written before the
//! instructions it applies to (CR 611.2b).
//!
//! "For as long as ~ remains tapped, another target permanent loses all abilities and
//! can't attack or block." (Immovable Rod), "For as long as ~ remains on the battlefield,
//! gain control of that permanent, it loses all abilities, and it can't attack or block."
//! (Opportunistic Dragon): each instruction of the list is read with the duration after
//! it, as the core grammar reads "[instruction] for as long as [condition]"; a second
//! predicate of the same subject ("... and can't attack or block") is about it.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Verbs that start a predicate whose subject is the previous clause's.
const PREDICATE_STARTS: [&str; 14] = [
    "can't ", "cannot ", "doesn't ", "don't ", "gets ", "get ", "gains ", "gain ", "loses ",
    "lose ", "has ", "have ", "is ", "becomes ",
];

/// The instructions of a list: "A, B, and C", "A and C" (a predicate without a subject
/// becomes "it [predicate]").
fn clauses(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for chunk in s.split(", ") {
        let chunk = chunk.strip_prefix("and ").unwrap_or(chunk);
        // "loses all abilities and can't attack or block".
        let mut rest = chunk;
        let mut pieces = Vec::new();
        while let Some((a, b)) = rest.split_once(" and ") {
            if PREDICATE_STARTS.iter().any(|p| b.starts_with(p)) {
                pieces.push(a.to_string());
                rest = b;
                pieces.push(format!("it {rest}"));
                rest = "";
                break;
            }
            break;
        }
        if pieces.is_empty() {
            parts.push(chunk.to_string());
        } else {
            if !rest.is_empty() {
                pieces.push(rest.to_string());
            }
            parts.extend(pieces);
        }
    }
    parts
}

fn for_as_long_as_first(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for as long as ")?;
    let (cond, rest) = r.split_once(", ")?;
    let suffix = format!(" for as long as {cond}");
    // Only durations the core grammar reads.
    let (d, _) = crate::oracle::effects::duration_suffix(&format!("x{suffix}"));
    if matches!(d, Duration::Permanent) {
        return None;
    }
    let parts = clauses(rest);
    if parts.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for p in parts {
        // A clause that isn't an instruction with its own duration (a trigger "at the
        // beginning of ...") isn't read here.
        if p.starts_with("at ") || p.starts_with("when") {
            return None;
        }
        out.push(parse_sentence(&format!("{p}{suffix}"), b)?);
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "durations 4: for as long as [condition], [instructions]", priority: 100, parse: for_as_long_as_first } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_instruction_lists() {
        assert_eq!(
            clauses("another target permanent loses all abilities and can't attack or block"),
            vec![
                "another target permanent loses all abilities".to_string(),
                "it can't attack or block".to_string()
            ]
        );
        assert_eq!(
            clauses("gain control of that permanent, it loses all abilities, and it can't attack or block"),
            vec![
                "gain control of that permanent".to_string(),
                "it loses all abilities".to_string(),
                "it can't attack or block".to_string()
            ]
        );
    }
}
