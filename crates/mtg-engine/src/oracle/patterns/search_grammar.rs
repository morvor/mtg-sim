//! The search clause grammar (CR 701.23), compiled to [`Effect::SearchCards`]:
//!
//! ```text
//! [searcher] search(es) [whose] [zones] for [card specs] [, where X is VALUE]
//!     [, reveal them] [, put them DEST | , put one DEST and the other DEST | , exile them]
//!     [, then shuffle | , then shuffle and put them on top | ... third from the top]
//! ```
//!
//! * searcher: "you" (no subject), "target player", "that player", "its controller",
//!   "each opponent may", with third-person verbs ("searches ..., puts ..., then shuffles");
//! * whose: "your", "their", "target opponent's", "that player's", "its owner's";
//! * zones: "library", "library and/or graveyard", "graveyard, hand, and library", ...;
//! * card specs: a list of parts ("a Forest card and a Plains card", "a white card, a
//!   blue card, ..., and a green card", "a card named X and/or a card named Y", "a land
//!   card of each basic land type"), each with a count ("up to two", "any number of",
//!   "all", "X") and a description (object phrases, names with commas, "with mana value
//!   equal to 1 plus the sacrificed creature's mana value"); "with different names";
//! * destinations, possibly split ("put one onto the battlefield tapped and the other
//!   into your hand").
//!
//! Sentences that complete a search ("Reveal those cards, put them into your hand, then
//! shuffle.", "If you search your library this way, shuffle.", "Then that player
//! shuffles.") are follow-ups that fill in the search.

use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;
use crate::types::*;

inventory::submit! {
    // After the simpler library-search patterns (`card_flow_search`, priority 90).
    EffectPattern { name: "search grammar: search zones for card specs", priority: 95, parse: search_clause }
}
inventory::submit! {
    FollowupPattern { name: "search grammar: complete the search", priority: 90, apply: complete_search }
}

/// Who searches.
struct Searcher {
    who: PlayerRef,
    /// "their" means each searcher's own (several searchers) or the searcher's.
    their: PlayerRef,
    optional: bool,
    /// A subject other than "you": verbs are third person ("searches", "puts").
    third: bool,
}

fn searcher<'a>(l: &'a str, b: &mut Builder) -> Option<(Searcher, &'a str)> {
    if let Some(r) = l
        .strip_prefix("search ")
        .or_else(|| l.strip_prefix("you search "))
    {
        let s = Searcher {
            who: PlayerRef::You,
            their: PlayerRef::You,
            optional: false,
            third: false,
        };
        return Some((s, r));
    }
    // "[player] searches", "[player] may search", "[players] may each search".
    for (verb, optional) in [
        (" searches ", false),
        (" may search ", true),
        (" may each search ", true),
    ] {
        let Some(at) = l.find(verb) else {
            continue;
        };
        let subject = &l[..at];
        let rest = &l[at + verb.len()..];
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let parsed = player_ref(subject, b);
        let Some((who, tail)) = parsed.filter(|(_, t)| t.trim().is_empty()) else {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
            continue;
        };
        let group = matches!(
            who,
            PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
        );
        // "may each search" is for several players.
        if verb == " may each search " && !group {
            return None;
        }
        let _ = tail;
        let their = if group {
            PlayerRef::Iterated
        } else {
            who.clone()
        };
        let s = Searcher {
            third: !matches!(who, PlayerRef::You),
            who,
            their,
            optional,
        };
        return Some((s, rest));
    }
    None
}

