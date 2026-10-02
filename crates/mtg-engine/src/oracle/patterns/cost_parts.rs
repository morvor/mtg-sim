//! Cost parts read as a verb and an object phrase (CR 118, 602.1a), beyond the fixed
//! forms the core cost parser knows (`oracle/costs.rs`):
//!
//! * removing counters: "Remove a +1/+1 counter from a creature you control", "Remove a
//!   counter from ~" (a counter of any kind), "Remove two counters from ~", "Remove three
//!   counters from among creatures you control", "Remove a quest counter from a permanent
//!   you control"; the player chooses each counter removed (CR 118.3);
//! * an amount the player chooses as the cost is paid: "Remove one or more [kind] counters
//!   from ~", "Remove any number of ...", "Remove all [kind] counters from ~", "Exile one
//!   or more creature cards from your graveyard": the amount is the ability's X
//!   ([`amount_as_x`], CR 107.3), so "the number of counters removed this way" and "for
//!   each card exiled this way" count it;
//! * sacrificing: "Sacrifice another creature or an artifact" (one permanent of either
//!   description), "Sacrifice an Aura attached to ~", "Sacrifice a creature named ...",
//!   "Sacrifice six creatures named ~", "Sacrifice all lands you control";
//! * exiling: "Exile a creature you control", "Exile another creature card from your
//!   graveyard", "Exile the top card of your library", "Exile the top four cards of your
//!   library", "Exile the top card of your graveyard";
//! * moving cards: "Put a card from your hand on top of your library", "Put a card exiled
//!   with ~ into its owner's graveyard", "Put three cards from your graveyard on the
//!   bottom of your library", "Put ~ on the bottom of its owner's library";
//! * compound parts joined by "and" ([`parse_compound`]): "Sacrifice ~ and a creature
//!   named Spitting Drake", "Sacrifice a white creature, a blue creature, and a black
//!   creature", "Sacrifice two lands and ~", "Remove three quest counters from ~ and
//!   sacrifice it", "Remove twelve time counters from ~ and exile it", "Remove an eon
//!   counter from ~ and return it to its owner's hand". Each is its own cost part, paid in
//!   order; one permanent can't be sacrificed twice.

use super::CostPattern;
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::phrases::*;
use crate::types::CounterKind;
use smol_str::SmolStr;

thread_local! {
    /// Whether the activation cost being compiled had an amount the player chooses
    /// rewritten as X ([`amount_as_x`]).
    static AMOUNT_X: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn you_control(f: Filter) -> Filter {
    Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)])
}

/// A card name as printed, from the raw Oracle text (the parser sees lowercase text).
pub(crate) fn printed_name(lower: &str) -> Option<SmolStr> {
    let raw = crate::oracle::raw_text();
    let i = raw.to_lowercase().find(&format!("named {lower}"))? + "named ".len();
    let name = raw.get(i..i + lower.len())?;
    // A name starts with a capital letter.
    name.chars()
        .next()
        .is_some_and(|c| c.is_uppercase())
        .then(|| name.into())
}

/// The qualifiers after an object phrase's head noun that the phrase parser leaves:
/// "attached to ~", "named ~", "named [name]", "exiled with ~".
fn qualified(f: Filter, tail: &str) -> Option<Filter> {
    let t = end(tail);
    let q = match t {
        "" => return Some(f),
        "attached to ~" => Filter::AttachedToAnyOf(Box::new(Sel::This)),
        "named ~" => Filter::SameNameAs(Box::new(Sel::This)),
        "exiled with ~" => Filter::And(vec![
            Filter::InZone(ZoneKind::Exile),
            Filter::In(Box::new(Sel::Linked)),
        ]),
        _ => {
            let name = t.strip_prefix("named ")?;
            if [",", " and ", " or ", "and/or", " named "]
                .iter()
                .any(|w| name.contains(w))
            {
                return None;
            }
            Filter::Named(printed_name(name)?)
        }
    };
    Some(Filter::and(vec![f, q]))
}

