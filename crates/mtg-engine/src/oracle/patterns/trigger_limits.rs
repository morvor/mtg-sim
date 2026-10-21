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

/// "At the beginning of each of your main phases, if you haven't added mana with this
/// ability this turn, you may add ..." (Carpet of Flowers): the ability triggers (and does
/// something as it resolves) only if its controller hasn't added mana with it this turn —
/// the same rule as "You may add .... Do this only once each turn." (CR 603.2h, 603.4).
fn if_you_havent_added_mana_with_this(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<crate::ability::Ability>> {
    use crate::ability::*;
    const IF: &str = ", if you haven't added mana with this ability this turn, ";
    let t = block.trim();
    let i = t.find(IF)?;
    let rest = &t[i + IF.len()..];
    if !rest.starts_with("you may add ") {
        return None;
    }
    let body = format!("{}, {rest}", &t[..i]);
    let a = crate::oracle::triggers::parse_triggered(&body, ctx)?;
    let AbilityKind::Triggered(mut tr) = a.kind.clone() else {
        return None;
    };
    if !matches!(tr.body.effect, Effect::May { .. }) {
        return None;
    }
    tr.do_once_per_turn = true;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { super::AbilityPattern { name: "if you haven't added mana with this ability this turn", priority: 0, parse: if_you_havent_added_mana_with_this } }
