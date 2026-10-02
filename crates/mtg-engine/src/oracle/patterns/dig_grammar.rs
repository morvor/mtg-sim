//! The dig grammar: what happens to cards looked at, revealed, milled or exiled from the
//! top of a library (CR 401, 701.20), composed step by step instead of as fixed sentences.
//!
//! * Source: "Look at the top N cards of your library", "Reveal the top N cards", "Mill N
//!   cards", "Exile the top N cards", "Reveal cards from the top of your library until you
//!   reveal N [kind] cards", "look at that many cards from the top of your library". The
//!   cards are kept in [`vars::DUG`] as the source resolves.
//! * Selection steps ([`DigStep::Take`]): "[you may] put/reveal/exile/return/choose
//!   [count] [description] from among them/those cards/the milled cards/the revealed cards
//!   [and put it/them] [destination]", "put [count] of them [destination]", with a
//!   count ("a", "up to two", "any number of", "all", "a random", "X"), an "and/or" list
//!   ("a creature card and/or a land card": one of each), and a destination ("into your
//!   hand", "onto the battlefield tapped and attacking", "onto the battlefield with a
//!   shield counter on it", "on top of your library in any order", ...).
//! * Rest step ([`DigStep::Rest`]): "put the rest [destination]", "the rest of the
//!   revealed cards", "all other cards revealed this way", "all revealed cards not cast
//!   this way", "the revealed cards", "shuffle the rest into your library".
//!
//! Each step is a sentence of its own (or a part of one joined by "and" / ", then"), so
//! conditions ("If you do, ...", "If ~ was bargained, ...") and other instructions
//! between the steps compose with them.

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "dig: a step on the cards dug", priority: 200, parse: step_sentence }
}
inventory::submit! {
    EffectPattern { name: "dig: look at / reveal / exile N cards from the top", priority: 200, parse: source_sentence }
}
inventory::submit! {
    FollowupPattern { name: "dig: a step on the cards dug (after the source)", priority: 200, apply: step_followup }
}

/// Marks (in `Builder::named`) that the text has dug cards ("from among them" has an
/// antecedent).
const DIG_MARK: &str = "\u{1}dig";

/// What kind of dig the effect ends with: `Some(true)` for revealing cards until some are
/// found (the cards found aren't part of "the rest"), `Some(false)` for other digs.
fn source_kind(e: &Effect) -> Option<bool> {
    match e {
        Effect::Dig { .. }
        | Effect::Mill { .. }
        | Effect::Exile {
            what: Sel::TopOfLibrary(..),
            ..
        } => Some(false),
        Effect::RevealUntil { .. } => Some(true),
        Effect::DigStep(s) => matches!(**s, DigStep::Until { .. }).then_some(true),
        Effect::Seq(v) => v.iter().rev().find_map(source_kind),
        Effect::May { effect, .. } => source_kind(effect),
        Effect::If {
            then, otherwise, ..
        } => source_kind(then).or_else(|| source_kind(otherwise)),
        _ => None,
    }
}

/// Whether the effect ends with a dig or a dig step (so the next sentence may continue
/// it).
fn ends_with_dig(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_with_dig),
        Effect::May { effect, .. } => ends_with_dig(effect),
        Effect::If {
            then, otherwise, ..
        } => ends_with_dig(then) || ends_with_dig(otherwise),
        other => matches!(other, Effect::DigStep(_)) || source_kind(other).is_some(),
    }
}

/// Called after each sentence of an effect text (see `oracle::effects`): notes that cards
/// were dug, and what "the rest" of them is.
pub fn note_source(e: Option<&Effect>, b: &mut Builder) {
    let Some(until) = e.and_then(source_kind) else {
        return;
    };
    let rest = if until {
        // "Reveal cards ... until you reveal a creature card. You may cast that card ...
        // Put the rest ...": the cards revealed other than the ones found.
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::DUG))),
            Filter::not(Filter::In(Box::new(Sel::Var(vars::DUG_FOUND)))),
        ]))
    } else {
        Sel::Var(vars::DUG)
    };
    b.named.retain(|(n, _)| n != DIG_MARK);
    b.named.push((DIG_MARK.to_string(), rest));
}

fn dug(b: &Builder) -> bool {
    b.named.iter().any(|(n, _)| n == DIG_MARK)
}

fn dug_sel() -> Sel {
    Sel::Var(vars::DUG)
}

