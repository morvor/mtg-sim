//! Object-phrase qualifiers used by trigger subjects ("whenever a creature you control
//! with an impostor counter on it dies"), registered as [`FilterSuffixPattern`]s so every
//! object phrase understands them:
//!
//! * `with an [kind] counter on it` (the "a [kind]" form is core), `with counters on it`,
//!   `with one or more counters on it` (any kind, CR 122.1);
//! * `with power or toughness N or less/greater` (either one);
//! * `with [keyword] or [keyword]` ("with flash or haste");
//! * `named ~` (the card's own name, CR 201.2);
//! * `owned by another player`, `owned by an opponent`, `an opponent owns`.

use super::FilterSuffixPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::*;

fn word_end(r: &str) -> bool {
    r.is_empty() || r.starts_with(' ') || r.starts_with(',')
}

/// "with an infection counter on it", "with counters on it", "with one or more counters
/// on it".
fn with_counters<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for p in ["with counters on it", "with one or more counters on it"] {
        if let Some(r) = t.strip_prefix(p) {
            if word_end(r) {
                return Some((Filter::HasCounter(None), r));
            }
        }
    }
    let r = t.strip_prefix("with an ")?;
    let (kind, r) = split_word(r);
    let r = r.strip_prefix("counter on it")?;
    if !word_end(r) || kind.is_empty() || !kind.chars().all(|c| c.is_alphabetic()) {
        return None;
    }
    Some((Filter::HasCounter(Some(kind.into())), r))
}

/// "with power or toughness 1 or less": either one.
fn power_or_toughness<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("with power or toughness ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    let r = r.trim_start();
    let (cmp, r) = if let Some(x) = r.strip_prefix("or less") {
        (Cmp::Le, x)
    } else if let Some(x) = r.strip_prefix("or greater") {
        (Cmp::Ge, x)
    } else {
        return None;
    };
    if !word_end(r) {
        return None;
    }
    let v = || Box::new(Value::c(n));
    Some((
        Filter::Or(vec![Filter::Power(cmp, v()), Filter::Toughness(cmp, v())]),
        r,
    ))
}

/// "with flash or haste": either keyword.
fn with_keyword_or_keyword<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("with ")?;
    let (a, r) = split_word(r);
    let r = r.strip_prefix("or ")?;
    let (b, r) = split_word(r);
    let ka = KeywordKind::from_name(a)?;
    let kb = KeywordKind::from_name(b.trim_end_matches(','))?;
    Some((
        Filter::Or(vec![Filter::HasKeyword(ka), Filter::HasKeyword(kb)]),
        r,
    ))
}

/// "named ~": with this card's name.
fn named_self<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("named ~")?;
    word_end(r).then(|| (Filter::SameNameAs(Box::new(Sel::This)), r))
}

/// "owned by another player", "owned by an opponent", "an opponent owns".
fn owned_by<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for (p, rel) in [
        ("owned by another player", PlayerRel::NotYou),
        ("owned by an opponent", PlayerRel::Opponent),
        ("an opponent owns", PlayerRel::Opponent),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if word_end(r) {
                return Some((Filter::OwnedBy(rel), r));
            }
        }
    }
    None
}

/// Whether an object phrase says it's owned by another player ("a permanent an opponent
/// owns"): "they"/"that player" in a trigger about it is that owner.
pub(crate) fn names_other_owner(f: &Filter) -> bool {
    match f {
        Filter::OwnedBy(PlayerRel::Opponent | PlayerRel::NotYou) => true,
        Filter::And(v) => v.iter().any(names_other_owner),
        _ => false,
    }
}

inventory::submit! { FilterSuffixPattern { name: "trigger subjects: with an [kind] counter / counters on it", priority: 120, parse: with_counters } }
inventory::submit! { FilterSuffixPattern { name: "trigger subjects: with power or toughness N", priority: 120, parse: power_or_toughness } }
inventory::submit! { FilterSuffixPattern { name: "trigger subjects: with [keyword] or [keyword]", priority: 120, parse: with_keyword_or_keyword } }
inventory::submit! { FilterSuffixPattern { name: "trigger subjects: named ~", priority: 120, parse: named_self } }
inventory::submit! { FilterSuffixPattern { name: "trigger subjects: owned by another player", priority: 120, parse: owned_by } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffixes_parse() {
        for s in [
            "creature with an impostor counter on it",
            "legendary creature with counters on it",
            "creature you control with one or more counters on it",
            "creature you control with power or toughness 1 or less",
            "creature you control with flash or haste",
            "creature you control named ~",
            "permanent owned by another player",
            "nonland permanent an opponent owns",
        ] {
            let (_, _, tail) = parse_object_phrase(s).unwrap_or_else(|| panic!("{s}"));
            assert!(end(tail).is_empty(), "{s}: {tail:?}");
        }
    }
}