/// An object phrase with its qualifiers, nothing after it.
fn object(s: &str) -> Option<(Filter, bool)> {
    let (f, plural, tail) = parse_object_phrase(s)?;
    Some((qualified(f, tail)?, plural))
}

/// "a creature", "two lands", "another creature or an artifact", "six creatures named ~":
/// the number and the filter.
fn counted_objects(s: &str) -> Option<(Value, Filter)> {
    let s = s.trim();
    let (n, r) = match parse_number(s) {
        Some((n, r)) if !matches!(n, Value::X) => (n, r),
        _ if s.starts_with("another ") => (Value::c(1), s),
        _ => return None,
    };
    // "an artifact or another creature", "another creature or a Treasure".
    for sep in [" or a ", " or an ", " or another "] {
        if let Some((a, b)) = r.split_once(sep) {
            if n.as_const() != Some(1) {
                return None;
            }
            let b = if sep == " or another " {
                format!("another {b}")
            } else {
                b.to_string()
            };
            let (fa, _) = object(a)?;
            let (fb, _) = object(&b)?;
            return Some((n, Filter::Or(vec![fa, fb])));
        }
    }
    let (f, plural) = object(r)?;
    if plural != (n.as_const() != Some(1)) {
        return None;
    }
    Some((n, f))
}

/// "Sacrifice [objects]".
fn sacrifice(p: &str) -> Option<CostPart> {
    let r = strip(p, "sacrifice")?;
    if let Some(r) = r.strip_prefix("all ") {
        // Every one of them (CR 701.21a): as many as match.
        let (f, plural) = object(r)?;
        if !plural {
            return None;
        }
        let f = you_control(f);
        return Some(CostPart::Sacrifice {
            count: Value::Count(f.clone()),
            filter: f,
        });
    }
    let (n, f) = counted_objects(r)?;
    Some(CostPart::Sacrifice {
        filter: you_control(f),
        count: n,
    })
}

/// "Remove [amount] [kind] counter(s) from [~ | a permanent you control | among ...]".
fn remove_counters(p: &str) -> Option<CostPart> {
    let r = strip(p, "remove")?;
    let (n, r) = parse_number(r)?;
    // "a counter" (any kind) or "a +1/+1 counter".
    let (kind, r) = match strip(r, "counters").or_else(|| strip(r, "counter")) {
        Some(r) => (None, r),
        None => {
            let (k, r) = counter_kind(r)?;
            let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
            (Some(k), r)
        }
    };
    let r = strip(r, "from")?;
    if end(r) == "~" {
        return Some(match kind {
            Some(kind) => CostPart::RemoveCounters { kind, count: n },
            None => CostPart::RemoveCountersFromAmong {
                kind: None,
                filter: Filter::Source,
                count: n,
            },
        });
    }
    let (filter, plural) = match strip(r, "among") {
        Some(group) => {
            let (f, plural) = object(group)?;
            (f, plural)
        }
        None => {
            // "from a creature you control": one permanent.
            if n.as_const() != Some(1) {
                return None;
            }
            let r = if r.starts_with("another ") {
                r
            } else {
                r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?
            };
            let (f, plural) = object(r)?;
            (f, !plural)
        }
    };
    if !plural {
        return None;
    }
    // Only permanents you control (the qualifier is part of every such phrase).
    if !format!("{filter:?}").contains("ControlledBy(You)") {
        return None;
    }
    Some(CostPart::RemoveCountersFromAmong {
        kind,
        filter: Filter::and(vec![filter, Filter::InZone(ZoneKind::Battlefield)]),
        count: n,
    })
}

