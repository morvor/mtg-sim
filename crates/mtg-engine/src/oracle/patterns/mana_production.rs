//! Amounts of mana (CR 106.1, 107.3): "Add X mana of any one color, where X is ~'s
//! power", "Add ten mana of any one color", "Add X mana in any combination of colors",
//! "Add three mana in any combination of {R} and/or {G}", "Add X {G}", "Whenever ~ is
//! dealt damage, add that much {R}"; and mana that
//! stays in the pool: "Until end of turn, you don't lose this mana as steps and phases
//! end." (CR 106.4).
//!
//! X is either defined by a following ", where X is ..." (see `r107_numbers.rs`, which
//! substitutes the value for X) or is the X chosen for the ability's cost ("Remove X ki
//! counters from ~: Add X mana of any one color", CR 107.3a).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "{r}" → R, and the rest of the text.
fn symbol(s: &str) -> Option<(ManaType, &str)> {
    let r = s.trim_start().strip_prefix('{')?;
    let (inner, rest) = r.split_once('}')?;
    let mut cs = inner.chars();
    let c = cs.next()?;
    if cs.next().is_some() {
        return None;
    }
    Some((ManaType::from_letter(c.to_ascii_uppercase())?, rest))
}

/// "{r} and/or {g}", "{w}, {u}, and/or {b}": two or more distinct mana types.
fn type_list(s: &str) -> Option<Vec<ManaType>> {
    let mut out = Vec::new();
    for w in s.split([',', ' ']).filter(|w| !w.is_empty()) {
        if matches!(w, "and/or" | "or" | "and") {
            continue;
        }
        let (t, rest) = symbol(w)?;
        if !rest.is_empty() || out.contains(&t) {
            return None;
        }
        out.push(t);
    }
    (out.len() >= 2).then_some(out)
}

/// An amount of mana: a number word, a numeral, or X. ("a"/"an" aren't amounts here.)
fn amount(s: &str) -> Option<(Value, &str)> {
    if s.starts_with("a ") || s.starts_with("an ") {
        return None;
    }
    // "that much": the amount of the triggering event (see `add_amount`).
    if let Some(r) = s.strip_prefix("that much ") {
        return Some((Value::EventAmount, r));
    }
    parse_number(s)
}

/// The production after "add ", for the amounts the core parser doesn't read.
fn production(r: &str) -> Option<ManaProduction> {
    let r = end(r);
    if r == "one mana of any color in your commander's color identity" {
        return Some(ManaProduction::CommanderIdentity);
    }
    let (n, rest) = amount(r)?;
    let rest = rest.trim_start();
    // "x mana of any one color", "ten mana of any one color".
    if rest == "mana of any one color" {
        // One, two and three are the core parser's; don't shadow them.
        if matches!(n, Value::Const(1..=3)) {
            return None;
        }
        return Some(ManaProduction::AnyOneColor(n));
    }
    if let Some(list) = rest.strip_prefix("mana in any combination of ") {
        if list == "colors" {
            // Constant amounts are parsed by `triggers_mana.rs`.
            if matches!(n, Value::Const(_)) {
                return None;
            }
            return Some(ManaProduction::AnyCombination(n));
        }
        return Some(ManaProduction::CombinationOf(type_list(list)?, n));
    }
    // "x {g}", "six {g}": that much mana of one type ("{g}{g}" is the core parser's).
    if !matches!(n, Value::Const(..=1)) {
        let (t, tail) = symbol(rest)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some(ManaProduction::Amount(t, n));
    }
    None
}

/// "add [amount] mana of any one color / in any combination of ...", "add x {g}".
fn add_amount(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("add ")?;
    // "Whenever ~ is dealt damage, add that much {R}": "that much" is the triggering
    // event's amount (the damage dealt, the number of attackers) only in the first
    // sentence of a triggered ability; later it may mean something else ("discard any
    // number of cards. If you do, draw that many cards and add that much {R}").
    if r.starts_with("that much ") && (!b.in_trigger || b.sentences != 0) {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: production(r)?,
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "mana: add an amount of mana", priority: 55, parse: add_amount } }

/// "Until end of turn, you don't lose this mana as steps and phases end." after a
/// sentence that adds mana (CR 106.4).
fn f_dont_lose_this_mana(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l != "until end of turn, you don't lose this mana as steps and phases end"
        || !super::mana_restrictions::contains_add_mana(prev)
        || matches!(prev, Effect::PersistentMana(_))
    {
        return false;
    }
    *prev = Effect::PersistentMana(Box::new(prev.clone()));
    true
}

inventory::submit! { FollowupPattern { name: "mana: you don't lose this mana", priority: 50, apply: f_dont_lose_this_mana } }