/// "your library", "their library and/or graveyard", "target opponent's graveyard, hand,
/// and library": whose zones, which, and whether the searcher chooses among them.
fn zones<'a>(
    s: &'a str,
    sr: &Searcher,
    b: &mut Builder,
) -> Option<(PlayerRef, Vec<ZoneKind>, bool, &'a str)> {
    let (whose, r) = if let Some(r) = s.strip_prefix("your ") {
        if !matches!(sr.who, PlayerRef::You) {
            return None;
        }
        (PlayerRef::You, r)
    } else if let Some(r) = s.strip_prefix("their ") {
        if matches!(sr.who, PlayerRef::You) {
            return None;
        }
        (sr.their.clone(), r)
    } else {
        let (owner, r) = s.split_once("'s ")?;
        let (p, tail) = player_ref(owner, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (p, r)
    };
    let (list, rest) = r.split_once(" for ")?;
    let mut zones = Vec::new();
    let mut optional = false;
    let words = list
        .replace(", and/or ", " and/or ")
        .replace(", and ", " and ")
        .replace(", ", " and ");
    let mut parts: Vec<&str> = Vec::new();
    for chunk in words.split(" and/or ") {
        if parts.len() > 0 || chunk.len() < words.len() {
            optional = true;
        }
        parts.extend(chunk.split(" and "));
    }
    // "and/or" anywhere makes the choice of zones the searcher's ("graveyard, hand
    // and/or library").
    optional = optional && words.contains(" and/or ");
    for z in parts {
        let k = match z.trim() {
            "library" => ZoneKind::Library,
            "graveyard" => ZoneKind::Graveyard,
            "hand" => ZoneKind::Hand,
            _ => return None,
        };
        if zones.contains(&k) {
            return None;
        }
        zones.push(k);
    }
    Some((whose, zones, optional, rest))
}

/// A real card's name at the start of `s` (the longest one that ends at a phrase
/// boundary), or "~". Returns the name as printed and the rest.
fn card_name<'a>(s: &'a str, b: &Builder) -> Option<(String, &'a str)> {
    if let Some(r) = s.strip_prefix('~') {
        if b.ctx.card_name.is_empty() {
            return None;
        }
        return Some((b.ctx.card_name.to_string(), r));
    }
    let mut best: Option<(String, &str)> = None;
    for (i, _) in s.char_indices().chain([(s.len(), ' ')]) {
        let (head, rest) = s.split_at(i);
        if head.is_empty() || !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
            continue;
        }
        if let Some(n) = printed_name(head) {
            best = Some((n, rest));
        }
    }
    best
}

/// The printed name of a real card (or a face of one) named `lower`.
fn printed_name(lower: &str) -> Option<String> {
    let c = mtg_data::cards().by_name(lower)?;
    if c.name.eq_ignore_ascii_case(lower) {
        return Some(c.name.clone());
    }
    // A face of a multi-faced card.
    c.name
        .split(" // ")
        .find(|f| f.eq_ignore_ascii_case(lower))
        .map(str::to_string)
        .or_else(|| Some(lower.to_string()))
}

/// One card description, without its count: "basic land card", "card named Liliana,
/// Death Wielder", "card named Festering Newt or Bubbling Cauldron", "creature card with
/// mana value equal to 1 plus the sacrificed creature's mana value". Returns the filter
/// and the rest.
fn description<'a>(s: &'a str, b: &mut Builder) -> Option<(Filter, String)> {
    let s = s.trim_start();
    for p in ["card named ", "cards named "] {
        if let Some(r) = s.strip_prefix(p) {
            let (n, mut rest) = card_name(r, b)?;
            let mut names = vec![Filter::Named(n.into())];
            // "a card named Festering Newt or Bubbling Cauldron".
            while let Some(r2) = rest.strip_prefix(" or ") {
                let Some((n2, r3)) = card_name(r2, b) else {
                    break;
                };
                names.push(Filter::Named(n2.into()));
                rest = r3;
            }
            let f = if names.len() == 1 {
                names.pop().expect("one name")
            } else {
                Filter::Or(names)
            };
            return Some((f, rest.to_string()));
        }
    }
    let (f, _, rest) = parse_object_phrase(s)?;
    // The description must name cards ("basic land card", "creature cards").
    let head = &s[..s.len() - rest.len()];
    if !head.contains("card") {
        return None;
    }
    let mut fs = vec![f];
    let mut rest = rest.trim_start().to_string();
    // Qualifiers whose values need the builder ("with mana value X or less", "with mana
    // value equal to 1 plus the sacrificed creature's mana value", "with mana value 4 or
    // 5", "that have mana value 9").
    loop {
        if let Some((f2, r)) = mana_value_qualifier(&rest, b) {
            fs.push(f2);
            rest = r;
            continue;
        }
        break;
    }
    Some((Filter::and(fs), rest))
}

