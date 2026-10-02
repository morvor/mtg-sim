//! Object qualifiers about what happened this turn and exceptions (see
//! `kw/basic_effects.rs` for the filters):
//!
//! - "that dealt damage this turn" (as a source, CR 120.1), "dealt damage this turn" (was
//!   dealt damage), "that didn't enter this turn";
//! - "that blocked this turn", "that blocked or was blocked this turn", "that blocked or
//!   was blocked by a legendary creature this turn" (CR 509.1, 509.1h);
//! - "except for [objects]": "all permanents except for artifacts and lands", "each
//!   creature except for creatures you control with flying", "all creatures except for
//!   ~".

use super::FilterSuffixPattern;
use crate::ability::*;
use crate::kw::basic_effects as be;
use crate::oracle::phrases::*;

fn this_turn<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let simple: [(&str, Filter); 6] = [
        (
            "that dealt damage this turn",
            Filter::Custom(be::DEALT_DAMAGE_THIS_TURN.into()),
        ),
        ("dealt damage this turn", Filter::DealtDamageThisTurn),
        (
            "that didn't enter this turn",
            Filter::not(Filter::EnteredThisTurn),
        ),
        (
            "that blocked this turn",
            Filter::Custom(be::BLOCKED_THIS_TURN.into()),
        ),
        (
            "that blocked or was blocked this turn",
            Filter::Custom(be::blocked_or_was_blocked_by(&Filter::Any).into()),
        ),
        (
            "that blocked or were blocked this turn",
            Filter::Custom(be::blocked_or_was_blocked_by(&Filter::Any).into()),
        ),
    ];
    for (p, f) in simple {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((f, r));
            }
        }
    }
    // "creature ~ is blocking" (Wall of Corpses): an attacker it blocks (CR 509.1).
    if let Some(r) = t.strip_prefix("~ is blocking") {
        return Some((Filter::Custom(be::BLOCKED_BY_SOURCE_LKI.into()), r));
    }
    // "all creatures ~ blocked this turn" (Defiant Vanguard).
    if let Some(r) = t.strip_prefix("~ blocked this turn") {
        return Some((Filter::Custom(be::BLOCKED_BY_SOURCE_THIS_TURN.into()), r));
    }
    // "that blocked or was blocked by a legendary creature this turn"
    let r = t.strip_prefix("that blocked or was blocked by ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (by, plural, rest) = parse_object_phrase(r)?;
    if plural {
        return None;
    }
    let rest = rest.trim_start().strip_prefix("this turn")?;
    Some((
        Filter::Custom(be::blocked_or_was_blocked_by(&by).into()),
        rest,
    ))
}

/// "except for ~", "except for artifacts and lands", "except for artifacts, lands, and
/// Phyrexians", "except for creatures you control with flying": the rest of the phrase.
fn except_for<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t
        .strip_prefix("except for ")
        .or_else(|| t.strip_prefix(", except for "))?;
    if let Some(rest) = r.strip_prefix('~') {
        if rest.is_empty() || rest.starts_with([' ', ',', '.']) {
            return Some((Filter::not(Filter::Source), rest));
        }
        return None;
    }
    // The exception ends the phrase: read it as alternatives.
    let alts = r.replace(", and ", ", or ").replace(" and ", " or ");
    let f = match super::basic_effects_targets::object_alternatives(&alts) {
        Some((f, true, rest)) if end(rest).is_empty() => f,
        _ => {
            let (f, plural, rest) = parse_object_phrase(r)?;
            if !plural || !end(rest).is_empty() {
                return None;
            }
            f
        }
    };
    Some((Filter::not(f), ""))
}

/// "that's a Wolf or a Werewolf": the object is one of them.
fn thats_a<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let mut r = t.strip_prefix("that's ")?;
    let mut alts = Vec::new();
    loop {
        let x = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
        let (w, rest) = split_word(x);
        let (f, plural, tail) = parse_object_phrase(w)?;
        if plural || !tail.trim().is_empty() {
            return None;
        }
        alts.push(f);
        r = rest;
        match r.strip_prefix("or ") {
            Some(x) => r = x,
            None => break,
        }
    }
    // A list ("that's a Cat, Elemental, or Beast") is read elsewhere, and so is a clause
    // that ends the phrase ("each creature you control that's an Ape or a Monkey.").
    if alts.len() < 2 || r.trim_start().starts_with(',') || end(r).trim().is_empty() {
        return None;
    }
    Some((Filter::Or(alts), r))
}

inventory::submit! { FilterSuffixPattern { name: "basic effects: that's a X or a Y", priority: 110, parse: thats_a } }
inventory::submit! { FilterSuffixPattern { name: "basic effects: this turn (dealt damage, blocked, entered)", priority: 90, parse: this_turn } }
inventory::submit! { FilterSuffixPattern { name: "basic effects: except for", priority: 90, parse: except_for } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifiers() {
        let (f, _, rest) = parse_object_phrase("permanents except for artifacts and lands").unwrap();
        assert_eq!(end(rest), "");
        assert!(format!("{f:?}").contains("Not(Or("), "{f:?}");
        let (f, _, rest) =
            parse_object_phrase("other permanents except for artifacts, lands, and phyrexians").unwrap();
        assert_eq!(end(rest), "");
        assert!(format!("{f:?}").contains("Phyrexian"), "{f:?}");
        let (_, _, rest) = parse_object_phrase("creature except for creatures you control with flying").unwrap();
        assert_eq!(end(rest), "");
        let (f, _, rest) = parse_object_phrase("creatures except for ~").unwrap();
        assert_eq!(end(rest), "");
        assert!(format!("{f:?}").contains("Not(Source)"), "{f:?}");
        let (_, _, rest) = parse_object_phrase("creature that dealt damage this turn").unwrap();
        assert_eq!(end(rest), "");
        let (_, _, rest) =
            parse_object_phrase("creature that blocked or was blocked by a legendary creature this turn").unwrap();
        assert_eq!(end(rest), "");
    }
}

/// "with power 4, 5, or 6" (Sarkhan's Unsealing), "with mana value 1, 2, or 3": any of the
/// listed numbers.
pub(crate) fn stat_in_list(s: &str) -> Option<(Filter, &str)> {
    let t = s.trim_start();
    let (stat, mut r) = if let Some(r) = t.strip_prefix("with power ") {
        ("power", r)
    } else if let Some(r) = t.strip_prefix("with toughness ") {
        ("toughness", r)
    } else if let Some(r) = t.strip_prefix("with mana value ") {
        ("mv", r)
    } else {
        return None;
    };
    // A number (digits, then a comma, a space or the end).
    let num = |x: &str| -> Option<(i32, usize)> {
        let d = x.chars().take_while(|c| c.is_ascii_digit()).count();
        Some((x[..d].parse().ok()?, d))
    };
    let mut ns = Vec::new();
    loop {
        let (n, d) = num(r)?;
        ns.push(n);
        let rest = &r[d..];
        if let Some(x) = rest.strip_prefix(", or ").or_else(|| rest.strip_prefix(" or ")) {
            let (n, d) = num(x)?;
            ns.push(n);
            r = &x[d..];
            break;
        }
        r = rest.strip_prefix(", ")?;
    }
    if ns.len() < 3 {
        return None;
    }
    let one = |n: i32| {
        let v = Box::new(Value::Const(n));
        match stat {
            "power" => Filter::Power(Cmp::Eq, v),
            "toughness" => Filter::Toughness(Cmp::Eq, v),
            _ => Filter::ManaValue(Cmp::Eq, v),
        }
    };
    Some((Filter::Or(ns.into_iter().map(one).collect()), r))
}
