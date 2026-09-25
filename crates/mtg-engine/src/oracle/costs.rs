//! Parsing activation costs ("{2}{G}, {T}, Sacrifice a creature") and activation
//! restrictions ("Activate only as a sorcery.").

use super::phrases::*;
use crate::ability::*;
use crate::mana::ManaCost;
use crate::types::*;

/// Parses a cost. Returns (cost, is_loyalty_ability).
pub fn parse_cost(s: &str) -> Option<(Cost, bool)> {
    let s = s.trim();
    // Loyalty costs: "+1", "-2", "0", "-X", "+X".
    let sl = s.replace('−', "-");
    if let Some(n) = parse_loyalty(&sl) {
        return Some((
            Cost {
                mana: None,
                parts: vec![CostPart::Loyalty(n)],
            },
            true,
        ));
    }
    let mut cost = Cost::default();
    for raw in split_cost_parts(s) {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        let lower = part.to_lowercase();
        if part.starts_with('{') {
            // A run of symbols: mana, {T}, {Q}, {E}.
            let mut mana = String::new();
            let mut i = 0;
            let bytes: Vec<char> = part.chars().collect();
            let mut rest = String::new();
            while i < bytes.len() {
                if bytes[i] == '{' {
                    let j = bytes[i..].iter().position(|c| *c == '}')? + i;
                    let sym: String = bytes[i + 1..j].iter().collect();
                    match sym.as_str() {
                        "T" => cost.parts.push(CostPart::Tap),
                        "Q" => cost.parts.push(CostPart::Untap),
                        "E" => {
                            // Energy: accumulate.
                            match cost.parts.iter_mut().find_map(|p| {
                                if let CostPart::PayEnergy(Value::Const(n)) = p {
                                    Some(n)
                                } else {
                                    None
                                }
                            }) {
                                Some(n) => *n += 1,
                                None => cost.parts.push(CostPart::PayEnergy(Value::Const(1))),
                            }
                        }
                        _ => mana.push_str(&format!("{{{sym}}}")),
                    }
                    i = j + 1;
                } else {
                    rest = bytes[i..].iter().collect();
                    break;
                }
            }
            if !mana.is_empty() {
                let m = ManaCost::parse(&mana)?;
                match cost.mana.as_mut() {
                    Some(t) => t.add(&m),
                    None => cost.mana = Some(m),
                }
            }
            if !rest.trim().is_empty() {
                return None;
            }
            continue;
        }
        cost.parts.push(parse_cost_part(&lower)?);
    }
    Some((cost, false))
}

fn parse_loyalty(s: &str) -> Option<i32> {
    let s = s.trim();
    if s == "0" {
        return Some(0);
    }
    if let Some(r) = s.strip_prefix('+') {
        return r.parse().ok();
    }
    if let Some(r) = s.strip_prefix('-') {
        return r.parse::<i32>().ok().map(|n| -n);
    }
    None
}