/// "Exile [objects you control | cards from your graveyard | the top N cards of your
/// library]".
fn exile(p: &str) -> Option<CostPart> {
    let r = strip(p, "exile")?;
    // "the top card of your library", "the top four cards of your library", "the top card
    // of your graveyard".
    if let Some(t) = r.strip_prefix("the top ") {
        let (n, t) = match parse_card_count(t) {
            Some(x) => x,
            None => (Value::c(1), strip(t, "card")?),
        };
        if matches!(n, Value::X) {
            return None;
        }
        let what = match end(t) {
            "of your library" => Sel::TopOfLibrary(PlayerRef::You, n.clone()),
            "of your graveyard" if n.as_const() == Some(1) => Sel::TopOfGraveyard(PlayerRef::You),
            _ => return None,
        };
        return Some(CostPart::Effect(Box::new(Effect::Move {
            what,
            to: Destination::zone(ZoneKind::Exile),
        })));
    }
    let (n, r) = match parse_number(r) {
        Some(x) => x,
        None if r.starts_with("another ") => (Value::c(1), r),
        None => return None,
    };
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural != (n.as_const() != Some(1)) {
        return None;
    }
    let zone = match (end(tail), f.zone()) {
        ("from your graveyard", _) | ("", Some(ZoneKind::Graveyard)) => ZoneKind::Graveyard,
        ("from your hand", _) | ("", Some(ZoneKind::Hand)) => ZoneKind::Hand,
        // "a creature you control".
        ("", None) if format!("{f:?}").contains("ControlledBy(You)") => ZoneKind::Battlefield,
        // "an instant or sorcery spell you control".
        ("", Some(ZoneKind::Stack)) if format!("{f:?}").contains("ControlledBy(You)") => {
            ZoneKind::Stack
        }
        _ => return None,
    };
    Some(CostPart::Exile {
        filter: f,
        zone,
        count: n,
    })
}

/// "Put a card from your hand on top of your library", "Put a card exiled with ~ into its
/// owner's graveyard", "Put three cards from your graveyard on the bottom of your library",
/// "Put ~ on the bottom of its owner's library".
fn put(p: &str) -> Option<CostPart> {
    let r = strip(p, "put")?;
    match end(r) {
        "~ on the bottom of its owner's library" => {
            return Some(CostPart::Effect(Box::new(Effect::Move {
                what: Sel::This,
                to: Destination::library_bottom(),
            })))
        }
        "~ on top of its owner's library" => {
            return Some(CostPart::Effect(Box::new(Effect::Move {
                what: Sel::This,
                to: Destination::library_top(),
            })))
        }
        _ => {}
    }
    let (n, r) = parse_number(r)?;
    if matches!(n, Value::X) {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural != (n.as_const() != Some(1)) {
        return None;
    }
    let tail = format!(" {}", end(tail));
    let (from, to) = [
        (" on top of your library", Destination::library_top()),
        (" on the bottom of your library", Destination::library_bottom()),
        (
            " into its owner's graveyard",
            Destination::zone(ZoneKind::Graveyard),
        ),
        (
            " into their owner's graveyard",
            Destination::zone(ZoneKind::Graveyard),
        ),
        (
            " into their owners' graveyards",
            Destination::zone(ZoneKind::Graveyard),
        ),
    ]
    .into_iter()
    .find_map(|(s, d)| Some((tail.strip_suffix(s)?.trim(), d)))?;
    // The phrase parser may have read "from your hand" into the filter already.
    let owned = |z: ZoneKind| format!("{f:?}").contains(&format!("InZone({z:?}), OwnedBy(You)"));
    let from = match (from, f.zone()) {
        ("", Some(ZoneKind::Hand)) if owned(ZoneKind::Hand) => "from your hand",
        ("", Some(ZoneKind::Graveyard)) if owned(ZoneKind::Graveyard) => "from your graveyard",
        (from, None) => from,
        _ => return None,
    };
    let from_zone = match from {
        "from your hand" => Filter::and(vec![
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        "from your graveyard" => Filter::and(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        "exiled with ~" => Filter::And(vec![
            Filter::InZone(ZoneKind::Exile),
            Filter::In(Box::new(Sel::Linked)),
        ]),
        _ => return None,
    };
    // A card put into a library or graveyard goes to its owner's (CR 400.3); "your
    // library" names cards you own.
    if to.zone == ZoneKind::Library && from == "exiled with ~" {
        return None;
    }
    // Moving cards from a hand is "PutFromHandOnLibrary" (the cards stay hidden).
    if from == "from your hand" && to.zone == ZoneKind::Library {
        return Some(CostPart::PutFromHandOnLibrary {
            filter: f,
            count: n,
            top: to.position == LibraryPosition::Top,
        });
    }
    Some(CostPart::Effect(Box::new(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, from_zone]),
            count: n,
            up_to: false,
            store: None,
        },
        to,
    })))
}