/// What "the rest" means (see [`note_source`]).
fn rest_sel(b: &Builder) -> Sel {
    b.named
        .iter()
        .find(|(n, _)| n == DIG_MARK)
        .map(|(_, s)| s.clone())
        .unwrap_or_else(dug_sel)
}

/// "from among them" and the like: the cards dug.
fn among_ref(s: &str) -> Option<&str> {
    for p in [
        "them",
        "those cards",
        "the milled cards",
        "the cards milled this way",
        "the revealed cards",
        "the cards revealed this way",
        "the exiled cards",
        "the cards exiled this way",
        "those exiled cards",
        "the looked-at cards",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',']) {
                return Some(r);
            }
        }
    }
    None
}

/// Library positions: "on top of your library", "back on top of that player's library in
/// any order", "on the bottom of your library in a random order", "on the bottom in any
/// order", "back".
fn library_dest(s: &str) -> Option<(Destination, &str)> {
    let (top, r) = if let Some(r) = ["back on top of ", "on top of "]
        .iter()
        .find_map(|p| s.strip_prefix(p))
    {
        (true, r)
    } else if let Some(r) = s.strip_prefix("on the bottom") {
        (false, r)
    } else if let Some(r) = s.strip_prefix("back") {
        // "put one back", "put the rest back in any order": on top.
        let r2 = r.strip_prefix(" on top").unwrap_or(r);
        let mut d = Destination::library_top();
        d.position = LibraryPosition::Top;
        return Some((d, strip_order(r2).1));
    } else {
        return None;
    };
    // "of your library", "of that player's library", "of their library", "of that
    // library", "of the library", "of their owners' libraries", or nothing ("on the
    // bottom in a random order").
    let r = if top {
        library_of(r)?
    } else {
        match r.strip_prefix(" of ") {
            Some(x) => library_of(x)?,
            None => r,
        }
    };
    let (random, r) = strip_order(r);
    let mut d = if top {
        Destination::library_top()
    } else {
        Destination::library_bottom()
    };
    if random {
        if top {
            return None;
        }
        d.position = LibraryPosition::BottomRandom;
    }
    Some((d, r))
}

fn library_of(s: &str) -> Option<&str> {
    [
        "your library",
        "that player's library",
        "their library",
        "that library",
        "the library",
        "their owners' libraries",
        "its owner's library",
    ]
    .iter()
    .find_map(|p| s.strip_prefix(p))
}

/// " in any order" / " in a random order" (whether random).
fn strip_order(s: &str) -> (bool, &str) {
    if let Some(r) = s.strip_prefix(" in any order") {
        (false, r)
    } else if let Some(r) = s.strip_prefix(" in a random order") {
        (true, r)
    } else {
        (false, s)
    }
}

/// Where cards go: "into your hand", "into your graveyard", "onto the battlefield tapped
/// and attacking under your control with a shield counter on it", a library position.
fn destination(s: &str) -> Option<(Destination, &str)> {
    let s = s.trim_start();
    for p in [
        "into your hand",
        "in your hand",
        "into their owner's hand",
        "into its owner's hand",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((Destination::zone(ZoneKind::Hand), r));
        }
    }
    for p in [
        "into your graveyard",
        "into that player's graveyard",
        "into their graveyard",
        "into their owners' graveyards",
        "into its owner's graveyard",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((Destination::zone(ZoneKind::Graveyard), r));
        }
    }
    if let Some(mut r) = s.strip_prefix("onto the battlefield") {
        let mut d = Destination::battlefield().under_your_control();
        if let Some(x) = r.strip_prefix(" tapped") {
            d.tapped = true;
            r = x;
        }
        if let Some(x) = r.strip_prefix(" and attacking") {
            if !d.tapped {
                return None;
            }
            d.attacking = true;
            r = x;
        }
        if let Some(x) = r.strip_prefix(" under your control") {
            r = x;
        }
        // "with a shield counter on it", "with an indestructible counter on it".
        if let Some(x) = r
            .strip_prefix(" with a ")
            .or_else(|| r.strip_prefix(" with an "))
        {
            let (k, x) = crate::oracle::costs::counter_kind(x)?;
            let x = x.trim_start().strip_prefix("counter on it")?;
            d.with_counters.push((k, Value::c(1)));
            r = x;
        }
        return Some((d, r));
    }
    library_dest(s)
}

