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
use crate::oracle::patterns::{ConditionPattern, EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;
use crate::types::*;

inventory::submit! {
    // After the simpler library-search patterns (`card_flow_search`, priority 90).
    EffectPattern { name: "search grammar: search zones for card specs", priority: 95, parse: search_clause }
}
inventory::submit! {
    FollowupPattern { name: "search grammar: complete the search", priority: 90, apply: complete_search }
}
inventory::submit! {
    EffectPattern { name: "search grammar: put the found cards somewhere [if condition]", priority: 95, parse: put_found }
}

inventory::submit! {
    ConditionPattern { name: "search grammar: an opponent controls more lands than you", priority: 100, parse: some_player_condition }
}
inventory::submit! {
    EffectPattern { name: "search grammar: that player shuffles / if you search your library this way, shuffle", priority: 95, parse: shuffle_after_search }
}

inventory::submit! {
    ConditionPattern { name: "search grammar: you control a land named Wastes", priority: 100, parse: control_named }
}

inventory::submit! {
    ConditionPattern { name: "search grammar: you've cast a spell named X and a spell named Y this turn", priority: 100, parse: cast_named_this_turn }
}

/// "you've cast a spell named Peer Through Depths and a spell named Reach Through Mists
/// this turn" (Sift Through Sands): each named spell was cast by you this turn (copies
/// weren't cast, CR 707.10).
fn cast_named_this_turn(c: &str) -> Option<Condition> {
    let r = c
        .strip_prefix("you've cast a spell named ")?
        .strip_suffix(" this turn")?;
    let names: Vec<String> = r
        .split(" and a spell named ")
        .map(printed_name)
        .collect::<Option<_>>()?;
    let conds: Vec<Condition> = names
        .into_iter()
        .map(|n| {
            Condition::Compare(
                Value::SpellsCastThisTurn(PlayerRef::You, Filter::Named(n.into())),
                Cmp::Ge,
                Value::c(1),
            )
        })
        .collect();
    Some(if conds.len() == 1 {
        conds.into_iter().next().expect("one")
    } else {
        Condition::And(conds)
    })
}

/// "you control a land named Wastes" (CR 201.2).
fn control_named(c: &str) -> Option<Condition> {
    let r = c
        .strip_prefix("you control a ")
        .or_else(|| c.strip_prefix("you control an "))?;
    let (kind, name) = r.split_once(" named ")?;
    let (f, _, tail) = parse_object_phrase(kind)?;
    let n = printed_name(name.trim())?;
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        f,
        Filter::Named(n.into()),
        Filter::ControlledBy(PlayerRel::You),
    ])))
}

/// "Then that player shuffles." (their library), and "If you search your library this way,
/// shuffle." after other instructions: only if the library was searched by this spell or
/// ability and hasn't been shuffled since (see `search_rules::YOU_SEARCHED_THIS_WAY`).
fn shuffle_after_search(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l == "that player shuffles" {
        if crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
            return None;
        }
        return Some(Effect::Shuffle {
            who: b.it_player.clone(),
        });
    }
    if l == "if you search your library this way, shuffle" {
        return Some(Effect::If {
            cond: Condition::Custom(crate::search_rules::YOU_SEARCHED_THIS_WAY.into()),
            then: Box::new(Effect::Shuffle {
                who: PlayerRef::You,
            }),
            otherwise: Box::new(Effect::Noop),
        });
    }
    None
}

/// "an opponent controls more lands than you", "a player controls more creatures than
/// you": some such player exists.
fn some_player_condition(c: &str) -> Option<Condition> {
    let (base, r) = if let Some(r) = c.strip_prefix("an opponent ") {
        (PlayerFilter::Opponent, r)
    } else {
        return None;
    };
    let pred = crate::oracle::patterns::value_grammar::player_clause(r)?;
    Some(Condition::Compare(
        Value::CountPlayers(PlayerFilter::And(vec![base, pred])),
        Cmp::Ge,
        Value::c(1),
    ))
}

/// A condition in a sentence about the search: "target opponent controls more lands than
/// you" (adding the target), or any other condition.
fn condition(c: &str, b: &mut Builder) -> Option<Condition> {
    for (p, pf, text) in [
        ("target opponent ", PlayerFilter::Opponent, "target opponent"),
        ("target player ", PlayerFilter::Any, "target player"),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            let pred = crate::oracle::patterns::value_grammar::player_clause(r)?;
            let slot = b.add_target(TargetSpec::player(pf, text), text);
            return Some(Condition::PlayerMatches(PlayerRef::Target(slot), pred));
        }
    }
    crate::oracle::statics::parse_condition(c, b.ctx)
}

/// "If [condition], you may search your library for an additional Plains card." (Tithe):
/// another part of the search, found only if the condition holds as it resolves.
/// "If [condition], instead search your library for ..." (Nissa's Triumph): a different
/// search. "If [condition], put those cards onto the battlefield instead of putting them
/// into your hand." (The Five Doctors).
fn search_conditional(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let saved = b.targets.len();
    let done = search_conditional_inner(c, x, prev, b).is_some();
    if !done {
        b.targets.truncate(saved);
    }
    done
}

