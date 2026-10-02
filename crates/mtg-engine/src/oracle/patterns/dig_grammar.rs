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
inventory::submit! {
    crate::oracle::patterns::ConditionPattern { name: "dig: if you put [cards] into your hand this way", priority: 200, parse: put_this_way }
}
inventory::submit! {
    FollowupPattern { name: "dig: you may play the cards exiled from the top", priority: 200, apply: may_play_dug }
}
inventory::submit! {
    EffectPattern { name: "dig: [source], [steps] / [source] and [instruction]", priority: 200, parse: source_then_steps }
}
inventory::submit! {
    EffectPattern { name: "dig: mill that many cards", priority: 200, parse: mill_that_many }
}
inventory::submit! {
    FollowupPattern { name: "dig: otherwise, you may put that card ...", priority: 200, apply: otherwise_put_untaken }
}

/// "Otherwise, you may put that card on the bottom of your library." after "Reveal the
/// top card of your library. If it's a creature card, put it onto the battlefield.": the
/// card if it wasn't taken.
fn otherwise_put_untaken(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("otherwise, ") else {
        return false;
    };
    let (may, r) = match r.strip_prefix("you may ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let Some(r) = r
        .strip_prefix("put that card ")
        .or_else(|| r.strip_prefix("put it "))
    else {
        return false;
    };
    let Some((to, "")) = destination(r) else {
        return false;
    };
    let taken_one = match prev {
        Effect::Dig {
            n: Value::Const(1),
            take: Value::Const(1),
            take_up_to: false,
            take_to,
            rest_to,
            ..
        } => crate::dig_steps::in_place(rest_to) && take_to.zone != to.zone,
        _ => false,
    };
    if !taken_one {
        return false;
    }
    let step = Effect::DigStep(Box::new(DigStep::Rest {
        from: dug_sel(),
        to,
    }));
    let step = if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(step),
        }
    } else {
        step
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, step]);
    true
}

/// "you may mill that many cards", "you mill that many cards" in a triggered ability: the
/// triggering event's amount ("Whenever ~ deals combat damage to a player").
fn mill_that_many(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    if l != "mill that many cards" || !b.in_trigger {
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Mill {
        who: PlayerRef::You,
        n: Value::EventAmount,
    })
}

/// Whether the effect ends by exiling the top cards of a library (perhaps optionally).
fn ends_exiling_top(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(..),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_exiling_top),
        Effect::May { effect, .. } => ends_exiling_top(effect),
        _ => false,
    }
}

/// "You may play those cards this turn", "You may play those cards until the end of your
/// next turn", "Until end of turn, you may cast spells from among those exiled cards"
/// after exiling cards from the top of your library ("you may exile that many cards from
/// the top of your library"): permissions for the exiled cards.
fn may_play_dug(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let (duration, spells_only) = match l {
        "you may play those cards this turn" | "until end of turn, you may play those cards" => {
            (Duration::EndOfTurn, false)
        }
        "you may play those cards until the end of your next turn"
        | "until the end of your next turn, you may play those cards" => {
            (Duration::UntilEndOfYourNextTurn, false)
        }
        "until end of turn, you may cast spells from among those exiled cards"
        | "until end of turn, you may cast spells from among those cards"
        | "you may cast spells from among those cards this turn" => (Duration::EndOfTurn, true),
        _ => return false,
    };
    if !ends_exiling_top(prev) {
        return false;
    }
    let grant = Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: Sel::Var(vars::DUG),
        duration,
        free: false,
    };
    let grant = if spells_only { grant.cast_only() } else { grant };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, grant]);
    true
}

/// "Look at the top ten cards of your library, exile up to two creature cards from among
/// them, then shuffle.", "Reveal the top five cards of your library and separate them
/// into two piles.": a dig and what's done with the cards in one sentence.
fn source_then_steps(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for sep in [", ", " and "] {
        let mut from = 0;
        while let Some(i) = l[from..].find(sep).map(|i| i + from) {
            from = i + sep.len();
            let (a, c) = (&l[..i], &l[i + sep.len()..]);
            if c.starts_with("then ") && sep == ", " {
                continue;
            }
            let saved = (b.targets.len(), b.it.clone(), b.named.clone());
            let Some(ea) = crate::oracle::effects::parse_simple(a, b) else {
                (b.targets.truncate(saved.0), b.it = saved.1.clone(), b.named = saved.2.clone());
                continue;
            };
            if !ends_with_dig(&ea) {
                b.targets.truncate(saved.0);
                (b.it, b.named) = (saved.1, saved.2);
                continue;
            }
            note_source(Some(&ea), b);
            b.it = Sel::Var(vars::DUG);
            let ec = if sep == ", " {
                parse_step(c, b)
            } else {
                crate::oracle::effects::parse_simple(c, b)
            };
            match ec {
                Some(ec) => return Some(Effect::seq(vec![ea, ec])),
                None => {
                    b.targets.truncate(saved.0);
                    (b.it, b.named) = (saved.1, saved.2);
                }
            }
        }
    }
    None
}