/// A counted description: "a land card", "up to two creature cards", "any number of
/// creature and/or land cards", "all artifact cards", "a random creature card", "X cards",
/// "a creature card and/or a land card".
struct Counted {
    filter: Filter,
    each_of: Vec<Filter>,
    count: Option<Value>,
    up_to: bool,
    random: bool,
    single: bool,
}

fn counted(s: &str, may: bool, b: &mut Builder) -> Option<Counted> {
    let s = s.trim();
    // "a creature card and/or a land card", "an angel card, a demon card, and/or a dragon
    // card": one of each.
    if s.contains(" and/or a") {
        let parts: Vec<&str> = s
            .split(", and/or ")
            .flat_map(|p| p.split(" and/or "))
            .flat_map(|p| p.split(", "))
            .collect();
        if parts.len() >= 2 && parts.iter().all(|p| p.starts_with("a ") || p.starts_with("an ")) {
            let mut each = Vec::new();
            for p in &parts {
                let d = p.split_once(' ')?.1;
                each.push(card_filter(d, b)?);
            }
            return Some(Counted {
                filter: Filter::Any,
                count: Some(Value::c(each.len() as i32)),
                each_of: each,
                up_to: may,
                random: false,
                single: false,
            });
        }
    }
    let (count, up_to, random, desc) = if let Some(r) = s.strip_prefix("any number of ") {
        (Some(Value::c(999)), true, false, r)
    } else if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        (Some(n), true, false, r)
    } else if let Some(r) = s.strip_prefix("all ") {
        if may {
            return None;
        }
        (None, false, false, r)
    } else if let Some(r) = s
        .strip_prefix("a random ")
        .or_else(|| s.strip_prefix("one random "))
    {
        (Some(Value::c(1)), false, true, r)
    } else {
        let (n, r) = parse_number(s)?;
        (Some(n), may, false, r)
    };
    let desc = desc.trim();
    let filter = if matches!(desc, "card" | "cards") {
        Filter::Card
    } else {
        card_filter(desc, b)?
    };
    let single = matches!(count, Some(Value::Const(1)));
    Some(Counted {
        filter,
        each_of: vec![],
        count,
        up_to,
        random,
        single,
    })
}

/// "[count] of them", "[count] of those cards", "one", "any number of them".
fn counted_of_them(s: &str, may: bool) -> Option<(Counted, &str)> {
    let (count, up_to, r) = if let Some(r) = s.strip_prefix("any number ") {
        (Value::c(999), true, r)
    } else if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        (n, true, r)
    } else {
        let (n, r) = parse_number(s)?;
        (n, may, r)
    };
    let r = r.trim_start();
    let r = r
        .strip_prefix("of them")
        .or_else(|| r.strip_prefix("of those cards"))
        .or_else(|| r.strip_prefix("of the revealed cards"))?;
    let single = matches!(count, Value::Const(1));
    Some((
        Counted {
            filter: Filter::Any,
            each_of: vec![],
            count: Some(count),
            up_to,
            random: false,
            single,
        },
        r,
    ))
}

/// Pronouns for the chosen cards after "and put": "it", "them", "that card", "those
/// cards", "the revealed cards".
fn strip_chosen_pronoun(s: &str) -> Option<&str> {
    [
        "it ",
        "them ",
        "that card ",
        "those cards ",
        "the revealed cards ",
        "the revealed card ",
    ]
    .iter()
    .find_map(|p| s.strip_prefix(p))
}

/// What the rest is called: "the rest", "the rest of the revealed cards", "the rest of
/// the cards", "all other cards revealed this way", "all revealed cards not cast this
/// way", "the other cards exiled this way", "the revealed cards", "all cards revealed this
/// way", "the other".
fn rest_noun(s: &str) -> Option<(bool, &str)> {
    for p in [
        "the rest of the revealed cards",
        "the rest of the cards",
        "the rest of the exiled cards",
        "the rest",
        "all other cards revealed this way",
        "all other revealed cards",
        "the other cards revealed this way",
        "the other cards exiled this way",
        "all other cards exiled this way",
        "all revealed cards not cast this way",
        "all cards revealed this way that weren't put onto the battlefield",
        "all cards revealed this way",
        "the revealed cards",
        "the revealed card",
        "the milled cards",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((false, r));
        }
    }
    s.strip_prefix("the other").map(|r| (true, r))
}