/// "Discard another card named ~" and other discards of a named card.
fn discard_named(p: &str) -> Option<CostPart> {
    let r = strip(p, "discard")?;
    let (n, f) = counted_objects(r)?;
    if !format!("{f:?}").contains("Name") {
        return None;
    }
    Some(CostPart::Discard {
        filter: f,
        count: n,
        random: false,
    })
}

/// "Unattach ~" (CR 701.3d), "Unattach an Equipment from ~".
fn unattach(p: &str) -> Option<CostPart> {
    let r = strip(p, "unattach")?;
    let what = match end(r) {
        "~" => Sel::This,
        r => {
            let what = r.strip_suffix(" from ~")?;
            let what = what.strip_prefix("a ").or_else(|| what.strip_prefix("an "))?;
            let (f, plural) = object(what)?;
            if plural {
                return None;
            }
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![f, Filter::AttachedToAnyOf(Box::new(Sel::This))]),
                count: Value::c(1),
                up_to: false,
                store: None,
            }
        }
    };
    Some(CostPart::Effect(Box::new(Effect::Unattach { what })))
}

/// "Pay half your life, rounded up" (CR 107.1a), "Pay fifty {E}".
fn pay(p: &str) -> Option<CostPart> {
    let r = end(strip(p, "pay")?);
    match r {
        "half your life, rounded up" | "half your life, rounded down" => {
            return Some(CostPart::PayLife(Value::Div(
                Box::new(Value::LifeTotal(PlayerRef::You)),
                2,
                r.ends_with("up"),
            )))
        }
        _ => {}
    }
    let (n, r) = parse_number(r)?;
    if matches!(n, Value::X) {
        return None;
    }
    // "{e}{e}" is read by the symbol parser; a number of {E} spelled out is energy too.
    (r.trim() == "{e}").then_some(CostPart::PayEnergy(n))
}

/// "Tap five untapped attacking creatures you control named ~".
fn tap(p: &str) -> Option<CostPart> {
    let r = strip(p, "tap")?;
    let (n, r) = parse_number(r)?;
    let r = strip(r, "untapped")?;
    let (f, plural) = object(r)?;
    if plural != (n.as_const() != Some(1)) {
        return None;
    }
    Some(CostPart::TapUntapped {
        filter: you_control(f),
        count: n,
    })
}

fn cost_part(p: &str) -> Option<CostPart> {
    sacrifice(p)
        .or_else(|| tap(p))
        .or_else(|| unattach(p))
        .or_else(|| pay(p))
        .or_else(|| remove_counters(p))
        .or_else(|| exile(p))
        .or_else(|| put(p))
        .or_else(|| discard_named(p))
}

inventory::submit! { CostPattern { name: "cost parts: verb and object phrase", priority: 50, parse: cost_part } }

/// Splits "a, b, and c" / "a and b" into items.
fn and_list(s: &str) -> Option<Vec<&str>> {
    let mut items: Vec<&str> = s.split(", ").collect();
    let last = items.pop()?;
    let last_items: Vec<&str> = match last.split_once(" and ") {
        Some((a, b)) if items.is_empty() => vec![a, b],
        Some(_) => return None,
        None => vec![last],
    };
    let mut out: Vec<&str> = items;
    for (i, it) in last_items.into_iter().enumerate() {
        let it = if i == 0 && !out.is_empty() {
            // "a, b, and c": the last item starts with "and".
            it.strip_prefix("and ")?
        } else {
            it
        };
        out.push(it);
    }
    if out.len() < 2 || out.iter().any(|x| x.trim().is_empty()) {
        return None;
    }
    // "a, b, and c" needs "and" before the last item; "a and b" has it already.
    if s.contains(", ") && !s.contains(", and ") {
        return None;
    }
    Some(out.into_iter().map(str::trim).collect())
}