/// "if you didn't put a card into your hand this way", "if you put no cards into your
/// hand this way", "if you put a Town card into your hand this way": about the cards the
/// previous instruction chose ([`DigStep::Take`] sets whether it chose any and `vars::IT`
/// to them).
fn put_this_way(c: &str) -> Option<Condition> {
    let c = end(c);
    if matches!(
        c,
        "you didn't put a card into your hand this way"
            | "you don't put a card into your hand this way"
            | "you put no cards into your hand this way"
            | "you didn't put a card into your hand"
    ) {
        return Some(Condition::Not(Box::new(Condition::PrevHappened)));
    }
    let desc = c
        .strip_prefix("you put ")?
        .strip_suffix(" into your hand this way")?;
    let desc = desc
        .strip_prefix("a ")
        .or_else(|| desc.strip_prefix("an "))?;
    let (f, _, rest) = parse_object_phrase(desc)?;
    if !rest.trim().is_empty() || !desc.ends_with(" card") {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        Filter::In(Box::new(Sel::Var(vars::IT))),
        f,
    ])))
}

/// Marks (in `Builder::named`) that the text has dug cards ("from among them" has an
/// antecedent).
const DIG_MARK: &str = "\u{1}dig";
/// Marks that cards were chosen (and revealed) from among them and left where they are
/// ("the revealed cards").
const CHOSEN_MARK: &str = "\u{1}dig-chosen";

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
pub(super) fn destination(s: &str) -> Option<(Destination, &str)> {
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

/// A card description, also an "or" list of kinds the shared object grammar doesn't read
/// as one phrase ("a blue or artifact card", "a Knight, Aura, Equipment, or legendary
/// artifact card", "land and/or legendary permanent cards with mana value X or less"):
/// each kind with the qualifiers that follow "card(s)", and "with mana value N or M".
fn dig_card_filter(desc: &str, b: &mut Builder) -> Option<Filter> {
    let desc = desc.trim();
    if matches!(desc, "card" | "cards") {
        return Some(Filter::Card);
    }
    if let Some(f) = card_filter(desc, b) {
        return Some(f);
    }
    // "[kinds] card(s)[qualifiers]".
    let (kinds, rest) = desc
        .split_once(" cards")
        .or_else(|| desc.split_once(" card"))?;
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    let noun = if desc.contains(" cards") { "cards" } else { "card" };
    // "with mana value 2 or 3".
    let (rest, mv_alts) = match rest.strip_prefix(" with mana value ") {
        Some(r) => match r.split_once(" or ") {
            Some((a, c)) if a.parse::<i32>().is_ok() && c.parse::<i32>().is_ok() => (
                "",
                Some(vec![
                    Filter::ManaValue(Cmp::Eq, Box::new(Value::c(a.parse().ok()?))),
                    Filter::ManaValue(Cmp::Eq, Box::new(Value::c(c.parse().ok()?))),
                ]),
            ),
            _ => (rest, None),
        },
        None => (rest, None),
    };
    let mut alts = Vec::new();
    for part in kinds
        .split(", or ")
        .flat_map(|p| p.split(", and/or "))
        .flat_map(|p| p.split(" and/or "))
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
    {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        if part == "double-faced" && rest.is_empty() {
            alts.push(Filter::and(vec![
                Filter::Card,
                Filter::Custom(crate::kw::dig_filters::DOUBLE_FACED.into()),
            ]));
            continue;
        }
        alts.push(card_filter(&format!("{part} {noun}{rest}"), b)?);
    }
    let kinds = match alts.len() {
        0 => return None,
        1 => alts.pop()?,
        _ => Filter::Or(alts),
    };
    Some(match mv_alts {
        Some(m) if alts_ok(&kinds) => Filter::and(vec![kinds, Filter::Or(m)]),
        Some(_) => return None,
        None if kinds_is_list(desc) => kinds,
        None => return None,
    })
}

fn alts_ok(_f: &Filter) -> bool {
    true
}

/// Whether the description is a list of kinds (not one the grammar rejected for another
/// reason).
fn kinds_is_list(desc: &str) -> bool {
    desc.contains(" or ") || desc.contains(" and/or ") || desc.contains(", ")
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
                each.push(dig_card_filter(d, b)?);
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
    let filter = dig_card_filter(desc, b)?;
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
    let r = match r
        .strip_prefix("of them")
        .or_else(|| r.strip_prefix("of those cards"))
        .or_else(|| r.strip_prefix("of the revealed cards"))
    {
        Some(x) => x,
        // "Put one into your hand and exile the rest", "Put one back": a number word.
        None if !s.starts_with("a ") && !s.starts_with("an ") && !s.starts_with("any ") => {
            let r = format!(" {r}");
            let off = s.len() - r.len();
            &s[off..]
        }
        None => return None,
    };
    // "Exile four of them at random."
    let (random, r) = match r.strip_prefix(" at random") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let single = matches!(count, Value::Const(1));
    Some((
        Counted {
            filter: Filter::Any,
            each_of: vec![],
            count: Some(count),
            up_to: up_to && !random,
            random,
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
    // The cards not chosen (nor found by revealing until).
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
        "the other",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with(' ') {
                return Some((false, r));
            }
        }
    }
    // All the cards still there.
    for p in [
        "all revealed cards not cast this way",
        "all cards revealed this way that weren't put onto the battlefield",
        "all cards revealed this way",
        "the revealed cards",
        "the revealed card",
        "the milled cards",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((true, r));
        }
    }
    None
}

/// The cards "the rest" (`all`: "all cards revealed this way") means.
fn rest_of(all: bool, rest: &Sel) -> Sel {
    if all {
        dug_sel()
    } else {
        rest.clone()
    }
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
        let (all, r) = rest_noun(r)?;
        return rest_step(r.trim_start(), &rest_of(all, rest));
    }
    if let Some(r) = l.strip_prefix("shuffle ") {
        let (all, r) = rest_noun(r)?;
        if r != " into your library" {
            return None;
        }
        let mut to = Destination::library_top();
        to.position = LibraryPosition::Shuffled;
        return Some(Effect::DigStep(Box::new(DigStep::Rest {
            from: rest_of(all, rest),
            to,
        })));
    }
    if let Some(r) = l.strip_prefix("exile ") {
        let (all, r) = rest_noun(r)?;
        if !r.is_empty() {
            return None;
        }
        return Some(Effect::DigStep(Box::new(DigStep::Rest {
            from: rest_of(all, rest),
            to: Destination::zone(ZoneKind::Exile),
        })));
    }
    None
}