/// The rest's destination ("on the bottom of your library in a random order", "into your
/// graveyard", "back in any order", "into your hand").
fn rest_step(dest: &str, rest: &Sel) -> Option<Effect> {
    let (to, tail) = destination(dest)?;
    if !tail.is_empty() {
        return None;
    }
    Some(Effect::DigStep(Box::new(DigStep::Rest {
        from: rest.clone(),
        to,
    })))
}

/// "[then] put the rest ...", "shuffle the rest into your library", "exile the rest".
fn parse_rest(l: &str, rest: &Sel) -> Option<Effect> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    if let Some(r) = l.strip_prefix("put ") {
        let (_, r) = rest_noun(r)?;
        return rest_step(r.trim_start(), rest);
    }
    if let Some(r) = l.strip_prefix("shuffle ") {
        let (_, r) = rest_noun(r)?;
        if r != " into your library" {
            return None;
        }
        let mut to = Destination::library_top();
        to.position = LibraryPosition::Shuffled;
        return Some(Effect::DigStep(Box::new(DigStep::Rest {
            from: rest.clone(),
            to,
        })));
    }
    if let Some(r) = l.strip_prefix("exile ") {
        let (_, r) = rest_noun(r)?;
        if !r.is_empty() {
            return None;
        }
        return Some(Effect::DigStep(Box::new(DigStep::Rest {
            from: rest.clone(),
            to: Destination::zone(ZoneKind::Exile),
        })));
    }
    None
}

/// "and the rest [destination]" / ", then put the rest [destination]" / " and put the rest
/// ..." / ", then shuffle" after a selection.
fn rest_tail(s: &str, rest: &Sel) -> Option<Vec<Effect>> {
    if s.is_empty() {
        return Some(vec![]);
    }
    if s == ", then shuffle" || s == " and shuffle" {
        return Some(vec![Effect::Shuffle {
            who: PlayerRef::You,
        }]);
    }
    for sep in [" and ", ", then ", ", and "] {
        if let Some(r) = s.strip_prefix(sep) {
            if let Some(e) = parse_rest(r, rest) {
                return Some(vec![e]);
            }
            // "and the rest into your graveyard" (the verb is the selection's).
            if let Some((_, r2)) = rest_noun(r) {
                if let Some(e) = rest_step(r2.trim_start(), rest) {
                    return Some(vec![e]);
                }
            }
        }
    }
    None
}

/// A selection step, possibly followed by the rest ("Put a land card from among them
/// onto the battlefield and the rest into your graveyard.").
fn parse_take(l: &str, b: &mut Builder) -> Option<Vec<Effect>> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (may, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let (verb, r) = ["put ", "reveal ", "exile ", "return ", "choose "]
        .iter()
        .find_map(|v| r.strip_prefix(v).map(|r| (v.trim(), r)))?;
    // The count and description, and what follows the reference to the cards.
    let (c, after) = if let Some((c, after)) = counted_of_them(r, may) {
        (c, after)
    } else {
        let (desc, after) = r.split_once(" from among ")?;
        let after = among_ref(after)?;
        (counted(desc, may, b)?, after)
    };
    // "choose from among them a card with flying, ..." isn't this.
    let (reveal, to, tail) = match verb {
        "reveal" => {
            if after.is_empty() || after.starts_with(',') {
                let mut d = Destination::library_top();
                d.position = LibraryPosition::FromTop(0);
                (true, d, after)
            } else {
                let r = after.strip_prefix(" and put ")?;
                let r = strip_chosen_pronoun(r)?;
                let (d, t) = destination(r)?;
                (true, d, t)
            }
        }
        "choose" => {
            let mut d = Destination::library_top();
            d.position = LibraryPosition::FromTop(0);
            (false, d, after)
        }
        "exile" => {
            let mut d = Destination::zone(ZoneKind::Exile);
            if let Some(r) = after.strip_prefix(" face down") {
                d.face_down = true;
                (false, d, r)
            } else {
                (false, d, after)
            }
        }
        "return" => {
            let r = after.trim_start();
            if let Some(t) = r.strip_prefix("to your hand") {
                (false, Destination::zone(ZoneKind::Hand), t)
            } else if let Some(t) = r.strip_prefix("to the battlefield") {
                (false, Destination::battlefield().under_your_control(), t)
            } else {
                return None;
            }
        }
        _ => {
            let (d, t) = destination(after)?;
            (false, d, t)
        }
    };
    // A revealed card goes to a hidden zone or stays in the library (CR 701.20a); "put
    // all ... onto the battlefield" needs no reveal.
    if reveal && matches!(to.zone, ZoneKind::Battlefield | ZoneKind::Graveyard) {
        return None;
    }
    let _ = c.single;
    let mut out = vec![Effect::DigStep(Box::new(DigStep::Take {
        from: dug_sel(),
        chooser: PlayerRef::You,
        filter: c.filter,
        each_of: c.each_of,
        count: c.count,
        up_to: c.up_to,
        random: c.random,
        reveal,
        to,
    }))];
    out.extend(rest_tail(tail, &rest_sel(b))?);
    Some(out)
}