fn search_conditional_inner(c: &str, x: &str, prev: &mut Effect, b: &mut Builder) -> Option<()> {
    let last = match prev {
        Effect::Seq(v) => v.last_mut()?,
        other => other,
    };
    let Effect::SearchCards(spec) = last else {
        return None;
    };
    if !matches!(spec.who, PlayerRef::You) {
        return None;
    }
    if let Some(desc) = x.strip_prefix("you may search your library for an additional ") {
        if !spec.dests.is_empty()
            || spec.shuffle != SearchShuffle::No
            || spec.zones != [ZoneKind::Library]
            || spec.optional
        {
            return None;
        }
        let cond = condition(c, b)?;
        let (parts, distinct, rest) = specs(&format!("a {desc}"), b)?;
        if !rest.is_empty() || distinct || parts.len() != 1 {
            return None;
        }
        let mut part = parts.into_iter().next()?;
        // "You may": a card with a stated quality needn't be found anyway (CR 701.23b).
        part.up_to = true;
        part.count = Value::If(Box::new(cond), Box::new(Value::c(1)), Box::new(Value::c(0)));
        spec.parts.push(part);
        return Some(());
    }
    if let Some(y) = x.strip_prefix("instead ") {
        if !spec.dests.is_empty() || spec.shuffle != SearchShuffle::No {
            return None;
        }
        let cond = condition(c, b)?;
        let alt = search_clause(y, b)?;
        let Effect::SearchCards(a) = &alt else {
            return None;
        };
        if !a.dests.is_empty() || a.shuffle != SearchShuffle::No || !matches!(a.who, PlayerRef::You) {
            return None;
        }
        let old = std::mem::take(last);
        *last = Effect::If {
            cond,
            then: Box::new(alt),
            otherwise: Box::new(old),
        };
        return Some(());
    }
    // "put those cards onto the battlefield instead of putting them into your hand".
    let y = x.strip_prefix("put ")?;
    let y = pronoun(y)?.trim_start();
    let you = Searcher {
        who: PlayerRef::You,
        their: PlayerRef::You,
        optional: false,
        third: false,
    };
    let (to, rest) = destination(y, &you, b)?;
    let rest = rest.trim();
    let instead_of = rest.strip_prefix("instead of putting ")?;
    let instead_of = pronoun(instead_of)?.trim_start();
    let (from, tail) = destination(instead_of, &you, b)?;
    if !tail.trim().is_empty()
        || spec.dests.len() != 1
        || spec.dests[0].to.zone != from.zone
        || to.zone == from.zone
    {
        return None;
    }
    let cond = condition(c, b)?;
    let mut alt = (**spec).clone();
    alt.dests[0].to = to;
    let old = std::mem::take(last);
    *last = Effect::If {
        cond,
        then: Box::new(Effect::SearchCards(Box::new(alt))),
        otherwise: Box::new(old),
    };
    Some(())
}

inventory::submit! {
    // Before the general "if [condition], [effect] instead" (priority 60).
    FollowupPattern { name: "search grammar: if [condition], search for an additional card / instead search", priority: 50, apply: search_conditional }
}

inventory::submit! {
    FollowupPattern { name: "search grammar: you may play/cast the exiled found cards", priority: 90, apply: may_play_found }
}

inventory::submit! {
    // Before the general "if [condition], [effect]" sentence.
    FollowupPattern { name: "search grammar: if you reveal a card named X this way, put it ...", priority: 50, apply: if_revealed_named }
}

/// "If you reveal a card named Hammer of Nazahn this way, put it onto the battlefield."
/// after "Search your library for an Equipment card and reveal it.": the found card goes
/// there if it has that name ("Otherwise, ..." completes it).
fn if_revealed_named(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you reveal a card named ") else {
        return false;
    };
    let Some((name, x)) = r.split_once(" this way, ") else {
        return false;
    };
    // The search before revealed what it found and left it where it was.
    let revealed = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::SearchCards(spec)) = revealed else {
        return false;
    };
    if !spec.reveal || !spec.dests.is_empty() || !matches!(spec.who, PlayerRef::You) {
        return false;
    }
    // "Hammer of ~": the card's own short name is part of the name.
    let short = b.ctx.card_name.split(", ").next().unwrap_or_default();
    let Some(name) = (if name.contains('~') && !short.is_empty() {
        printed_name(&name.replace('~', &short.to_lowercase()))
    } else {
        printed_name(name)
    }) else {
        return false;
    };
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return false;
    }
    let Some(mv) = put_found(x, b) else {
        return false;
    };
    if !matches!(mv, Effect::Move { .. }) {
        return false;
    }
    let cond = Condition::SelMatches(Sel::Var(vars::IT), Filter::Named(name.into()));
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(mv),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