/// One cost part a compound part may end with: "and sacrifice it", "and exile it", "and
/// return it to its owner's hand", "and sacrifice ~".
fn self_followup(s: &str) -> Option<CostPart> {
    match end(s) {
        "sacrifice it" | "sacrifice ~" => Some(CostPart::SacrificeSelf),
        "exile it" | "exile ~" => Some(CostPart::ExileSelf),
        "return it to its owner's hand" => Some(CostPart::ReturnSelfToHand),
        _ => None,
    }
}

/// Compound cost parts joined by "and" (lowercase): each part in order.
pub fn parse_compound(p: &str) -> Option<Vec<CostPart>> {
    let p = end(p);
    // "Remove three quest counters from ~ and sacrifice it".
    if p.starts_with("remove ") {
        let (a, b) = p.rsplit_once(" and ")?;
        let first = crate::oracle::costs::parse_cost_part(a)?;
        return Some(vec![first, self_followup(b)?]);
    }
    let (verb, rest) = split_word(p);
    if !matches!(verb, "sacrifice" | "exile") {
        return None;
    }
    let items = and_list(rest)?;
    let mut out = Vec::new();
    for it in items {
        let part = format!("{verb} {it}");
        let c = crate::oracle::costs::parse_cost_part(&part)?;
        // Nothing paid in a way that could pick the same object twice without "other".
        out.push(c);
    }
    // "Sacrifice ~ and a creature you control": ~ is paid for first so it isn't chosen
    // again.
    out.sort_by_key(|c| !matches!(c, CostPart::SacrificeSelf | CostPart::ExileSelf));
    Some(out)
}

/// An amount the player chooses as the cost is paid, rewritten as X (CR 107.3): "remove
/// one or more [kind] counters from ~" → "remove x ...", with the condition X is 1 or more;
/// "remove any number of" → X; "remove all [kind] counters from ~" → X, with the condition
/// that X is the number of those counters on it. `None` if the cost has no such amount (or
/// already has an X).
pub fn amount_as_x(cost: &str) -> Option<(String, Option<Condition>)> {
    let lower = cost.to_lowercase();
    if super::value_grammar::cost_has_x(&lower) {
        return None;
    }
    for (verb, phrase) in [
        ("remove", "one or more "),
        ("remove", "any number of "),
        ("remove", "all "),
        ("exile", "one or more "),
        ("exile", "any number of "),
        ("sacrifice", "one or more "),
        ("sacrifice", "any number of "),
    ] {
        let pat = format!("{verb} {phrase}");
        let Some(i) = lower.find(&pat) else {
            continue;
        };
        let after = &lower[i + pat.len()..];
        let rewritten = format!("{}{verb} x {after}", &lower[..i]);
        let cond = match phrase {
            "one or more " => Some(Condition::Compare(Value::X, Cmp::Ge, Value::c(1))),
            "any number of " => None,
            _ => {
                // "all [kind] counters from ~" (only the source's own counters).
                let part = after.split(',').next().unwrap_or(after);
                let part = part.split(" and ").next().unwrap_or(part);
                let (kind, r) = counter_kind(part)?;
                if end(strip(r, "counters")?) != "from ~" {
                    return None;
                }
                Some(Condition::Compare(
                    Value::X,
                    Cmp::Eq,
                    Value::CountersOn(Box::new(Sel::This), Some(kind)),
                ))
            }
        };
        return Some((rewritten, cond));
    }
    None
}

/// Runs `f` with [`paid_this_way`] recognizing the amount [`amount_as_x`] rewrote.
pub fn with_amount_x<T>(on: bool, f: impl FnOnce() -> T) -> T {
    let saved = AMOUNT_X.with(|c| c.replace(on));
    let out = f();
    AMOUNT_X.with(|c| c.set(saved));
    out
}