/// "Put that card onto the battlefield and the rest on the bottom of your library in a
/// random order", "Put those land cards onto the battlefield tapped and the rest ...",
/// "put the revealed cards into your hand, then shuffle": the card(s) found or chosen.
fn parse_put_found(l: &str, b: &Builder) -> Option<Vec<Effect>> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("put ")?;
    let (from, r) = if let Some(r) = r
        .strip_prefix("that card ")
        .or_else(|| r.strip_prefix("it "))
    {
        (Sel::Var(vars::IT), r)
    } else if let Some(r) = [
        "those land cards ",
        "those permanent cards ",
        "those creature cards ",
        "those nonland cards ",
        "those cards ",
        "the nonland cards revealed this way ",
    ]
    .iter()
    .find_map(|p| r.strip_prefix(p))
    {
        (Sel::Var(vars::DUG_FOUND), r)
    } else if let Some(r) = ["the revealed cards ", "the chosen cards ", "the revealed card "]
        .iter()
        .find_map(|p| r.strip_prefix(p))
    {
        (Sel::Var(vars::DUG_CHOSEN), r)
    } else {
        return None;
    };
    let (to, tail) = destination(r)?;
    // "Put it into your hand." alone is an ordinary instruction.
    if tail.is_empty() && matches!(from, Sel::Var(vars::IT)) {
        return None;
    }
    let mut out = vec![Effect::DigStep(Box::new(DigStep::Take {
        from,
        chooser: PlayerRef::You,
        filter: Filter::Any,
        each_of: vec![],
        count: None,
        up_to: false,
        random: false,
        reveal: false,
        to,
    }))];
    out.extend(rest_tail(tail, &rest_sel(b))?);
    Some(out)
}

/// One step: a selection, the found cards, or the rest.
fn parse_step(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let v = parse_take(l, b)
        .or_else(|| parse_put_found(l, b))
        .or_else(|| parse_rest(l, &rest_sel(b)).map(|e| vec![e]))?;
    Some(Effect::seq(v))
}

fn step_sentence(l: &str, b: &mut Builder) -> Option<Effect> {
    if !dug(b) {
        return None;
    }
    let e = parse_step(l, b)?;
    b.it = Sel::Var(vars::IT);
    Some(e)
}

fn step_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_dig(prev) {
        return false;
    }
    let saved = b.named.clone();
    note_source(Some(prev), b);
    let Some(e) = parse_step(l, b) else {
        b.named = saved;
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    b.it = Sel::Var(vars::IT);
    note_source(Some(prev), b);
    true
}