fn mana_value_qualifier(s: &str, b: &mut Builder) -> Option<(Filter, String)> {
    let r = s
        .strip_prefix("with mana value ")
        .or_else(|| s.strip_prefix("that have mana value "))
        .or_else(|| s.strip_prefix("that has mana value "))?;
    let mv = |c: Cmp, v: Value| Filter::ManaValue(c, Box::new(v));
    if let Some(r) = r.strip_prefix("equal to ") {
        let (v, rest) = crate::oracle::patterns::r107_numbers::value_phrase(r, b)?;
        return Some((mv(Cmp::Eq, v), rest));
    }
    if let Some(r) = r.strip_prefix("less than or equal to ") {
        let (v, rest) = crate::oracle::patterns::r107_numbers::value_phrase(r, b)?;
        return Some((mv(Cmp::Le, v), rest));
    }
    let (n, r) = parse_number(r)?;
    let r = r.trim_start();
    for (p, c) in [("or less", Cmp::Le), ("or greater", Cmp::Ge)] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((mv(c, n), rest.to_string()));
        }
    }
    // "with mana value 4 or 5".
    if let Some(r2) = r.strip_prefix("or ") {
        if let Some((m, rest)) = parse_number(r2) {
            if m.as_const().is_some() && n.as_const().is_some() {
                return Some((Filter::Or(vec![mv(Cmp::Eq, n), mv(Cmp::Eq, m)]), rest.to_string()));
            }
        }
    }
    Some((mv(Cmp::Eq, n), r.to_string()))
}

/// A part's count: "a", "an", "two", "X", "up to N", "any number of", "all", "exactly
/// N". Returns (count, up to, all, rest).
fn count(s: &str) -> Option<(Value, bool, bool, &str)> {
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((Value::c(999), true, false, r));
    }
    if let Some(r) = s.strip_prefix("all ") {
        return Some((Value::c(999), true, true, r));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        return Some((n, true, false, r.trim_start()));
    }
    let s = s.strip_prefix("exactly ").unwrap_or(s);
    let (n, r) = parse_number(s)?;
    Some((n, false, false, r.trim_start()))
}

/// The separators between parts that have their own articles.
const PART_SEPS: [(&str, Sep); 6] = [
    (", and/or ", Sep::And),
    (", and ", Sep::And),
    (" and/or ", Sep::And),
    (" and ", Sep::And),
    (", or ", Sep::Or),
    (" or ", Sep::Or),
];

#[derive(Clone, Copy, PartialEq)]
enum Sep {
    And,
    Or,
}

/// The card specs: a list of parts, each found separately ("a Forest card and a Plains
/// card"), or alternatives for one card ("a snow permanent card, a legendary card, or a
/// Saga card"). Returns the parts, whether they need different names, and the rest.
fn specs(s: &str, b: &mut Builder) -> Option<(Vec<SearchPart>, bool, String)> {
    let s = s.strip_prefix("any card").map_or(s.to_string(), |r| format!("a card{r}"));
    let mut items: Vec<(Value, bool, bool, Filter)> = Vec::new();
    let mut seps: Vec<Sep> = Vec::new();
    let mut rest: String = s;
    loop {
        let (n, up_to, all, r) = count(&rest)?;
        let (f, r) = description(r, b)?;
        items.push((n, up_to, all, f));
        let next = PART_SEPS.iter().find_map(|(p, sep)| {
            let r2 = r.strip_prefix(p)?;
            // The next part has its own article or count.
            count(r2).is_some().then(|| (*sep, r2.to_string()))
        });
        match next {
            Some((sep, r2)) => {
                seps.push(sep);
                rest = r2;
            }
            None => {
                rest = r;
                break;
            }
        }
    }
    // "with different names", "that each have different names".
    let mut distinct = false;
    for p in [
        " that each have different names",
        " that have different names",
        " with different names",
    ] {
        if let Some(r) = rest.strip_prefix(p) {
            distinct = true;
            rest = r.to_string();
            break;
        }
    }
    // "a land card of each basic land type" (CR 205.3i): one part per type.
    if let Some(r) = rest.strip_prefix(" of each basic land type") {
        if items.len() != 1 || !matches!(items[0].0, Value::Const(1)) {
            return None;
        }
        let f = items.pop()?.3;
        let parts = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .iter()
            .map(|t| SearchPart {
                filter: Filter::and(vec![f.clone(), Filter::Subtype(Subtype::new(t))]),
                count: Value::c(1),
                up_to: false,
                all: false,
            })
            .collect();
        return Some((parts, distinct, r.to_string()));
    }
    let or = seps.contains(&Sep::Or);
    if or {
        // Alternatives for one card: every part is "a"/"an" and every separator "or"
        // (the last may follow a comma list).
        if seps.iter().any(|s| *s != Sep::Or)
            || items.iter().any(|(n, up, all, _)| !matches!(n, Value::Const(1)) || *up || *all)
        {
            return None;
        }
        let filter = Filter::Or(items.into_iter().map(|i| i.3).collect());
        let part = SearchPart {
            filter,
            count: Value::c(1),
            up_to: false,
            all: false,
        };
        return Some((vec![part], distinct, rest));
    }
    let parts = items
        .into_iter()
        .map(|(count, up_to, all, filter)| SearchPart {
            filter,
            count,
            up_to,
            all,
        })
        .collect();
    Some((parts, distinct, rest))
}