/// "+1/+1 counters removed this way", "counters removed this way", "cards exiled this
/// way": the amount the player chose for the cost, the ability's X ([`amount_as_x`]).
pub fn paid_this_way(s: &str) -> Option<Value> {
    if !AMOUNT_X.with(|c| c.get()) {
        return None;
    }
    let s = end(s);
    let s = s.strip_prefix("the ").unwrap_or(s);
    if let Some(r) = s
        .strip_suffix(" removed this way")
        .or_else(|| s.strip_suffix(" removed"))
    {
        let r = r.strip_suffix("counters").or_else(|| r.strip_suffix("counter"))?;
        let r = r.trim();
        if !r.is_empty() {
            let (_k, rest): (CounterKind, &str) = counter_kind(r)?;
            if !rest.trim().is_empty() {
                return None;
            }
        }
        return Some(Value::X);
    }
    let r = s
        .strip_suffix(" exiled this way")
        .or_else(|| s.strip_suffix(" sacrificed this way"))?;
    let (_f, _, tail) = parse_object_phrase(r)?;
    end(tail).is_empty().then_some(Value::X)
}

/// [`paid_this_way`] at the start of `r` ("+1/+1 counters removed this way plus one"):
/// the value and the rest.
pub fn paid_this_way_prefix(r: &str) -> Option<(Value, String)> {
    if !AMOUNT_X.with(|c| c.get()) {
        return None;
    }
    for w in [" removed this way", " exiled this way", " sacrificed this way"] {
        if let Some(i) = r.find(w) {
            let (head, rest) = r.split_at(i + w.len());
            let v = paid_this_way(head)?;
            return Some((v, rest.to_string()));
        }
    }
    None
}

/// The effect text of an ability whose cost amount is its X ([`amount_as_x`]), with "that
/// many" / "that much" in its first sentence meaning that amount ("Create that many 1/1
/// green Insect creature tokens", "Add that much {C}"). `None` if it says neither, or says
/// one of them more than once or after the first sentence.
pub fn effect_with_amount(eff: &str) -> Option<String> {
    if eff.to_lowercase().len() != eff.len() {
        return None;
    }
    // "with power less than or equal to the number of +1/+1 counters removed this way"
    // (Simic Manipulator): a comparison with that amount.
    let compared = compared_with_amount(eff);
    let eff = compared.as_deref().unwrap_or(eff);
    let lower = eff.to_lowercase();
    let first = lower.split(". ").next().unwrap_or(&lower);
    for (pat, rep) in [
        ("that many ", "X "),
        ("that much {c}", "X {C}"),
        ("that much damage", "X damage"),
    ] {
        if lower.matches(pat).count() == 1 && first.contains(pat) {
            let i = lower.find(pat)?;
            return Some(format!("{}{rep}{}", &eff[..i], &eff[i + pat.len()..]));
        }
    }
    compared
}