/// "look at that many cards from the top of your library", "reveal a number of cards
/// from the top of your library equal to ...", "exile the top N cards of your library",
/// "reveal cards from the top of your library until you reveal two land cards", "look at
/// the top X plus one cards of your library".
fn source_sentence(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(e) = until_source(l, b) {
        return Some(e);
    }
    let (verb, r) = ["look at ", "reveal ", "exile "]
        .iter()
        .find_map(|v| l.strip_prefix(v).map(|r| (v.trim(), r)))?;
    let saved = b.targets.len();
    let (n, r) = if let Some(r) = r.strip_prefix("the top ") {
        let (n, r) = super::value_grammar::parse_value(r, b)?;
        let r = r.strip_prefix(" cards of ")?;
        (n, r.to_string())
    } else if let Some(r) = r.strip_prefix("a number of cards from the top of ") {
        let (lib, rest) = r.split_once(" equal to ")?;
        let (n, tail) = super::value_grammar::parse_value(rest, b)?;
        if !tail.trim().is_empty() {
            b.targets.truncate(saved);
            return None;
        }
        (n, lib.to_string())
    } else {
        let (n, r) = super::value_grammar::parse_value(r, b)?;
        let r = r.strip_prefix(" cards from the top of ")?;
        (n, r.to_string())
    };
    // Only numbers that existing forms don't already read (`card_flow_dig`).
    if verb != "exile" && (n.as_const().is_some() || matches!(n, Value::X)) && l.contains(" the top ")
    {
        b.targets.truncate(saved);
        return None;
    }
    let who = match r.as_str() {
        "your library" => PlayerRef::You,
        _ => {
            b.targets.truncate(saved);
            return None;
        }
    };
    b.it = Sel::Var(vars::IT);
    Some(match verb {
        "exile" => Effect::Exile {
            what: Sel::TopOfLibrary(who, n),
            face_down: false,
            link: false,
        },
        _ => Effect::Dig {
            who,
            n,
            reveal: verb == "reveal",
            filter: Filter::Any,
            take: Value::c(0),
            take_up_to: true,
            take_to: Destination::zone(ZoneKind::Hand),
            rest_to: {
                let mut d = Destination::library_top();
                d.position = LibraryPosition::FromTop(0);
                d
            },
        },
    })
}

/// "reveal cards from the top of your library until you reveal two land cards", "exile
/// cards from the top of your library until you exile X permanent cards, where X is ...".
fn until_source(l: &str, b: &mut Builder) -> Option<Effect> {
    let (exile, r) = if let Some(r) =
        l.strip_prefix("reveal cards from the top of your library until you reveal ")
    {
        (false, r)
    } else if let Some(r) =
        l.strip_prefix("exile cards from the top of your library until you exile ")
    {
        (true, r)
    } else {
        return None;
    };
    // A single card is `card_flow_reveal_until`'s.
    if r.starts_with("a ") || r.starts_with("an ") {
        return None;
    }
    let saved = b.targets.len();
    let (n, desc) = if let Some(r) = r.strip_prefix("that many ") {
        (Value::Prev, r.to_string())
    } else if let Some(r) = r.strip_prefix("a number of ") {
        let (desc, v) = r.split_once(" equal to ")?;
        let (n, tail) = super::value_grammar::parse_value(v, b)?;
        if !tail.trim().is_empty() {
            b.targets.truncate(saved);
            return None;
        }
        (n, desc.to_string())
    } else {
        let (n, r) = parse_number(r)?;
        (n, r.to_string())
    };
    let Some(filter) = card_filter(&desc, b) else {
        b.targets.truncate(saved);
        return None;
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::DigStep(Box::new(DigStep::Until {
        who: PlayerRef::You,
        filter: Filter::and(vec![Filter::Card, filter]),
        count: n,
        exile,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destinations() {
        for (s, zone) in [
            ("into your hand", ZoneKind::Hand),
            ("onto the battlefield tapped and attacking", ZoneKind::Battlefield),
            ("onto the battlefield with a shield counter on it", ZoneKind::Battlefield),
            ("on the bottom in a random order", ZoneKind::Library),
            ("back on top of that player's library in any order", ZoneKind::Library),
            ("on the bottom of your library in any order", ZoneKind::Library),
            ("into that player's graveyard", ZoneKind::Graveyard),
        ] {
            let (d, r) = destination(s).unwrap_or_else(|| panic!("{s}"));
            assert_eq!(d.zone, zone, "{s}");
            assert!(r.is_empty(), "{s}: {r}");
        }
        let (d, _) = destination("on the bottom of your library in a random order").unwrap();
        assert_eq!(d.position, LibraryPosition::BottomRandom);
        let (d, _) = destination("onto the battlefield tapped and attacking").unwrap();
        assert!(d.tapped && d.attacking);
        assert!(destination("onto the battlefield and attacking").is_none());
    }

    #[test]
    fn rests() {
        for s in [
            "put the rest into your graveyard",
            "then put the rest on the bottom of your library in a random order",
            "put the rest of the revealed cards on the bottom of your library in any order",
            "put all other cards revealed this way into your graveyard",
            "put all revealed cards not cast this way on the bottom of your library in a random order",
            "shuffle the rest into your library",
            "put the rest back in any order",
            "put the rest on the bottom in a random order",
        ] {
            assert!(parse_rest(s, &dug_sel()).is_some(), "{s}");
        }
        assert!(parse_rest("put the rest into your library third from the top", &dug_sel()).is_none());
    }
}