/// "Until end of turn, you may play that card." / "You may cast them this turn." after a
/// search that exiled the found cards (Thada Adel, Chandra, Heart of Fire): a permission
/// for those objects (CR 400.7).
fn may_play_found(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let cast_only = match end(l) {
        "until end of turn, you may play that card"
        | "until end of turn, you may play those cards"
        | "you may play that card this turn"
        | "you may play them this turn" => false,
        "you may cast them this turn" | "you may cast that card this turn" => true,
        _ => return false,
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::SearchCards(spec)) = last else {
        return false;
    };
    let exiles = spec.dests.len() == 1 && spec.dests[0].to.zone == ZoneKind::Exile;
    // A permission to cast is one to play only for cards that can't be lands.
    if !exiles || (cast_only && !spec.parts.iter().all(|p| excludes_lands(&p.filter))) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            duration: Duration::EndOfTurn,
            free: false,
        },
    ]);
    true
}

/// Whether no land card matches `f` ("red instant and/or sorcery cards").
fn excludes_lands(f: &Filter) -> bool {
    match f {
        Filter::Type(t) => *t != CardType::Land,
        Filter::And(v) => v.iter().any(excludes_lands),
        Filter::Or(v) => !v.is_empty() && v.iter().all(excludes_lands),
        _ => false,
    }
}