const PRONOUNS: [&str; 7] = [
    "those cards",
    "that card",
    "the cards",
    "the card",
    "them",
    "it",
    "both",
];

fn pronoun(s: &str) -> Option<&str> {
    PRONOUNS.iter().find_map(|p| {
        let r = s.strip_prefix(p)?;
        (r.is_empty() || r.starts_with([' ', ','])).then_some(r)
    })
}

/// Where found cards go: "into your hand", "into their graveyard", "onto the battlefield
/// tapped under your control with a stun counter on it". Returns the destination and the
/// rest.
fn destination<'a>(s: &'a str, sr: &Searcher, b: &mut Builder) -> Option<(Destination, &'a str)> {
    for (p, z) in [
        ("into your hand", ZoneKind::Hand),
        ("into their hand", ZoneKind::Hand),
        ("into that player's hand", ZoneKind::Hand),
        ("into your graveyard", ZoneKind::Graveyard),
        ("into their graveyard", ZoneKind::Graveyard),
        ("into that player's graveyard", ZoneKind::Graveyard),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            // "your" names the searcher's own hand only when the searcher is you.
            if (p.contains("your") && !matches!(sr.who, PlayerRef::You))
                || (!p.contains("your") && matches!(sr.who, PlayerRef::You))
            {
                return None;
            }
            return Some((Destination::zone(z), r));
        }
    }
    for (p, pos) in [
        ("on top of your library", LibraryPosition::Top),
        ("on the bottom of your library", LibraryPosition::Bottom),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if !matches!(sr.who, PlayerRef::You) {
                return None;
            }
            let mut d = Destination::zone(ZoneKind::Library);
            d.position = pos;
            return Some((d, r));
        }
    }
    let mut r = s.strip_prefix("onto the battlefield")?;
    let mut d = Destination::battlefield();
    // CR 110.2a: a permanent enters under the control of the player who put it there.
    d.controller = Some(match sr.who {
        PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer => {
            PlayerRef::Iterated
        }
        ref w => w.clone(),
    });
    loop {
        if let Some(x) = r.strip_prefix(" tapped") {
            d.tapped = true;
            r = x;
        } else if let Some(x) = r.strip_prefix(" under your control") {
            d.controller = Some(PlayerRef::You);
            r = x;
        } else if let Some(x) = r.strip_prefix(" under their control") {
            if matches!(sr.who, PlayerRef::You) {
                return None;
            }
            r = x;
        } else if let Some(x) = r.strip_prefix(" under ").and_then(|x| {
            let (owner, rest) = x.split_once("'s control")?;
            Some((owner, rest))
        }) {
            let (p, tail) = player_ref(x.0, b)?;
            if !tail.trim().is_empty() {
                return None;
            }
            d.controller = Some(p);
            r = x.1;
        } else if let Some(x) = r.strip_prefix(" with ") {
            // "with an additional +1/+1 counter on it", "with X additional +1/+1 counters
            // on it", "with a stun counter on it".
            let (n, x) = parse_number(x)?;
            let x = x.trim_start();
            let x = x.strip_prefix("additional ").unwrap_or(x);
            let (kind, x) = crate::oracle::costs::counter_kind(x)?;
            let x = x
                .strip_prefix(" counters on it")
                .or_else(|| x.strip_prefix(" counter on it"))
                .or_else(|| x.strip_prefix(" counters on them"))?;
            d.with_counters.push((kind, n));
            r = x;
        } else {
            break;
        }
    }
    Some((d, r))
}

