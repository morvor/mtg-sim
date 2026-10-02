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
    // A loyalty cost in a quoted ability is written in brackets ("[+1]: ...").
    let sl = match sl.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        Some(inner) => inner.to_string(),
        None => sl,
    };
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
                    // Symbols are case-insensitive: callers may pass lowercased text
                    // ("you may pay {e}{e}").
                    let sym: String = bytes[i + 1..j].iter().collect::<String>().to_uppercase();
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
        match parse_cost_part(&lower) {
            Some(c) => cost.parts.push(c),
            // "Sacrifice ~ and a creature you control": several parts (see
            // `patterns::cost_parts`).
            None => cost
                .parts
                .extend(super::patterns::cost_parts::parse_compound(&lower)?),
        }
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
    // (start, end) byte ranges of the comma-separated parts.
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push((start, i));
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push((start, s.len()));
    // A comma list inside one part ("Discard an enchantment, instant, or sorcery card"):
    // a part starting with "or" continues the previous one, and so does each one-word
    // list item before it.
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (a, b) in out {
        let part = s[a..b].trim_start().to_lowercase();
        // A list of objects continues too: "Sacrifice a white creature, a blue creature,
        // and a black creature", "Tap three untapped Advisors, Artificers, and/or Monks".
        let continues = ["or ", "and ", "and/or ", "a ", "an ", "rounded "]
            .iter()
            .any(|w| part.starts_with(w));
        if continues && !merged.is_empty() {
            let mut first = merged.pop().unwrap();
            while !s[first.0..first.1].trim().contains(' ') && !merged.is_empty() {
                first = merged.pop().unwrap();
            }
            merged.push((first.0, b));
        } else {
            merged.push((a, b));
        }
    }
    merged.into_iter().map(|(a, b)| &s[a..b]).collect()
}

/// "... with different names" after the objects of a cost ("sacrifice three artifact
/// tokens with different names", "discard three cards with different names"): they must
/// be chosen together with that relationship (`Filter::Together`, CR 201.2b).
fn together_suffix(r: &str) -> (&str, Option<Filter>) {
    match end(r).strip_suffix(" with different names") {
        Some(rest) => (rest, Some(Filter::Together(TargetGroup::DifferentNames))),
        None => (r, None),
    }
}

/// Parses one part of a cost (lowercase): the core forms, then the registered patterns.
pub fn parse_cost_part(p: &str) -> Option<CostPart> {
    let p = end(p);
    parse_cost_part_core(p).or_else(|| {
        crate::oracle::patterns::cost_patterns()
            .iter()
            .find_map(|c| (c.parse)(p))
    })
}