fn split_cost_parts(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

fn parse_cost_part(p: &str) -> Option<CostPart> {
    let p = end(p);
    if p == "sacrifice ~" {
        return Some(CostPart::SacrificeSelf);
    }
    if let Some(r) = strip(p, "sacrifice") {
        let (n, r2) = parse_number(r).unwrap_or((Value::Const(1), r));
        let (f, _, tail) = parse_object_phrase(r2)?;
        if !end(tail).is_empty() {
            return None;
        }
        let f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
        return Some(CostPart::Sacrifice {
            filter: f,
            count: n,
        });
    }
    if p == "discard ~" {
        return Some(CostPart::DiscardSelf);
    }
    if p == "discard your hand" {
        return Some(CostPart::DiscardHand);
    }
    if let Some(r) = strip(p, "discard") {
        let (n, r2) = parse_number(r)?;
        let random = r2.contains("at random");
        let r2 = r2.replace(" at random", "");
        let r2 = r2.trim();
        let filter = if r2 == "card" || r2 == "cards" {
            Filter::Any
        } else {
            let (f, _, tail) = parse_object_phrase(r2)?;
            if !end(tail).is_empty() {
                return None;
            }
            f
        };
        return Some(CostPart::Discard {
            filter,
            count: n,
            random,
        });
    }
    if let Some(r) = strip(p, "pay") {
        if let Some((n, r2)) = parse_number(r) {
            if strip(r2, "life").is_some() {
                return Some(CostPart::PayLife(n));
            }
        }
        return None;
    }
    if p == "exile ~" || p == "exile ~ from your graveyard" || p == "exile ~ from your hand" {
        return Some(CostPart::ExileSelf);
    }
    if let Some(r) = strip(p, "exile") {
        let (n, r2) = parse_number(r)?;
        let (f, _, tail) = parse_object_phrase(r2)?;
        let zone = if tail.contains("graveyard") || f.zone() == Some(ZoneKind::Graveyard) {
            ZoneKind::Graveyard
        } else if tail.contains("hand") || f.zone() == Some(ZoneKind::Hand) {
            ZoneKind::Hand
        } else {
            return None;
        };
        return Some(CostPart::Exile {
            filter: f,
            zone,
            count: n,
        });
    }
    if let Some(r) = strip(p, "remove") {
        let (n, r2) = parse_number(r)?;
        let (kind, r3) = counter_kind(r2)?;
        let r3 = strip(r3, "counters").or_else(|| strip(r3, "counter"))?;
        if end(r3) == "from ~" {
            return Some(CostPart::RemoveCounters { kind, count: n });
        }
        return None;
    }
    if let Some(r) = strip(p, "put") {
        let (n, r2) = parse_number(r)?;
        let (kind, r3) = counter_kind(r2)?;
        let r3 = strip(r3, "counters").or_else(|| strip(r3, "counter"))?;
        if end(r3) == "on ~" {
            return Some(CostPart::AddCounters { kind, count: n });
        }
        return None;
    }
    if let Some(r) = strip(p, "tap") {
        let (n, r2) = parse_number(r).unwrap_or((Value::Const(1), r));
        let r2 = strip(r2, "untapped").unwrap_or(r2);
        let (f, _, tail) = parse_object_phrase(r2)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(CostPart::TapUntapped {
            filter: Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
            count: n,
        });
    }
    if p == "return ~ to its owner's hand" {
        return Some(CostPart::ReturnSelfToHand);
    }
    if let Some(r) = strip(p, "return") {
        let (n, r2) = parse_number(r)?;
        let (f, _, tail) = parse_object_phrase(r2)?;
        if end(tail) == "to its owner's hand" || end(tail) == "to their owner's hand" {
            return Some(CostPart::ReturnToHand {
                filter: Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
                count: n,
            });
        }
        return None;
    }
    if p == "exert ~" {
        return Some(CostPart::ExertSelf);
    }
    if let Some(r) = strip(p, "mill") {
        let (n, _) = parse_card_count(r)?;
        return Some(CostPart::Mill(n));
    }
    if let Some(r) = strip(p, "collect evidence") {
        return Some(CostPart::CollectEvidence(r.trim().parse().ok()?));
    }
    if p == "forage" {
        return Some(CostPart::Forage);
    }
    None
}

/// "+1/+1 counter", "loyalty counter", "charge counters".
pub fn counter_kind(s: &str) -> Option<(CounterKind, &str)> {
    let s = s.trim_start();
    let (w, rest) = split_word(s);
    if w.starts_with('+') || w.starts_with('-') {
        return Some((w.into(), rest));
    }
    if w == "counter" || w == "counters" {
        return None;
    }
    Some((w.into(), rest))
}

/// Splits trailing activation restrictions off an effect text.
/// Returns (effect text, timing, max per turn, any player may activate).
pub fn split_activation_restrictions(s: &str) -> (&str, ActivationTiming, Option<u32>, bool) {
    let mut text = s.trim();
    let mut timing = ActivationTiming::Instant;
    let mut max = None;
    let mut any = false;
    loop {
        let lower = text.to_lowercase();
        let pats: [(&str, u8); 15] = [
            ("activate only as a sorcery.", 1),
            ("activate only once each turn.", 2),
            ("activate only during your turn.", 3),
            ("activate only during your upkeep.", 4),
            ("activate only during combat.", 5),
            ("activate only before blockers are declared.", 6),
            ("any player may activate this ability.", 7),
            ("activate only as a sorcery and only once each turn.", 8),
            ("activate only during an opponent's turn.", 9),
            // Combat timing windows (CR 506.8g).
            ("activate only before attackers are declared.", 10),
            ("activate only after attackers are declared.", 11),
            (
                "activate only during combat after blockers are declared.",
                12,
            ),
            ("activate only before the combat damage step.", 13),
            ("activate only before the end of combat step.", 14),
            (
                "activate only during combat before blockers are declared.",
                15,
            ),
        ];
        let mut matched = false;
        for (p, k) in pats {
            if lower.ends_with(p) {
                text = text[..text.len() - p.len()].trim();
                match k {
                    1 => timing = ActivationTiming::Sorcery,
                    2 => max = Some(1),
                    3 => timing = ActivationTiming::YourTurn,
                    4 => timing = ActivationTiming::YourUpkeep,
                    5 => timing = ActivationTiming::Combat,
                    6 => timing = ActivationTiming::BeforeBlockers,
                    7 => any = true,
                    8 => {
                        timing = ActivationTiming::Sorcery;
                        max = Some(1);
                    }
                    9 => timing = ActivationTiming::OpponentsTurn,
                    10..=15 => {
                        let (point, after, during_combat) = match k {
                            10 => (CombatPoint::AttackersDeclared, false, false),
                            11 => (CombatPoint::AttackersDeclared, true, false),
                            12 => (CombatPoint::BlockersDeclared, true, true),
                            13 => (CombatPoint::CombatDamageStep, false, false),
                            14 => (CombatPoint::EndOfCombatStep, false, false),
                            _ => (CombatPoint::BlockersDeclared, false, true),
                        };
                        timing = ActivationTiming::CombatWindow(CombatTiming {
                            point,
                            after,
                            during_combat,
                        });
                    }
                    _ => {}
                }
                matched = true;
                break;
            }
        }
        if !matched {
            break;
        }
    }
    (text, timing, max, any)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn costs() {
        let (c, l) = parse_cost("{2}{G}, {T}, Sacrifice a creature").unwrap();
        assert!(!l);
        assert_eq!(c.mana.as_ref().unwrap().mana_value(), 3);
        assert!(c.has_tap());
        assert_eq!(c.parts.len(), 2);
        let (c, l) = parse_cost("-3").unwrap();
        assert!(l);
        assert_eq!(c.loyalty(), Some(-3));
        let (c, _) = parse_cost("{T}, Pay 1 life").unwrap();
        assert_eq!(c.parts.len(), 2);
    }
}