/// The instructions after the card specs: reveal, put/exile (possibly split), shuffle.
/// Fills them into `spec`; returns false unless all of `t` is understood.
fn tail(t: &str, spec: &mut SearchSpec, sr: &Searcher, b: &mut Builder) -> bool {
    tail_inner(t, spec, sr, b).is_some_and(|r| r.trim().is_empty())
}

fn tail_inner<'a>(
    t: &'a str,
    spec: &mut SearchSpec,
    sr: &Searcher,
    b: &mut Builder,
) -> Option<&'a str> {
    let mut t = t;
    // Reveal.
    for p in [", reveal ", " and reveal ", ", reveals ", " and reveals "] {
        if let Some(x) = t.strip_prefix(p) {
            t = pronoun(x)?;
            spec.reveal = true;
            break;
        }
    }
    // Put / exile.
    let put = [
        ", then put ",
        ", and put ",
        ", put ",
        " and put ",
        ", then puts ",
        ", and puts ",
        ", puts ",
        " and puts ",
    ]
    .iter()
    .find_map(|p| t.strip_prefix(p));
    if let Some(x) = put {
        t = put_dests(x, spec, sr, b)?;
    } else if let Some(x) = [", exile ", " and exile ", ", exiles ", " and exiles "]
        .iter()
        .find_map(|p| t.strip_prefix(p))
    {
        t = pronoun(x)?;
        spec.dests = vec![SearchDest {
            count: None,
            to: Destination::zone(ZoneKind::Exile),
        }];
    }
    // Shuffle.
    let shuffles = [
        ", then shuffle",
        " then shuffle",
        ", shuffle",
        ", then shuffles",
        " then shuffles",
    ];
    if let Some(x) = shuffles.iter().find_map(|p| t.strip_prefix(p)) {
        // "then shuffle and put that card on top", "... third from the top", "... on top
        // in any order" (CR 701.24b): the found cards stay in the library.
        if let Some(y) = x
            .strip_prefix(" and put ")
            .or_else(|| x.strip_prefix(" and puts "))
        {
            if !spec.dests.is_empty() {
                return None;
            }
            let y = pronoun(y)?;
            let (pos, y) = if let Some(z) = y
                .strip_prefix(" on top in any order")
                .or_else(|| y.strip_prefix(" on top"))
            {
                (LibraryPosition::Top, z)
            } else if let Some(z) = y.strip_prefix(" third from the top") {
                (LibraryPosition::FromTop(2), z)
            } else if let Some(z) = y.strip_prefix(" second from the top") {
                (LibraryPosition::FromTop(1), z)
            } else {
                return None;
            };
            if !spec.zones.iter().all(|z| *z == ZoneKind::Library) {
                return None;
            }
            let mut d = Destination::zone(ZoneKind::Library);
            d.position = pos;
            spec.dests = vec![SearchDest { count: None, to: d }];
            spec.shuffle = SearchShuffle::Before;
            return Some(y);
        }
        spec.shuffle = SearchShuffle::After;
        return Some(x);
    }
    Some(t)
}