fn parse_cost_part_core(p: &str) -> Option<CostPart> {
    if p == "sacrifice ~" {
        return Some(CostPart::SacrificeSelf);
    }
    // CR 701.4a: "behold three Elementals".
    if let Some(c) = super::patterns::a701_behold::behold_cost_part(p) {
        return Some(c);
    }
    if let Some(r) = strip(p, "sacrifice") {
        let (n, r2) = parse_number(r).unwrap_or((Value::Const(1), r));
        let (r2, together) = together_suffix(r2);
        let (f, _, tail) = parse_object_phrase(r2)?;
        if !end(tail).is_empty() {
            return None;
        }
        let mut f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
        if let Some(t) = together {
            f = Filter::and(vec![f, t]);
        }
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
        // Grandeur (CR 207.2c): "Discard another card named ~".
        if r.trim() == "another card named ~" {
            return Some(CostPart::Discard {
                filter: Filter::and(vec![Filter::SameNameAs(Box::new(Sel::This)), Filter::Other]),
                count: Value::c(1),
                random: false,
            });
        }
        let (n, r2) = parse_number(r)?;
        let random = r2.contains("at random");
        let r2 = r2.replace(" at random", "");
        let (r2, together) = together_suffix(r2.trim());
        let filter = if r2 == "card" || r2 == "cards" {
            Filter::Any
        } else if let Some((a, b)) = r2
            .split_once(" card or a ")
            .or_else(|| r2.split_once(" card or an "))
        {
            // "discard a Mountain card or a red card".
            let mut fs = Vec::new();
            for part in [format!("{a} card"), b.to_string()] {
                let (f, _, tail) = parse_object_phrase(&part)?;
                if !end(tail).is_empty() {
                    return None;
                }
                fs.push(f);
            }
            Filter::Or(fs)
        } else {
            let (f, _, tail) = parse_object_phrase(r2)?;
            if !end(tail).is_empty() {
                return None;
            }
            f
        };
        let filter = match together {
            Some(t) => Filter::and(vec![filter, t]),
            None => filter,
        };
        return Some(CostPart::Discard {
            filter,
            count: n,
            random,
        });
    }
    if let Some(r) = strip(p, "pay") {
        // "Pay life equal to ~'s power" (e.g. a ward cost): the amount is determined as
        // the cost is paid (CR 702.21b rulings).
        match r.trim() {
            "life equal to ~'s power" => {
                return Some(CostPart::PayLife(Value::PowerOf(Box::new(Sel::This))))
            }
            "life equal to ~'s toughness" => {
                return Some(CostPart::PayLife(Value::ToughnessOf(Box::new(Sel::This))))
            }
            _ => {}
        }
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
        // Nothing may follow the zone ("... from your graveyard and pay its mana cost" is
        // a different cost).
        let (zone, f) = match end(tail) {
            "from your graveyard" => (ZoneKind::Graveyard, f),
            "from a graveyard" => (ZoneKind::Graveyard, from_any_graveyard(f)),
            "from a single graveyard" => (ZoneKind::Graveyard, from_a_single_graveyard(f)),
            "from your hand" => (ZoneKind::Hand, f),
            "" if f.zone() == Some(ZoneKind::Graveyard) => {
                (ZoneKind::Graveyard, from_any_graveyard(f))
            }
            "" if f.zone() == Some(ZoneKind::Hand) => (ZoneKind::Hand, f),
            _ => return None,
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
    // "Put N [kind] counters on ~" (other "put" costs are left to the registered
    // patterns, e.g. a processor's "put a card an opponent owns from exile into that
    // player's graveyard").
    if let Some(r) = strip(p, "put") {
        let add_counters = || {
            let (n, r2) = parse_number(r)?;
            let (kind, r3) = counter_kind(r2)?;
            let r3 = strip(r3, "counters").or_else(|| strip(r3, "counter"))?;
            (end(r3) == "on ~").then_some(CostPart::AddCounters { kind, count: n })
        };
        if let Some(c) = add_counters() {
            return Some(c);
        }
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
    // "Reveal a black card in your hand" (e.g. a morph cost, CR 702.37a).
    if let Some(r) = strip(p, "reveal") {
        let (n, r2) = parse_number(r)?;
        let (f, _, tail) = parse_object_phrase(r2)?;
        if !matches!(end(tail), "" | "in your hand" | "from your hand") {
            return None;
        }
        return Some(CostPart::RevealFromHand {
            filter: f,
            count: n,
        });
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

/// Cards a cost exiles "from a graveyard": any player's, unless the phrase said whose
/// ("from your graveyard", "from an opponent's graveyard"). A cost's cards are otherwise
/// its payer's (see `Game::exile_cost_cards`).
pub fn from_any_graveyard(f: Filter) -> Filter {
    let owned = match &f {
        Filter::And(v) => v.iter().any(|x| matches!(x, Filter::OwnedBy(_))),
        Filter::OwnedBy(_) => true,
        _ => false,
    };
    if owned {
        f
    } else {
        Filter::and(vec![f, Filter::OwnedBy(PlayerRel::Any)])
    }
}

/// Cards a cost exiles "from a single graveyard": any player's, all with the same owner
/// (chosen together, `target_groups.rs`; Night Soil ruling).
pub fn from_a_single_graveyard(f: Filter) -> Filter {
    Filter::and(vec![
        from_any_graveyard(f),
        Filter::Together(TargetGroup::SameOwner),
    ])
}

/// "+1/+1 counter", "loyalty counter", "charge counters".
pub fn counter_kind(s: &str) -> Option<(CounterKind, &str)> {
    let s = s.trim_start();
    // Keyword counters named by two words (CR 122.1b): "a double strike counter".
    for k in crate::layers::KEYWORD_COUNTERS {
        if let Some(rest) = s.strip_prefix(k).filter(|_| k.contains(' ')) {
            if rest.starts_with(" counter") {
                return Some((k.into(), rest));
            }
        }
    }
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
        let pats: [(&str, u8); 18] = [
            ("activate only as a sorcery.", 1),
            ("activate only once each turn.", 2),
            ("activate only during your turn.", 3),
            ("activate only during your upkeep.", 4),
            ("activate only during combat.", 5),
            ("activate only before blockers are declared.", 6),
            ("any player may activate this ability.", 7),
            ("activate only as a sorcery and only once each turn.", 8),
            ("activate only during an opponent's turn.", 9),
            // CR 602.5e.
            ("activate only as an instant.", 18),
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
            (
                "activate only during your turn and only once each turn.",
                16,
            ),
            ("activate only during the end of combat step.", 17),
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
                    16 => {
                        timing = ActivationTiming::YourTurn;
                        max = Some(1);
                    }
                    18 => timing = ActivationTiming::AsInstant,
                    10..=15 | 17 => {
                        let (point, after, during_combat) = match k {
                            10 => (CombatPoint::AttackersDeclared, false, false),
                            11 => (CombatPoint::AttackersDeclared, true, false),
                            12 => (CombatPoint::BlockersDeclared, true, true),
                            13 => (CombatPoint::CombatDamageStep, false, false),
                            14 => (CombatPoint::EndOfCombatStep, false, false),
                            // "During the end of combat step": during combat, once
                            // that step has begun (CR 506.8).
                            17 => (CombatPoint::EndOfCombatStep, true, true),
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