/// "Put it onto the battlefield tapped if it's a land card", "put that card into your
/// hand", "put that card on top of your library" (the cards a search found, which "it"
/// names; a conditional destination is a separate instruction).
fn put_found(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let p = l.strip_prefix("put ")?;
    // After "Search ... and exile the rest.", only "the chosen cards" follow.
    let found = matches!(b.it, Sel::Var(crate::search_rules::FOUND));
    if !matches!(b.it, Sel::Var(vars::IT)) && !found {
        return None;
    }
    // "Put the chosen cards on top of your library in any order." (Doomsday: the cards
    // the search found; their owner orders them, CR 401.4). Only right after a search that
    // left the found cards where they were.
    if let Some(r) = p.strip_prefix("the chosen cards ") {
        if found && r == "on top of your library in any order" {
            return Some(Effect::Custom(crate::search_rules::FOUND_ON_TOP_ANY_ORDER.into()));
        }
        return None;
    }
    if found {
        return None;
    }
    let r = pronoun(p)?.trim_start();
    let you = Searcher {
        who: PlayerRef::You,
        their: PlayerRef::You,
        optional: false,
        third: false,
    };
    let (to, rest) = destination(r, &you, b)?;
    let mv = Effect::Move {
        what: Sel::Var(vars::IT),
        to,
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(mv);
    }
    let c = rest.strip_prefix("if ")?;
    // "if its mana value is 2 or less".
    if let Some(q) = c.strip_prefix("its mana value is ") {
        let (f, tail) = mana_value_qualifier(&format!("with mana value {q}"), b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some(Effect::If {
            cond: Condition::SelMatches(Sel::Var(vars::IT), f),
            then: Box::new(mv),
            otherwise: Box::new(Effect::Noop),
        });
    }
    let cond = match crate::oracle::patterns::statics_conditions::pronoun_state(c) {
        Some(f) if c.starts_with("it") => Condition::SelMatches(Sel::Var(vars::IT), f),
        _ => crate::oracle::statics::parse_condition(c, b.ctx)?,
    };
    Some(Effect::If {
        cond,
        then: Box::new(mv),
        otherwise: Box::new(Effect::Noop),
    })
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
        // "Any number of target players may each search their library": each of them
        // searches their own.
        if subject == "any number of target players" && verb == " may each search " {
            let mut spec = TargetSpec::player(PlayerFilter::Any, subject);
            spec.min = 0;
            spec.max = Value::c(99);
            let slot = b.add_target(spec, subject);
            let s = Searcher {
                who: PlayerRef::Target(slot),
                their: PlayerRef::Iterated,
                optional,
                third: false,
            };
            return Some((s, rest));
        }
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
        // "may each search" is for several players; "each opponent may search" is the
        // opponents' choice in APNAP order (`each_opponent_may`).
        if (verb == " may each search ") != (group && optional) {
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
    // The head ("basic land card", "Bird or basic land card") runs to the word "card".
    let head_end = s
        .match_indices("card")
        .map(|(i, _)| i)
        .find(|i| {
            let after = &s[i + 4..];
            let after = after.strip_prefix('s').unwrap_or(after);
            (*i == 0 || s[..*i].ends_with(' ')) && (after.is_empty() || after.starts_with([' ', ',']))
        })
        .map(|i| {
            let after = &s[i + 4..];
            i + 4 + usize::from(after.starts_with('s'))
        })?;
    let mut fs = Vec::new();
    let mut rest = s[head_end..].to_string();
    match head(&s[..head_end]) {
        Some(f) => fs.push(f),
        None => {
            let (f, _, r) = parse_object_phrase(s)?;
            let h = s[..s.len() - r.len()].trim_end_matches([' ', ',']);
            if h.len() < head_end {
                return None;
            }
            fs.push(f);
            rest = s[h.len()..].to_string();
        }
    }
    // Qualifiers this grammar reads itself ("with mana value equal to 1 plus the
    // sacrificed creature's mana value", "with deathtouch, hexproof, reach, or trample");
    // others the object phrase grammar reads ("not named ~", "with that name").
    let mut generic_done = fs.len() > 1 || rest.len() < s.len() - head_end;
    loop {
        if let Some((f2, r)) = rest.strip_prefix(' ').and_then(|r| qualifier(r, b)) {
            fs.push(f2);
            rest = r;
            continue;
        }
        let qualifies = [" with", " that", " not ", " without "]
            .iter()
            .any(|p| rest.starts_with(p));
        if !generic_done && qualifies {
            generic_done = true;
            if let Some((f, _, r)) = parse_object_phrase(s) {
                let h = s[..s.len() - r.len()].trim_end_matches([' ', ',']);
                if h.len() > head_end {
                    fs = vec![f];
                    rest = s[h.len()..].to_string();
                    continue;
                }
            }
        }
        break;
    }
    Some((Filter::and(fs), rest))
}

/// A card description's head: "basic land card", "Spider Hero card", "Bird or basic land
/// card" (a Bird card or a basic land card), "basic, Sphere, or Locus land card".
fn head(h: &str) -> Option<Filter> {
    if let Some((f, _, r)) = parse_object_phrase(h) {
        if r.trim().is_empty() {
            return Some(f);
        }
    }
    let (words, noun) = h.rsplit_once(' ')?;
    if !matches!(noun, "card" | "cards") {
        return None;
    }
    // Subtypes only: "Spider Hero card".
    let ws: Vec<&str> = words.split(' ').collect();
    if ws.len() > 1 && ws.iter().all(|w| subtype_word(w).is_some()) {
        let mut fs: Vec<Filter> = ws
            .iter()
            .filter_map(|w| subtype_word(w).map(Filter::Subtype))
            .collect();
        fs.push(Filter::Card);
        return Some(Filter::and(fs));
    }
    // Alternatives that share the noun: "Bird or basic land", "basic, Sphere, or Locus
    // land".
    let alts: Vec<&str> = words
        .split(", or ")
        .flat_map(|x| x.split(" or "))
        .flat_map(|x| x.split(", "))
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .collect();
    if alts.len() < 2 {
        return None;
    }
    let last = *alts.last()?;
    // "Locus land": the last alternative's final word ("land") is shared by alternatives
    // that only modify it (a supertype or a land type).
    let shared = last.rsplit_once(' ').map(|(_, w)| w);
    let mut out = Vec::new();
    for a in &alts {
        let modifies_shared = shared.is_some_and(|w| {
            w == "land" && !a.contains(' ') && (*a == "basic" || is_land_type(a) && *a != last)
        });
        let phrase = if *a == last || !modifies_shared {
            format!("{a} {noun}")
        } else {
            format!("{a} {} {noun}", shared.expect("checked"))
        };
        let (f, _, r) = parse_object_phrase(&phrase)?;
        if !r.trim().is_empty() {
            return None;
        }
        out.push(f);
    }
    Some(Filter::Or(out))
}

fn is_land_type(w: &str) -> bool {
    let mut c = w.chars();
    let cap: String = c
        .next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default();
    crate::types::is_land_type(&cap)
}

/// "with mana value ...", "with power or toughness 6 or greater", "with flashback or
/// disturb", "with a mana ability", "with enchant creature".
fn qualifier(s: &str, b: &mut Builder) -> Option<(Filter, String)> {
    if let Some(x) = mana_value_qualifier(s, b) {
        return Some(x);
    }
    if let Some(r) = s.strip_prefix("with power or toughness ") {
        let (n, r) = parse_number(r)?;
        let r = r.trim_start();
        let (c, r) = if let Some(r) = r.strip_prefix("or greater") {
            (Cmp::Ge, r)
        } else if let Some(r) = r.strip_prefix("or less") {
            (Cmp::Le, r)
        } else {
            return None;
        };
        let f = Filter::Or(vec![
            Filter::Power(c, Box::new(n.clone())),
            Filter::Toughness(c, Box::new(n)),
        ]);
        return Some((f, r.to_string()));
    }
    // "land cards that each have a basic land type" (Slimefoot's Survey): any card with
    // one of the five basic land types, basic or not.
    if let Some(r) = [
        "that each have a basic land type",
        "that have a basic land type",
        "that has a basic land type",
    ]
    .iter()
    .find_map(|p| s.strip_prefix(p))
    {
        let f = Filter::Or(
            ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .map(|n| Filter::Subtype((*n).into()))
                .collect(),
        );
        return Some((f, r.to_string()));
    }
    if let Some(r) = s.strip_prefix("with a mana ability") {
        return Some((Filter::Custom(crate::search_rules::HAS_MANA_ABILITY.into()), r.to_string()));
    }
    if let Some(r) = s.strip_prefix("with enchant creature") {
        return Some((Filter::Custom(crate::search_rules::ENCHANT_CREATURE.into()), r.to_string()));
    }
    // A list of keywords: "with flashback or disturb", "with deathtouch, hexproof, reach,
    // or trample".
    let r = s.strip_prefix("with ")?;
    let mut kws = Vec::new();
    let mut rest = r;
    loop {
        let (k, r2) = keyword(rest)?;
        kws.push(Filter::HasKeyword(k));
        if let Some(r3) = r2.strip_prefix(", or ").or_else(|| r2.strip_prefix(" or ")) {
            let (k, r4) = keyword(r3)?;
            kws.push(Filter::HasKeyword(k));
            rest = r4;
            break;
        }
        rest = r2.strip_prefix(", ")?;
    }
    (kws.len() > 1).then(|| (Filter::Or(kws), rest.to_string()))
}

/// A keyword's name at the start of `s` (two-word names first).
fn keyword(s: &str) -> Option<(crate::keywords::KeywordKind, &str)> {
    let ends: Vec<usize> = s
        .match_indices([' ', ','])
        .map(|(i, _)| i)
        .chain([s.len()])
        .collect();
    for e in ends.iter().take(2).rev() {
        if let Some(k) = crate::keywords::KeywordKind::from_name(&s[..*e]) {
            return Some((k, &s[*e..]));
        }
    }
    None
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
const PART_SEPS: [(&str, Sep); 7] = [
    (", and/or ", Sep::And),
    (", and ", Sep::And),
    (" and/or ", Sep::And),
    (" and ", Sep::And),
    (", or ", Sep::Or),
    (" or ", Sep::Or),
    (", ", Sep::Comma),
];

#[derive(Clone, Copy, PartialEq)]
enum Sep {
    And,
    Or,
    /// A comma in a list whose last separator says which kind it is.
    Comma,
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
        let (mut f, mut r) = description(r, b)?;
        // "basic land cards and/or Gate cards": alternatives without their own counts.
        loop {
            let alt = [" and/or ", ", or ", " or "].iter().find_map(|p| {
                let r2 = r.strip_prefix(p)?;
                if count(r2).is_some() {
                    return None;
                }
                let saved = b.targets.len();
                let d = description(r2, b);
                if d.is_none() {
                    b.targets.truncate(saved);
                }
                d
            });
            let Some((f2, r2)) = alt else {
                break;
            };
            f = Filter::Or(vec![f, f2]);
            r = r2;
        }
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
    // A comma list ends with "and" or "or" ("a white card, a blue card, and a green
    // card").
    if seps.last() == Some(&Sep::Comma) {
        return None;
    }
    let or = seps.contains(&Sep::Or);
    if or {
        // Alternatives for one card: every part is "a"/"an" and every separator "or"
        // (the last may follow a comma list).
        if seps.iter().any(|s| *s == Sep::And)
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
    d.controller = Some(match (&sr.who, &sr.their) {
        (PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer, _)
        | (_, PlayerRef::Iterated) => PlayerRef::Iterated,
        (w, _) => w.clone(),
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
            let x = x.trim_start();
            let x = x
                .strip_prefix("counters on it")
                .or_else(|| x.strip_prefix("counter on it"))?;
            d.with_counters.push((kind, n));
            r = x;
        } else {
            break;
        }
    }
    Some((d, r))
}

/// The instructions after the card specs: reveal, put/exile (possibly split), shuffle.
/// Fills them into `spec`; returns what's left of `t`.
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
    } else if let Some(x) = [
        ", exile ",
        ", and exile ",
        " and exile ",
        ", exiles ",
        " and exiles ",
    ]
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
        ", then shuffles",
        " then shuffles",
        ", then shuffle",
        " then shuffle",
        ", shuffle",
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
    let out = search_clause_inner(l, b).or_else(|| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
        subject_then_search(l, b)
    });
    if out.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    out
}

/// "That player loses 3 life, searches their library ...", "Target player gains 2 life,
/// then searches their library ...": an instruction for the same player first.
fn subject_then_search(l: &str, b: &mut Builder) -> Option<Effect> {
    let (at, len) = [", then searches ", ", searches "]
        .iter()
        .find_map(|p| l.find(p).map(|i| (i, p.len())))?;
    let head = &l[..at];
    let n0 = b.targets.len();
    let first = crate::oracle::effects::parse_clause(head, b)?;
    let who = if head.starts_with("target ") {
        // The target the first instruction's subject added.
        (b.targets.len() > n0).then_some(PlayerRef::Target(n0 as u8))?
    } else {
        let (w, _) = player_ref(head, b)?;
        if b.targets.len() != n0 {
            return None;
        }
        w
    };
    if matches!(
        who,
        PlayerRef::You | PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
    ) || !matches!(b.targets.get(n0).map(|t| &t.what), None | Some(TargetKind::Player(_)))
    {
        return None;
    }
    // The search's subject is that same player.
    let prev_player = std::mem::replace(&mut b.it_player, who);
    let search = search_clause_inner(&format!("that player searches {}", &l[at + len..]), b);
    if search.is_none() {
        b.it_player = prev_player;
    }
    Some(Effect::seq(vec![first, search?]))
}

fn search_clause_inner(l: &str, b: &mut Builder) -> Option<Effect> {
    let (sr, r) = searcher(l, b)?;
    let (whose, zones, zones_optional, r) = zones(r, &sr, b)?;
    // "Search its owner's graveyard, hand, and library ... That player shuffles."
    if !matches!(whose, PlayerRef::You | PlayerRef::Iterated) {
        b.it_player = whose.clone();
    }
    // The card specs run to the first action.
    let cut = ACTIONS
        .iter()
        .filter_map(|a| r.find(a))
        .min()
        .unwrap_or(r.len());
    let (desc, mut t) = r.split_at(cut);
    let (mut parts, distinct, leftover) = specs(desc, b)?;
    if !leftover.trim().is_empty() {
        return None;
    }
    // "Search its owner's graveyard, hand, and library for any number of cards with that
    // name": the name of the object "it" is (Deadly Cover-Up), not a chosen name.
    if let PlayerRef::OwnerOf(sel) = &whose {
        for p in &mut parts {
            if !replace_chosen_name(&mut p.filter, sel) {
                return None;
            }
        }
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
    // "..., then the player shuffles" / "..., then that player shuffles": the library of
    // the player whose library was searched.
    let mut that_player_shuffles = false;
    let owned2;
    for p in [", then the player shuffles", ", then that player shuffles"] {
        if let Some(x) = t.strip_suffix(p) {
            that_player_shuffles = true;
            owned2 = x.to_string();
            t = owned2.as_str();
            break;
        }
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
    // "Search your library and graveyard for five cards and exile the rest." (Doomsday):
    // the found cards stay where they are; every other card in the searched zones is
    // exiled.
    if let Some(x) = [" and exile the rest", ", then exile the rest", ", exile the rest"]
        .iter()
        .find_map(|p| t.strip_prefix(p))
    {
        if !x.trim().is_empty()
            || sr.third
            || sr.optional
            || spec.zones_optional
            || !matches!(spec.whose, PlayerRef::You)
            || x_value.is_some()
            || that_player_shuffles
        {
            return None;
        }
        let rest: Vec<Sel> = spec
            .zones
            .iter()
            .map(|z| {
                Sel::All(Filter::and(vec![
                    Filter::InZone(*z),
                    Filter::OwnedBy(PlayerRel::You),
                    Filter::Not(Box::new(Filter::In(Box::new(Sel::Var(
                        crate::search_rules::FOUND,
                    ))))),
                ]))
            })
            .collect();
        let exile = Effect::Exile {
            what: if rest.len() == 1 {
                rest.into_iter().next().expect("one zone")
            } else {
                Sel::Union(rest)
            },
            face_down: false,
            link: false,
        };
        // "The chosen cards" are the found ones, after the exile changed "it".
        b.it = Sel::Var(crate::search_rules::FOUND);
        return Some(Effect::seq(vec![
            Effect::SearchCards(Box::new(spec)),
            Effect::Store {
                var: crate::search_rules::FOUND,
                sel: Sel::Var(vars::IT),
            },
            exile,
        ]));
    }
    let leftover = tail_inner(t, &mut spec, &sr, b)?.to_string();
    // "..., put that card into your hand, discard a card at random, then shuffle": an
    // instruction between putting the cards somewhere and shuffling.
    let mut then = None;
    if !leftover.trim().is_empty() {
        if sr.third || spec.shuffle != SearchShuffle::No || spec.dests.is_empty() {
            return None;
        }
        let x = leftover.strip_prefix(", ")?;
        let x = x.strip_prefix("then ").unwrap_or(x);
        let (x, shuffle) = match x.strip_suffix(", then shuffle") {
            Some(x) => (x, true),
            None => (x, false),
        };
        let e = crate::oracle::effects::parse_clause(x, b)?;
        if !library_free(&e) {
            return None;
        }
        if shuffle {
            spec.shuffle = SearchShuffle::After;
        }
        then = Some(e);
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
    if that_player_shuffles {
        if own_library(&spec) || spec.shuffle != SearchShuffle::No {
            return None;
        }
        spec.shuffle = SearchShuffle::After;
    }
    let e = Effect::SearchCards(Box::new(spec));
    let e = match x_value {
        Some(x) => crate::oracle::patterns::r107_numbers::substitute_x(&e, &x)?,
        None => e,
    };
    b.it = Sel::Var(vars::IT);
    Some(match then {
        Some(t) => Effect::seq(vec![e, t]),
        None => e,
    })
}

/// Replaces "with that name" (`Filter::ChosenName`) with the name of `sel`; false if the
/// filter refers to a chosen name some other way.
fn replace_chosen_name(f: &mut Filter, sel: &Sel) -> bool {
    match f {
        Filter::ChosenName => {
            *f = Filter::SameNameAs(Box::new(sel.clone()));
            true
        }
        Filter::And(v) | Filter::Or(v) => v.iter_mut().all(|x| replace_chosen_name(x, sel)),
        other => !format!("{other:?}").contains("Chosen"),
    }
}

/// Whether two player references are the same reference.
fn same(a: &PlayerRef, c: &PlayerRef) -> bool {
    format!("{a:?}") == format!("{c:?}")
}

/// Whether the searcher searches their own zones.
fn own_library(spec: &SearchSpec) -> bool {
    // `Iterated`: each searcher's own.
    same(&spec.whose, &spec.who) || matches!(spec.whose, PlayerRef::Iterated)
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

/// Like [`update_searches`], also looking past later instructions to the most recent
/// search, if those instructions don't involve a library.
fn update_deep(prev: &mut Effect, f: &mut dyn FnMut(&mut SearchSpec) -> bool) -> bool {
    let mut new = prev.clone();
    match deep(&mut new, f) {
        Some(n) if n > 0 => {
            *prev = new;
            true
        }
        _ => false,
    }
}

fn deep(e: &mut Effect, f: &mut dyn FnMut(&mut SearchSpec) -> bool) -> Option<usize> {
    match e {
        Effect::SearchCards(s) => f(s).then_some(1),
        Effect::May { effect, .. } => deep(effect, f),
        Effect::If {
            then, otherwise, ..
        } => Some(deep(then, f)? + deep(otherwise, f)?),
        Effect::Seq(v) => {
            for i in (0..v.len()).rev() {
                let n = deep(&mut v[i], f)?;
                if n > 0 {
                    return v[i + 1..].iter().all(library_free).then_some(n);
                }
            }
            Some(0)
        }
        _ => Some(0),
    }
}

/// Whether an instruction doesn't involve a library (so shuffling one before or after it
/// makes no difference).
fn library_free(e: &Effect) -> bool {
    let j = serde_json::to_string(e).unwrap_or_default();
    !["\"Library\"", "Draw", "Mill", "Scry", "Surveil", "Dig", "Search", "Shuffle", "Explore"]
        .iter()
        .any(|w| j.contains(w))
}

/// "Then each player who searched their library this way shuffles." after "each opponent
/// may search their library ...": each opponent who accepted shuffles after searching.
fn opponents_who_searched_shuffle(prev: &mut Effect) -> bool {
    use crate::oracle::patterns::each_opponent_may::ACCEPTED;
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(Effect::ForEachPlayer { who: PlayerRef::Var(var), effect }) = v.last_mut() else {
        return false;
    };
    if *var != ACCEPTED {
        return false;
    }
    let Effect::AsPlayer { effect: inner, .. } = &mut **effect else {
        return false;
    };
    match &mut **inner {
        Effect::Search {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            shuffle,
            ..
        } if !*shuffle => {
            *shuffle = true;
            true
        }
        Effect::SearchCards(s)
            if matches!((&s.who, &s.whose), (PlayerRef::You, PlayerRef::You))
                && s.shuffle == SearchShuffle::No =>
        {
            s.shuffle = SearchShuffle::After;
            true
        }
        _ => false,
    }
}

/// Sentences that complete the search before them.
fn complete_search(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    // "If you search your library this way, shuffle." / "If they search their library
    // this way, they shuffle." / "Then shuffle." after instructions that use the found
    // cards: only a searched library is shuffled (the found cards are the same objects
    // whether it's shuffled before or after those instructions, which don't look at the
    // library).
    let you = matches!(
        l,
        "if you search your library this way, shuffle" | "then shuffle" | "shuffle"
    );
    if you || l == "if they search their library this way, they shuffle" {
        let done = update_deep(prev, &mut |s| {
            let ok = s.shuffle == SearchShuffle::No
                && s.zones.contains(&ZoneKind::Library)
                && own_library(s)
                && you == (matches!(s.who, PlayerRef::You));
            if ok {
                s.shuffle = SearchShuffle::After;
            }
            ok
        });
        if done || l.starts_with("if ") {
            return done;
        }
    }
    // "Then each player who searched their library this way shuffles." after "each
    // opponent may search their library ..." (`each_opponent_may`).
    if l == "then each player who searched their library this way shuffles" {
        return opponents_who_searched_shuffle(prev);
    }
    // "Then that player shuffles." after searching another player's library.
    let that = b.it_player.clone();
    let that_shuffles = |s: &mut SearchSpec| {
        let ok = s.shuffle == SearchShuffle::No
            && s.zones.contains(&ZoneKind::Library)
            && !same(&s.whose, &s.who)
            && same(&s.whose, &that);
        if ok {
            s.shuffle = SearchShuffle::After;
        }
        ok
    };
    if matches!(l, "then that player shuffles" | "that player shuffles") {
        return update_deep(prev, &mut { that_shuffles });
    }
    // "That player shuffles, then draws a card for each card exiled from their hand this
    // way." (Lost Legacy).
    if l == "that player shuffles, then draws a card for each card exiled from their hand this way" {
        let mut whose = None;
        let ok = update_searches(prev, &mut |s| {
            let exiles = s.dests.len() == 1 && s.dests[0].to.zone == ZoneKind::Exile;
            let ok = exiles && s.zones.contains(&ZoneKind::Hand) && that_shuffles(s);
            whose = Some(s.whose.clone());
            ok
        });
        if !ok {
            return false;
        }
        let draw = Effect::Draw {
            who: whose.expect("a search"),
            n: Value::CountSel(Box::new(Sel::Var(crate::search_rules::FROM_HAND))),
        };
        let old = std::mem::take(prev);
        *prev = Effect::seq(vec![old, draw]);
        return true;
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
    let mut then: Option<Effect> = None;
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
        let Some(left) = tail_inner(&t, &mut new, &sr, b) else {
            return false;
        };
        if new.shuffle != SearchShuffle::No && !own_library(&new) {
            return false;
        }
        // "Put it into your hand, then discard a card at random." (Wild Research).
        let left = left.trim();
        if !left.is_empty() {
            let Some(x) = left.strip_prefix(", then ") else {
                return false;
            };
            if new.dests.is_empty() || new.shuffle != SearchShuffle::No {
                return false;
            }
            match crate::oracle::effects::parse_clause(x, b) {
                Some(e) if library_free(&e) && then.is_none() => then = Some(e),
                _ => return false,
            }
        }
        *s = new;
        true
    });
    if let (true, Some(e)) = (ok, then.take()) {
        let old = std::mem::take(prev);
        *prev = Effect::seq(vec![old, e]);
    }
    if ok {
        b.it = Sel::Var(vars::IT);
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardDb;
    use crate::oracle::CompileContext;

    /// The effect a text compiles to on a sorcery, if every ability is understood.
    fn compiled(text: &str) -> Option<String> {
        let tl = TypeLine::parse("Sorcery");
        let ctx = CompileContext {
            card_name: "Testcard",
            full_name: "Testcard",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let c = crate::oracle::compile(text, &ctx);
        c.unsupported
            .is_empty()
            .then(|| format!("{:?}", c.abilities))
    }

    #[test]
    fn grammar_composes_parts_zones_and_searchers() {
        // Several parts with their own articles.
        let e = compiled(
            "Search your library for a red card and a green card, reveal them, put them into your hand, then shuffle.",
        )
        .expect("two parts");
        assert!(e.contains("Color(Red)") && e.contains("Color(Green)"), "{e}");
        assert!(e.matches("SearchPart").count() == 2, "{e}");
        // Another player searching, with third-person verbs.
        let e = compiled(
            "Target opponent searches their library for a land card, puts it onto the battlefield, then shuffles.",
        )
        .expect("third person");
        assert!(e.contains("who: Target(0), whose: Target(0)"), "{e}");
        // Zones in another order, a real card's name, and the follow-up shuffle.
        let e = compiled(
            "Search your graveyard and/or library for a card named Grizzly Bears and put it onto the battlefield. If you search your library this way, shuffle.",
        )
        .expect("zones");
        assert!(e.contains("zones: [Graveyard, Library], zones_optional: true"), "{e}");
        assert!(e.contains("Named(\"Grizzly Bears\")") && e.contains("shuffle: After"), "{e}");
        // A split with "two of them" and "the rest".
        let e = compiled(
            "Search your library for up to four basic land cards, reveal them, put two of them onto the battlefield tapped and the rest into your hand, then shuffle.",
        )
        .expect("split");
        assert!(e.contains("count: Some(Const(2))"), "{e}");
    }

    #[test]
    fn grammar_rejects_what_it_cannot_do_exactly() {
        // A name that isn't a real card's.
        assert!(compiled(
            "Search your library and/or graveyard for a card named Notacard Atall, reveal it, and put it into your hand."
        )
        .is_none());
        // Third-person verbs need a third-person subject.
        assert!(compiled(
            "Search your library for a land card, puts it onto the battlefield, then shuffles."
        )
        .is_none());
    }

    #[test]
    fn cards_that_each_have_a_basic_land_type() {
        // Slimefoot's Survey: nonbasic lands with a basic land type are found too.
        let e = compiled(
            "Search your library for up to two land cards that each have a basic land type, put them onto the battlefield tapped, then shuffle.",
        )
        .expect("qualifier");
        assert!(
            e.contains("Or([Subtype(\"Plains\"), Subtype(\"Island\"), Subtype(\"Swamp\"), Subtype(\"Mountain\"), Subtype(\"Forest\")])"),
            "{e}"
        );
        assert!(!e.contains("Basic"), "{e}");
        let e = compiled(
            "Search your library for a land card that has a basic land type, reveal it, put it into your hand, then shuffle.",
        )
        .expect("singular");
        assert!(e.contains("Subtype(\"Forest\")"), "{e}");
    }

    #[test]
    fn exile_the_rest_and_put_the_chosen_cards_on_top() {
        let e = compiled(
            "Search your library and graveyard for five cards and exile the rest. Put the chosen cards on top of your library in any order.",
        )
        .expect("doomsday");
        assert!(e.contains("dests: [], shuffle: No"), "{e}");
        assert!(e.contains("Store"), "{e}");
        assert!(e.contains(crate::search_rules::FOUND_ON_TOP_ANY_ORDER), "{e}");
        // Not after a search that chooses its zones, or another player's.
        assert!(compiled(
            "Search your library and/or graveyard for five cards and exile the rest."
        )
        .is_none());
        assert!(compiled(
            "Search target player's library and graveyard for five cards and exile the rest."
        )
        .is_none());
    }

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