/// "it into your hand", "them onto the battlefield tapped", "one onto the battlefield
/// tapped and the other into your hand", "two of them onto the battlefield and the rest
/// into your hand".
fn put_dests<'a>(
    x: &'a str,
    spec: &mut SearchSpec,
    sr: &Searcher,
    b: &mut Builder,
) -> Option<&'a str> {
    if let Some(r) = pronoun(x) {
        let (to, r) = destination(r.trim_start(), sr, b)?;
        spec.dests = vec![SearchDest { count: None, to }];
        return Some(r);
    }
    // Split: "[N] [of them] DEST and the other/the rest DEST".
    let (n, r) = parse_number(x)?;
    n.as_const()?;
    let r = r.trim_start();
    let r = r.strip_prefix("of them ").unwrap_or(r);
    let (first, r) = destination(r, sr, b)?;
    let r = r
        .strip_prefix(" and the other ")
        .or_else(|| r.strip_prefix(" and the rest "))
        .or_else(|| r.strip_prefix(" and the others "))?;
    let (second, r) = destination(r, sr, b)?;
    spec.dests = vec![
        SearchDest {
            count: Some(n),
            to: first,
        },
        SearchDest {
            count: None,
            to: second,
        },
    ];
    Some(r)
}

/// The action markers that end the card specs.
const ACTIONS: [&str; 14] = [
    ", where x is ",
    ", reveal ",
    " and reveal ",
    ", reveals ",
    " and reveals ",
    ", put ",
    " and put ",
    ", puts ",
    " and puts ",
    ", exile ",
    " and exile ",
    ", exiles ",
    ", then shuffle",
    ", then shuffles",
];

fn search_clause(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let out = search_clause_inner(l, b);
    if out.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    out
}

fn search_clause_inner(l: &str, b: &mut Builder) -> Option<Effect> {
    let (sr, r) = searcher(l, b)?;
    let (whose, zones, zones_optional, r) = zones(r, &sr, b)?;
    // The card specs run to the first action.
    let cut = ACTIONS
        .iter()
        .filter_map(|a| r.find(a))
        .min()
        .unwrap_or(r.len());
    let (desc, mut t) = r.split_at(cut);
    let (parts, distinct, leftover) = specs(desc, b)?;
    if !leftover.trim().is_empty() {
        return None;
    }
    // "..., where X is [value], put them ...": X is that value (CR 107.3c).
    let mut x_value: Option<Value> = None;
    let owned;
    if let Some(v) = t.strip_prefix(", where x is ") {
        let (v, rest) = crate::oracle::patterns::r107_numbers::value_phrase(v, b)?;
        x_value = Some(crate::oracle::patterns::r107_numbers::nonnegative(v));
        owned = rest;
        t = owned.as_str();
    }
    let mut spec = SearchSpec {
        who: sr.who.clone(),
        whose,
        zones,
        zones_optional,
        parts,
        distinct_names: distinct,
        optional: sr.optional,
        reveal: false,
        dests: vec![],
        shuffle: SearchShuffle::No,
    };
    if !tail(t, &mut spec, &sr, b) {
        return None;
    }
    // Third-person verbs go with a subject other than you, and only then.
    let third_verbs = [", reveals ", ", puts ", " and puts ", ", then shuffles", ", exiles "];
    if !sr.third && third_verbs.iter().any(|v| t.contains(v)) {
        return None;
    }
    if sr.third && t.contains(", then shuffle") && !t.contains(", then shuffles") {
        return None;
    }
    // "Then shuffle" names the searcher's own library: searching another player's
    // library, it's "then that player shuffles".
    if spec.shuffle != SearchShuffle::No && !own_library(&spec) {
        return None;
    }
    let e = Effect::SearchCards(Box::new(spec));
    let e = match x_value {
        Some(x) => crate::oracle::patterns::r107_numbers::substitute_x(&e, &x)?,
        None => e,
    };
    b.it = Sel::Var(vars::IT);
    Some(e)
}

/// Whether two player references are the same reference.
fn same(a: &PlayerRef, c: &PlayerRef) -> bool {
    format!("{a:?}") == format!("{c:?}")
}

/// Whether the searcher searches their own zones.
fn own_library(spec: &SearchSpec) -> bool {
    same(&spec.whose, &spec.who)
        || (matches!(spec.whose, PlayerRef::Iterated)
            && matches!(
                spec.who,
                PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
            ))
}

