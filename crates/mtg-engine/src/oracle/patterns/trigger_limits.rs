//! Trigger limits as grammar: "This ability triggers only once." and "This ability
//! triggers only twice each turn." after a triggered ability's effect (CR 603.2;
//! `trigger_limits.rs`). "Only once each turn" is read by the core trigger parser.

use crate::ability::TriggerLimit;
use crate::oracle::phrases::parse_number;

/// Splits a trailing limit sentence off a triggered ability's effect text (original case).
pub fn limit_suffix(eff: &str) -> Option<(&str, TriggerLimit)> {
    let el = eff.to_lowercase();
    let t = el.trim_end().trim_end_matches('.');
    let i = t.rfind("this ability triggers only ")?;
    // A sentence of its own.
    if i > 0 && !t[..i].trim_end().ends_with('.') {
        return None;
    }
    let limit = match &t[i + "this ability triggers only ".len()..] {
        "once" => TriggerLimit::Once,
        "twice each turn" => TriggerLimit::PerTurn(2),
        other => {
            let n = other.strip_suffix(" times each turn")?;
            let (v, rest) = parse_number(n)?;
            if !rest.trim().is_empty() {
                return None;
            }
            match v {
                crate::ability::Value::Const(k) if k > 1 => TriggerLimit::PerTurn(k as u32),
                _ => return None,
            }
        }
    };
    Some((eff[..i].trim_end(), limit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits() {
        let (r, l) = limit_suffix("Draw a card. This ability triggers only once.").unwrap();
        assert_eq!((r, l), ("Draw a card.", TriggerLimit::Once));
        let (_, l) = limit_suffix("Draw a card. This ability triggers only twice each turn.").unwrap();
        assert_eq!(l, TriggerLimit::PerTurn(2));
        let (_, l) =
            limit_suffix("Draw a card. This ability triggers only three times each turn.").unwrap();
        assert_eq!(l, TriggerLimit::PerTurn(3));
        assert!(limit_suffix("Draw a card. This ability triggers only once each turn.").is_none());
        assert!(limit_suffix("Draw a card.").is_none());
    }
}