/// "and the rest [destination]" / ", then put the rest [destination]" / " and put the rest
/// ..." / ", then shuffle" after a selection.
fn rest_tail(s: &str, rest: &Sel, mut builder: Option<&mut Builder>) -> Option<Vec<Effect>> {
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
            if let Some((all, r2)) = rest_noun(r) {
                if let Some(e) = rest_step(r2.trim_start(), &rest_of(all, rest)) {
                    return Some(vec![e]);
                }
            }
            // "and up to one Elf card from among them into your hand": another selection.
            if let Some(b) = builder.as_deref_mut() {
                let again = if r.starts_with("put ") || r.starts_with("exile ") {
                    r.to_string()
                } else {
                    format!("put {r}")
                };
                if let Some(v) = parse_take(&again, b) {
                    return Some(v);
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
    let mut from = dug_sel();
    let (c, after) = if let Some((desc, after)) = r.split_once(" from the chosen pile") {
        // "Put a card from the chosen pile into your hand" (CR 700.3).
        from = Sel::Var(crate::piles::CHOSEN);
        (counted(desc, may, b)?, after)
    } else if let Some((desc, after)) = r.split_once(" from among ") {
        let after = among_ref(after)?;
        (counted(desc, may, b)?, after)
    } else if let Some(x) = r
        .strip_prefix("any number of those ")
        .filter(|x| x.contains(" cards"))
    {
        // "Put any number of those permanent cards onto the battlefield": the cards found.
        let (desc, after) = x.split_once(" cards")?;
        from = Sel::Var(vars::DUG_CHOSEN);
        let filter = dig_card_filter(&format!("{desc} cards"), b)?;
        (
            Counted {
                filter,
                each_of: vec![],
                count: Some(Value::c(999)),
                up_to: true,
                random: false,
                single: false,
            },
            after,
        )
    } else if let Some((desc, after)) = r
        .split_once(" revealed this way")
        .or_else(|| r.split_once(" exiled this way"))
        .or_else(|| r.split_once(" milled this way"))
    {
        // "Put all creature cards revealed this way into your hand".
        (counted(desc, may, b)?, after)
    } else {
        counted_of_them(r, may)?
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
    let chosen_in_place = crate::dig_steps::in_place(&to);
    let mut out = vec![Effect::DigStep(Box::new(DigStep::Take {
        from,
        chooser: PlayerRef::You,
        filter: c.filter,
        each_of: c.each_of,
        count: c.count,
        up_to: c.up_to,
        random: c.random,
        reveal,
        to,
    }))];
    let rest = rest_sel(b);
    out.extend(rest_tail(tail, &rest, Some(b))?);
    if chosen_in_place && !b.named.iter().any(|(n, _)| n == CHOSEN_MARK) {
        b.named.push((CHOSEN_MARK.to_string(), Sel::Var(vars::DUG_CHOSEN)));
    }
    Some(out)
}

/// "Put that card onto the battlefield and the rest on the bottom of your library in a
/// random order", "Put those land cards onto the battlefield tapped and the rest ...",
/// "put the revealed cards into your hand, then shuffle": the card(s) found or chosen.
fn parse_put_found(l: &str, b: &Builder) -> Option<Vec<Effect>> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (verb, r) = if let Some(r) = l.strip_prefix("put ") {
        ("put", r)
    } else if let Some(r) = l.strip_prefix("exile ") {
        ("exile", r)
    } else {
        return None;
    };
    let chosen_mark = b.named.iter().any(|(n, _)| n == CHOSEN_MARK);
    let (from, r) = if let Some(r) = r
        .strip_prefix("that card")
        .or_else(|| r.strip_prefix("it"))
        .filter(|r| r.is_empty() || r.starts_with(' '))
    {
        (Sel::Var(vars::IT), r)
    } else if let Some(r) = [
        "those land cards",
        "those permanent cards",
        "those creature cards",
        "those nonland cards",
        "those cards",
        "the nonland cards revealed this way",
        "the chosen cards",
    ]
    .iter()
    .find_map(|p| r.strip_prefix(p))
    {
        (Sel::Var(vars::DUG_CHOSEN), r)
    } else if let Some(r) = ["the revealed cards", "the revealed card"]
        .iter()
        .find_map(|p| r.strip_prefix(p))
    {
        // Cards revealed from among them ("You may reveal up to two creature cards from
        // among them. ... put the revealed cards into your hand."), or all the cards
        // revealed ("Each player reveals the top card of their library. You may put the
        // revealed cards into their owners' graveyards.").
        if chosen_mark {
            (Sel::Var(vars::DUG_CHOSEN), r)
        } else {
            (dug_sel(), r)
        }
    } else {
        return None;
    };
    let (to, tail) = if verb == "exile" {
        (Destination::zone(ZoneKind::Exile), r)
    } else {
        destination(r)?
    };
    // "Put it into your hand." alone is an ordinary instruction.
    if tail.is_empty() && matches!(from, Sel::Var(vars::IT)) && to.zone != ZoneKind::Library {
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
    out.extend(rest_tail(tail, &rest_sel(b), None)?);
    Some(out)
}

/// "you may reveal that card", "you may reveal it" after looking at the top card: the
/// card is revealed (or not), and stays where it is.
fn reveal_that_card(l: &str) -> Option<Vec<Effect>> {
    if !matches!(l, "reveal that card" | "reveal it") {
        return None;
    }
    let mut to = Destination::library_top();
    to.position = LibraryPosition::FromTop(0);
    Some(vec![Effect::DigStep(Box::new(DigStep::Take {
        from: dug_sel(),
        chooser: PlayerRef::You,
        filter: Filter::Any,
        each_of: vec![],
        count: Some(Value::c(1)),
        up_to: false,
        random: false,
        reveal: true,
        to,
    }))])
}

/// One step: a selection, the found cards, or the rest.
fn parse_step(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(v) = parse_take(l, b) {
        return Some(Effect::seq(v));
    }
    // "You may put that card on the bottom of your library.", "You may reveal that card."
    let (may, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let v = parse_put_found(r, b)
        .or_else(|| reveal_that_card(r))
        .or_else(|| parse_rest(r, &rest_sel(b)).map(|e| vec![e]))?;
    let e = Effect::seq(v);
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
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
    let parsed = source_amount(r, b).and_then(|(n, lib)| Some((n, library_owner(&lib, b)?)));
    let Some((n, who)) = parsed else {
        b.targets.truncate(saved);
        return None;
    };
    // An X: a spell's X, or one the text defines ("where X is ...").
    let spell_x = (b.ctx.is_spell() && !b.in_trigger) || super::value_grammar::x_defined();
    if format!("{n:?}").contains('X') && !spell_x {
        b.targets.truncate(saved);
        return None;
    }
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

/// How many cards from the top of which library: "the top X plus one cards of your
/// library", "the top card of defending player's library", "that many cards from the top
/// of your library", "a number of cards from the top of your library equal to ...".
fn source_amount(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    let that_many = |r: &str, b: &Builder| -> Option<String> {
        let x = r.strip_prefix("that many cards from the top of ")?;
        // The triggering event's amount ("deals combat damage to a player, look at that
        // many cards").
        b.in_trigger.then(|| x.to_string())
    };
    if let Some(r) = r.strip_prefix("the top card of ") {
        return Some((Value::c(1), r.to_string()));
    }
    if let Some(r) = r.strip_prefix("the top ") {
        // "the top two cards of", "the top X plus one cards of", "the top X cards of".
        if let Some((n, r)) = parse_number(r) {
            let r = r.trim_start();
            if let Some(x) = r.strip_prefix("plus one cards of ") {
                return Some((Value::Sum(vec![n, Value::c(1)]), x.to_string()));
            }
            if let Some(x) = r.strip_prefix("cards of ") {
                return Some((n, x.to_string()));
            }
        }
        let (n, r) = super::value_grammar::parse_value(r, b)?;
        let r = r.strip_prefix(" cards of ")?;
        return Some((n, r.to_string()));
    }
    if let Some(lib) = that_many(r, b) {
        return Some((Value::EventAmount, lib));
    }
    if let Some(r) = r.strip_prefix("a number of cards from the top of ") {
        let (lib, rest) = r.split_once(" equal to ")?;
        let (n, tail) = super::value_grammar::parse_value(rest, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some((n, lib.to_string()));
    }
    let (n, r) = super::value_grammar::parse_value(r, b)?;
    let r = r.strip_prefix(" cards from the top of ")?;
    Some((n, r.to_string()))
}

/// Whose library: "your library", "defending player's library", "that library" / "that
/// player's library" (the player the text is about).
fn library_owner(s: &str, b: &Builder) -> Option<PlayerRef> {
    Some(match s {
        "your library" => PlayerRef::You,
        "defending player's library" => PlayerRef::DefendingPlayer,
        "that library" | "that player's library" | "their library"
            if matches!(b.it_player, PlayerRef::TriggerPlayer | PlayerRef::Target(_)) =>
        {
            b.it_player.clone()
        }
        _ => return None,
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

    fn with_builder<T>(f: impl FnOnce(&mut Builder) -> T) -> T {
        let tl = crate::types::TypeLine::parse("Sorcery");
        let ctx = crate::oracle::CompileContext {
            card_name: "X",
            full_name: "X",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let mut b = Builder::new(&ctx);
        f(&mut b)
    }

    #[test]
    fn probe_filters() {
        with_builder(|b| {
            for d in [
                "blue or artifact card",
                "land or double-faced card",
                "land or legendary turtle card",
                "knight, aura, equipment, or legendary artifact card",
                "creature card or garruk planeswalker card",
                "land and/or legendary permanent cards with mana value x or less",
                "permanent card with mana value 3 or less",
                "artifact card with mana value 2 or 3",
                "card with {x} in its mana cost",
                "cards of the chosen type",
                "creature cards of the chosen type",
                "hero or enchantment card",
                "double-faced card",
                "nonland, nonlegendary card",
            ] {
                println!("{d} => {:?}", dig_card_filter(d, b));
            }
        });
    }

    #[test]
    fn probe_texts() {
        let Ok(path) = std::env::var("DIG_TEXTS") else {
            return;
        };
        let texts = std::fs::read_to_string(path).unwrap();
        for t in texts.lines() {
            let (trig, t) = match t.strip_prefix("T:") {
                Some(x) => (true, x),
                None => (false, t),
            };
            with_builder(|b| {
                b.in_trigger = trig;
                if trig {
                    b.it_player = PlayerRef::TriggerPlayer;
                }
                let r = crate::oracle::effects::parse_effect_text(&t.to_lowercase(), b);
                println!("{} {t}", if r.is_some() { "OK" } else { "NO" });
                if std::env::var("DIG_DEBUG").is_ok() {
                    println!("{r:?}");
                }
                if r.is_none() {
                    // Prefixes: which sentence fails.
                    let sents: Vec<&str> = t.split(". ").collect();
                    for k in 1..=sents.len() {
                        let pre = sents[..k].join(". ");
                        let ok = with_builder(|b2| {
                            b2.in_trigger = trig;
                            crate::oracle::effects::parse_effect_text(&pre.to_lowercase(), b2)
                                .is_some()
                        });
                        if !ok {
                            println!("   fails at: {}", sents[k - 1]);
                            break;
                        }
                    }
                }
            });
        }
    }
}