/// "with [stat] less than or equal to the number of [things] [paid] this way" →
/// "with [stat] X or less" (see [`effect_with_amount`]).
fn compared_with_amount(eff: &str) -> Option<String> {
    for stat in ["power", "toughness", "mana value"] {
        let pat = format!("with {stat} less than or equal to the number of ");
        let Some(i) = eff.find(&pat) else {
            continue;
        };
        let rest = &eff[i + pat.len()..];
        let j = [" removed this way", " exiled this way", " sacrificed this way"]
            .iter()
            .filter_map(|w| rest.find(w).map(|j| j + w.len()))
            .min()?;
        // The counted things are one object phrase ("+1/+1 counters", "cards").
        if rest[..j].contains(['.', ',']) {
            return None;
        }
        return Some(format!(
            "{}with {stat} X or less{}",
            &eff[..i],
            &rest[j..]
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::costs::parse_cost;

    fn parts(s: &str) -> Vec<CostPart> {
        parse_cost(s).unwrap_or_else(|| panic!("{s:?}")).0.parts
    }

    #[test]
    fn verb_and_object_cost_parts() {
        // Counters from another permanent, any kind or one kind.
        assert!(matches!(
            parts("{2}, Remove a +1/+1 counter from a creature you control").as_slice(),
            [CostPart::RemoveCountersFromAmong { kind: Some(k), count: Value::Const(1), .. }] if k == "+1/+1"
        ));
        assert!(matches!(
            parts("{T}, Remove a counter from another permanent you control").as_slice(),
            [CostPart::Tap, CostPart::RemoveCountersFromAmong { kind: None, .. }]
        ));
        assert!(matches!(
            parts("Remove two counters from ~").as_slice(),
            [CostPart::RemoveCountersFromAmong { kind: None, filter: Filter::Source, count: Value::Const(2) }]
        ));
        // "... from ~ and sacrifice it": two parts.
        assert!(matches!(
            parts("Remove four quest counters from ~ and sacrifice it").as_slice(),
            [CostPart::RemoveCounters { count: Value::Const(4), .. }, CostPart::SacrificeSelf]
        ));
        // Sacrifices: alternatives, several objects, ~ first.
        assert!(matches!(
            parts("{1}, Sacrifice another creature or an artifact").as_slice(),
            [CostPart::Sacrifice { filter: Filter::And(_), count: Value::Const(1) }]
        ));
        assert_eq!(
            parts("{3}{B}, {T}, Sacrifice a blue creature, a black creature, and a red creature")
                .iter()
                .filter(|p| matches!(p, CostPart::Sacrifice { .. }))
                .count(),
            3
        );
        assert!(matches!(
            parts("{T}, Sacrifice two lands and ~").as_slice(),
            [CostPart::Tap, CostPart::SacrificeSelf, CostPart::Sacrifice { count: Value::Const(2), .. }]
        ));
        // Exile from the top of a library, from a graveyard, a spell from the stack.
        assert!(matches!(
            parts("{2}, Exile the top card of your library").as_slice(),
            [CostPart::Effect(_)]
        ));
        assert!(matches!(
            parts("{4}{B}, Exile another creature card from your graveyard").as_slice(),
            [CostPart::Exile { zone: ZoneKind::Graveyard, .. }]
        ));
        assert!(matches!(
            parts("Exile an instant or sorcery spell you control").as_slice(),
            [CostPart::Exile { zone: ZoneKind::Stack, .. }]
        ));
        // Tapping several untapped creatures; putting a card from hand on top.
        assert!(matches!(
            parts("Tap three untapped Advisors, Artificers, and/or Monks you control").as_slice(),
            [CostPart::TapUntapped { count: Value::Const(3), .. }]
        ));
        assert!(matches!(
            parts("Put a card from your hand on top of your library").as_slice(),
            [CostPart::PutFromHandOnLibrary { top: true, .. }]
        ));
        // Not a cost this grammar knows.
        assert!(parse_cost("Exile the top creature card of your graveyard").is_none());
    }

    #[test]
    fn chosen_amounts_become_x() {
        let (c, cond) = amount_as_x("{T}, Remove one or more +1/+1 counters from ~").unwrap();
        assert_eq!(c, "{t}, remove x +1/+1 counters from ~");
        assert!(matches!(cond, Some(Condition::Compare(Value::X, Cmp::Ge, _))));
        let (c, cond) = amount_as_x("Remove all charge counters from ~").unwrap();
        assert_eq!(c, "remove x charge counters from ~");
        assert!(matches!(cond, Some(Condition::Compare(Value::X, Cmp::Eq, _))));
        assert!(amount_as_x("{T}, Sacrifice one or more artifacts").is_some());
        assert!(amount_as_x("{X}, Remove X +1/+1 counters from ~").is_none());
        assert_eq!(
            effect_with_amount("It deals that much damage to any target.").as_deref(),
            Some("It deals X damage to any target.")
        );
        assert_eq!(
            effect_with_amount("Create that many 1/1 green Insect creature tokens.").as_deref(),
            Some("Create X 1/1 green Insect creature tokens.")
        );
        assert_eq!(
            effect_with_amount(
                "Gain control of target creature with power less than or equal to the number of +1/+1 counters removed this way."
            )
            .as_deref(),
            Some("Gain control of target creature with power X or less.")
        );
        assert_eq!(effect_with_amount("Draw a card."), None);
    }
}