/// Calls `f` on the search `e` ends with, possibly optional, and on both branches of a
/// conditional ("Search for X. If ..., instead search for Y. Reveal those cards, ...").
/// Returns how many searches there were.
fn visit_searches(e: &mut Effect, f: &mut dyn FnMut(&mut SearchSpec) -> bool) -> Option<usize> {
    match e {
        Effect::SearchCards(s) => f(s).then_some(1),
        Effect::May { effect, .. } => visit_searches(effect, f),
        Effect::If {
            then, otherwise, ..
        } => {
            let a = visit_searches(then, f)?;
            let c = visit_searches(otherwise, f)?;
            // Both branches search, or the condition only adds to one ("If ..., you may
            // search for an additional card").
            Some(a + c)
        }
        Effect::Seq(v) => match v.last_mut() {
            Some(last) => visit_searches(last, f),
            None => Some(0),
        },
        Effect::Noop => Some(0),
        _ => Some(0),
    }
}

/// Applies `f` to every search `prev` ends with; changes `prev` only if there's at least
/// one and `f` succeeds on all.
fn update_searches(prev: &mut Effect, f: &mut dyn FnMut(&mut SearchSpec) -> bool) -> bool {
    let mut new = prev.clone();
    match visit_searches(&mut new, f) {
        Some(n) if n > 0 => {
            *prev = new;
            true
        }
        _ => false,
    }
}

/// Sentences that complete the search before them.
fn complete_search(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    // "If you search your library this way, shuffle." / "If they search their library
    // this way, they shuffle.": only a searched library is shuffled.
    if matches!(
        l,
        "if you search your library this way, shuffle"
            | "if they search their library this way, they shuffle"
            | "then each player who searched their library this way shuffles"
    ) {
        let you = l.starts_with("if you ");
        return update_searches(prev, &mut |s| {
            let ok = s.shuffle == SearchShuffle::No
                && s.zones.contains(&ZoneKind::Library)
                && own_library(s)
                && you == (matches!(s.who, PlayerRef::You));
            if ok {
                s.shuffle = SearchShuffle::After;
            }
            ok
        });
    }
    // "Then that player shuffles." after searching another player's library.
    if matches!(l, "then that player shuffles" | "that player shuffles") {
        let that = b.it_player.clone();
        return update_searches(prev, &mut |s| {
            let ok = s.shuffle == SearchShuffle::No
                && s.zones.contains(&ZoneKind::Library)
                && !same(&s.whose, &s.who)
                && same(&s.whose, &that);
            if ok {
                s.shuffle = SearchShuffle::After;
            }
            ok
        });
    }
    // "Reveal those cards, put them into your hand, then shuffle.", "Put that card onto
    // the battlefield, then shuffle.", "Put one into your hand and the other into your
    // graveyard. Then shuffle.", "Shuffle and put that card on top.", "Then shuffle."
    let l = l.strip_prefix("then ").unwrap_or(l);
    let t = if let Some(r) = l.strip_prefix("shuffle") {
        format!(", then shuffle{r}")
    } else {
        format!(", {l}")
    };
    let ok = update_searches(prev, &mut |s| {
        if !matches!(s.who, PlayerRef::You) || s.shuffle != SearchShuffle::No {
            return false;
        }
        // A sentence that only shuffles may follow one that put the cards somewhere; the
        // others complete a search that hasn't.
        if !s.dests.is_empty() && !t.starts_with(", then shuffle") {
            return false;
        }
        if s.reveal && t.starts_with(", reveal") {
            return false;
        }
        let sr = Searcher {
            who: PlayerRef::You,
            their: PlayerRef::You,
            optional: false,
            third: false,
        };
        let mut new = s.clone();
        if !tail(&t, &mut new, &sr, b) || (new.shuffle != SearchShuffle::No && !own_library(&new)) {
            return false;
        }
        *s = new;
        true
    });
    if ok {
        b.it = Sel::Var(vars::IT);
    }
    ok
}

#[cfg(test)]
mod tests {
    use crate::card::CardDb;

    fn show(name: &str) -> String {
        let d = CardDb::global().get(name).expect("card");
        format!("{:?} {:?}", d.unsupported_text(), d.faces[0].chars.abilities)
    }

    #[test]
    fn named_cards_with_commas() {
        let s = show("Liliana's Influence");
        assert!(s.contains("Liliana, Death Wielder"), "{s}");
        assert!(s.starts_with("[]"), "{s}");
    }
}
