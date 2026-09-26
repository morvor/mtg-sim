//! Parsing of noun phrases: numbers, object descriptions ("nontoken creature you
//! control"), target phrases ("up to two target creatures"), and player phrases.
//!
//! Functions take a lowercase phrase and return the parsed structure plus the unparsed
//! remainder, so callers can chain them.

use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::types::*;
use smol_str::SmolStr;

/// Parses a number word or digits at the start of `s`. Returns (value, rest).
pub fn parse_number(s: &str) -> Option<(Value, &str)> {
    let s = s.trim_start();
    let (w, rest) = split_word(s);
    let n = match w {
        "a" | "an" | "one" | "1" => 1,
        "two" | "2" => 2,
        "three" | "3" => 3,
        "four" | "4" => 4,
        "five" | "5" => 5,
        "six" | "6" => 6,
        "seven" | "7" => 7,
        "eight" | "8" => 8,
        "nine" | "9" => 9,
        "ten" | "10" => 10,
        "eleven" | "11" => 11,
        "twelve" | "12" => 12,
        "thirteen" | "13" => 13,
        "fourteen" | "14" => 14,
        "fifteen" | "15" => 15,
        "twenty" | "20" => 20,
        "x" => return Some((Value::X, rest)),
        other => {
            if let Ok(n) = other.parse::<i32>() {
                n
            } else {
                return None;
            }
        }
    };
    Some((Value::Const(n), rest))
}

/// Splits off the first word.
pub fn split_word(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    match s.find(' ') {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

/// Strips a prefix (case-sensitive; callers lowercase first). Returns the rest.
pub fn strip<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    s.trim_start().strip_prefix(p).map(|r| r.trim_start())
}

fn singular(w: &str) -> String {
    let w = w.trim_end_matches(',');
    if let Some(t) = CardType::from_word(w) {
        return t.word().to_string();
    }
    for (pl, sg) in [
        ("sorceries", "sorcery"),
        ("elves", "elf"),
        ("dwarves", "dwarf"),
        ("wolves", "wolf"),
        ("werewolves", "werewolf"),
        ("fungi", "fungus"),
        ("mercenaries", "mercenary"),
        ("allies", "ally"),
        ("faeries", "faerie"),
        ("zombies", "zombie"),
        ("goblins", "goblin"),
        ("equipment", "equipment"),
        ("auras", "aura"),
        ("permanents", "permanent"),
        ("spells", "spell"),
        ("cards", "card"),
        ("tokens", "token"),
    ] {
        if w == pl {
            return sg.to_string();
        }
    }
    if let Some(stem) = w.strip_suffix("ies") {
        return format!("{stem}y");
    }
    if w.ends_with("ses") || w.ends_with("xes") || w.ends_with("ches") || w.ends_with("shes") {
        return w[..w.len() - 2].to_string();
    }
    if let Some(stem) = w.strip_suffix('s') {
        if !stem.ends_with('s') {
            return stem.to_string();
        }
    }
    w.to_string()
}

fn capitalize(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Recognizes a subtype word (any case/plural) and returns its canonical form.
pub fn subtype_word(w: &str) -> Option<Subtype> {
    let lower = w.to_lowercase();
    let sg = singular(&lower);
    let cap = capitalize(&sg);
    if subtype_kind(&cap).is_some() {
        return Some(SmolStr::new(cap));
    }
    // Plurals the general rule gets wrong: "Horses", "Heroes", "Mice", "Pegasi".
    let irregular = match lower.as_str() {
        "mice" => Some("mouse"),
        "oxen" => Some("ox"),
        "pegasi" => Some("pegasus"),
        "cyclopes" => Some("cyclops"),
        _ => None,
    };
    for cand in [lower.strip_suffix('s'), lower.strip_suffix("es"), irregular]
        .into_iter()
        .flatten()
    {
        let cap = capitalize(cand);
        if subtype_kind(&cap).is_some() {
            return Some(SmolStr::new(cap));
        }
    }
    // Possessive land types like "Urza's"
    let cap_raw = capitalize(w);
    if subtype_kind(&cap_raw).is_some() {
        return Some(SmolStr::new(cap_raw));
    }
    None
}

/// A single "head" noun: card type, subtype, "permanent", "spell", "card".
pub fn head_noun(w: &str) -> Option<Filter> {
    let sg = singular(w);
    match sg.as_str() {
        "permanent" => return Some(Filter::Permanent),
        "spell" => return Some(Filter::Spell),
        // Tokens and copies aren't cards (CR 108.2, 108.2b).
        "card" => return Some(Filter::Card),
        "token" => return Some(Filter::Token),
        // CR 700.12.
        "outlaw" => return Some(crate::game_terms::outlaw_filter()),
        _ => {}
    }
    if let Some(t) = CardType::from_word(&sg) {
        return Some(Filter::Type(t));
    }
    subtype_word(w).map(Filter::Subtype)
}

/// Adjectives preceding the head noun.
pub fn adjective(w: &str) -> Option<Filter> {
    if let Some(c) = Color::from_word(w) {
        return Some(Filter::Color(c));
    }
    if let Some(rest) = w.strip_prefix("non-").or_else(|| w.strip_prefix("non")) {
        if let Some(c) = Color::from_word(rest) {
            return Some(Filter::not(Filter::Color(c)));
        }
        if let Some(f) = head_noun(rest) {
            return Some(Filter::not(f));
        }
        if let Some(s) = Supertype::from_word(rest) {
            return Some(Filter::not(Filter::Supertype(s)));
        }
        if rest == "token" {
            return Some(Filter::not(Filter::Token));
        }
    }
    if let Some(s) = Supertype::from_word(w) {
        return Some(Filter::Supertype(s));
    }
    Some(match w {
        "colorless" => Filter::Colorless,
        "multicolored" => Filter::Multicolored,
        "monocolored" => Filter::Monocolored,
        "tapped" => Filter::Tapped,
        "untapped" => Filter::Untapped,
        "attacking" => Filter::Attacking,
        "blocking" => Filter::Blocking,
        "blocked" => Filter::Blocked,
        "unblocked" => Filter::Unblocked,
        "token" => Filter::Token,
        "historic" => Filter::Historic,
        "face-down" => Filter::FaceDown,
        "enchanted" => Filter::Enchanted,
        "equipped" => Filter::Equipped,
        "modified" => Filter::Modified,
        // CR 700.16.
        "worthy" => crate::game_terms::worthy_filter(),
        // CR 701.27g.
        "transformed" => Filter::Custom(crate::transform_rules::TRANSFORMED.into()),
        _ => return None,
    })
}

/// Parses an object description like "nontoken creature you control with flying".
/// Returns (filter, plural?, rest).
pub fn parse_object_phrase(s: &str) -> Option<(Filter, bool, &str)> {
    let mut s = s.trim_start();
    let mut parts: Vec<Filter> = Vec::new();
    // "another"/"other"
    if let Some(r) = strip(s, "another ") {
        parts.push(Filter::Other);
        s = r;
    } else if let Some(r) = strip(s, "other ") {
        parts.push(Filter::Other);
        s = r;
    }
    // Adjectives. Space- and comma-separated adjectives are conjunctive ("nonartifact,
    // nonblack creature"); a list joined by "or" is disjunctive ("attacking or blocking
    // creature", "red, white, or black creature").
    let is_adj = |w: &str| {
        let w2 = w.trim_end_matches(',');
        !w2.is_empty()
            && (head_noun(w2).is_none() || matches!(w2, "token" | "tokens"))
            && adjective(w2).is_some()
    };
    let mut items: Vec<Vec<Filter>> = vec![vec![]];
    let mut disjunctive = false;
    let mut last_adjective = "";
    loop {
        let (w, rest) = split_word(s);
        let w2 = w.trim_end_matches(',');
        if w2.is_empty() {
            break;
        }
        if matches!(w2, "or" | "and/or") && items.iter().any(|v| !v.is_empty()) {
            // Only an adjective list continues after "or".
            if !is_adj(split_word(rest).0) {
                break;
            }
            disjunctive = true;
            if !items.last().unwrap().is_empty() {
                items.push(vec![]);
            }
            s = rest;
            continue;
        }
        // Stop if this word is a head noun (but "token" can be either).
        if !is_adj(w) {
            break;
        }
        items.last_mut().unwrap().push(adjective(w2).unwrap());
        last_adjective = w2;
        s = rest;
        if w.ends_with(',') {
            let next = split_word(rest).0;
            if is_adj(next) || matches!(next, "or" | "and/or") {
                items.push(vec![]);
            }
        }
    }
    items.retain(|v| !v.is_empty());
    if disjunctive && items.len() > 1 {
        parts.push(Filter::Or(
            items.into_iter().map(Filter::and).collect::<Vec<_>>(),
        ));
    } else {
        parts.extend(items.into_iter().flatten());
    }
    // Head nouns joined by "or", "and/or", commas.
    let mut heads: Vec<Filter> = Vec::new();
    // Start of the heads joined since the last narrowing noun ("instant or sorcery"
    // before "spell").
    let mut group_start = 0;
    let mut plural = false;
    let mut head_subtypes_only = true;
    loop {
        let (w, rest) = split_word(s);
        let w2 = w.trim_end_matches(',');
        let Some(f) = head_noun(w2) else { break };
        if !matches!(f, Filter::Subtype(_)) {
            head_subtypes_only = false;
        }
        if (w2.ends_with('s') && singular(w2) != w2) || matches!(w2, "mice" | "pegasi" | "oxen") {
            plural = true;
        }
        heads.push(f);
        s = rest;
        // Continue after "or", "and/or", "and", or comma lists.
        let t = s.trim_start();
        if let Some(r) = t.strip_prefix("or ").or_else(|| t.strip_prefix("and/or ")) {
            s = r;
            continue;
        }
        if w.ends_with(',') {
            if let Some(r) = t.strip_prefix("or ").or_else(|| t.strip_prefix("and/or ")) {
                s = r;
            } else if let Some(r) = t.strip_prefix("and ") {
                // "artifacts, creatures, and lands" (a plural list names a union), "each
                // artifact, creature, and enchantment" (card types).
                let next = head_noun(split_word(r).0.trim_end_matches(','));
                let types = matches!(heads.last(), Some(Filter::Type(_)))
                    && matches!(next, Some(Filter::Type(_)));
                if (plural && next.is_some()) || types {
                    s = r;
                }
            }
            continue;
        }
        // "artifacts and enchantments": plural nouns joined by "and" name a union.
        if plural {
            if let Some(r) = t.strip_prefix("and ") {
                let nw = split_word(r).0.trim_end_matches(',');
                if nw.ends_with('s') && head_noun(nw).is_some() {
                    s = r;
                    continue;
                }
            }
        }
        // "each creature and planeswalker", "all artifact and creature cards": card types
        // joined by "and" name objects with either type.
        if let Some(r) = t.strip_prefix("and ") {
            let nw = split_word(r).0.trim_end_matches(',');
            if matches!(heads.last(), Some(Filter::Type(_)))
                && matches!(head_noun(nw), Some(Filter::Type(_)))
            {
                s = r;
                continue;
            }
        }
        // "creature card", "artifact spell", "Elf creature": a following head noun narrows.
        let (nw, nrest) = split_word(s);
        let nw2 = nw.trim_end_matches(',');
        // "Mercenary permanent card": "permanent card" narrows as one noun (CR 110.4a-b).
        let (nw2, nrest, perm_card) = match (nw2, split_word(nrest)) {
            ("permanent", (w, r)) if matches!(w.trim_end_matches(','), "card" | "cards") => {
                (w.trim_end_matches(','), r, true)
            }
            _ => (nw2, nrest, false),
        };
        if let Some(nf) = head_noun(nw2) {
            let nf = if perm_card {
                Filter::and(vec![Filter::PermanentCard, nf])
            } else {
                nf
            };
            // The noun narrows every head joined since the last narrowing: "instant or
            // sorcery spell", "artifact and enchantment cards".
            let group: Vec<Filter> = heads
                .drain(group_start..)
                .map(|last| match (last, &nf) {
                    // "permanent card" / "permanent spell" (CR 110.4a-b): not on the
                    // battlefield, but with a permanent card type.
                    (Filter::Permanent, Filter::Card | Filter::Spell | Filter::Any) => {
                        Filter::PermanentCard
                    }
                    (last, _) => last,
                })
                .collect();
            if nw2.ends_with('s') && singular(nw2) != nw2 {
                plural = true;
            }
            let joined = if group.len() == 1 {
                group.into_iter().next().unwrap()
            } else {
                Filter::Or(group)
            };
            heads.push(Filter::and(vec![joined, nf]));
            group_start = heads.len();
            s = nrest;
            // "Dragon creature cards": a subtype narrowed by a card type, then by "card".
            let mut ends_in_card = matches!(nw2, "card" | "cards");
            let (cw, crest) = split_word(s);
            let cw2 = cw.trim_end_matches(',');
            if matches!(cw2, "card" | "cards")
                && matches!(heads.last(), Some(Filter::And(v)) if v.len() == 2
                    && matches!(v[0], Filter::Subtype(_))
                    && matches!(v[1], Filter::Type(_)))
            {
                if cw2 == "cards" {
                    plural = true;
                }
                let last = heads.pop().unwrap();
                heads.push(Filter::and(vec![last, Filter::Card]));
                group_start = heads.len();
                s = crest;
                ends_in_card = true;
            }
            // allow "creature card or artifact card"
            let t = s.trim_start();
            if let Some(r) = t.strip_prefix("or ") {
                // "basic land card or Gate card": adjectives before a complete "... card"
                // phrase describe only that phrase, not the alternative after "or" (but
                // "nontoken artifact creature or Vehicle" is one description).
                if ends_in_card && parts.iter().any(|p| !matches!(p, Filter::Other)) {
                    let adjs: Vec<Filter> = parts
                        .iter()
                        .filter(|p| !matches!(p, Filter::Other))
                        .cloned()
                        .collect();
                    parts.retain(|p| matches!(p, Filter::Other));
                    for h in heads.iter_mut() {
                        let mut v = adjs.clone();
                        v.push(h.clone());
                        *h = Filter::and(v);
                    }
                }
                s = r;
                continue;
            }
        }
        break;
    }
    // "a token", "tokens you control": "token" was the head noun after all.
    if heads.is_empty() && !disjunctive && matches!(last_adjective, "token" | "tokens") {
        parts.pop();
        heads.push(Filter::Token);
        plural = last_adjective == "tokens";
    }
    if heads.is_empty() {
        return None;
    }
    let _ = head_subtypes_only;
    let head = if heads.len() == 1 {
        heads.pop().unwrap()
    } else {
        Filter::Or(heads)
    };
    parts.push(head);
    // Suffixes.
    loop {
        let t = s.trim_start();
        let (f, rest) = if let Some(r) = t.strip_prefix("you control") {
            (Filter::ControlledBy(PlayerRel::You), r)
        } else if let Some(r) = t.strip_prefix("your team controls") {
            // CR 102.4: "your team" means "you and/or your teammates".
            (
                Filter::Or(vec![
                    Filter::ControlledBy(PlayerRel::You),
                    Filter::ControlledBy(PlayerRel::Teammate),
                ]),
                r,
            )
        } else if let Some(r) = t.strip_prefix("you don't control") {
            (Filter::ControlledBy(PlayerRel::NotYou), r)
        } else if let Some(r) = t
            .strip_prefix("an opponent controls")
            .or_else(|| t.strip_prefix("your opponents control"))
        {
            (Filter::ControlledBy(PlayerRel::Opponent), r)
        } else if let Some(r) = t.strip_prefix("the triggering player controls") {
            // Internal form of "that player controls" inside a trigger whose player is
            // the triggering player (see `triggers_effects::that_player_controls`).
            (Filter::ControlledBy(PlayerRel::TriggerPlayer), r)
        } else if let Some(r) = t.strip_prefix("you own") {
            (Filter::OwnedBy(PlayerRel::You), r)
        } else if let Some(r) = t
            .strip_prefix("in your graveyard")
            .or_else(|| t.strip_prefix("from your graveyard"))
        {
            (
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                r,
            )
        } else if let Some(r) = t
            .strip_prefix("in a graveyard")
            .or_else(|| t.strip_prefix("from a graveyard"))
        {
            (Filter::InZone(ZoneKind::Graveyard), r)
        } else if let Some(r) = t
            .strip_prefix("in an opponent's graveyard")
            .or_else(|| t.strip_prefix("from an opponent's graveyard"))
        {
            (
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::OwnedBy(PlayerRel::Opponent),
                ]),
                r,
            )
        } else if let Some(r) = t
            .strip_prefix("in your hand")
            .or_else(|| t.strip_prefix("from your hand"))
        {
            (
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Hand),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                r,
            )
        } else if let Some(r) = t.strip_prefix("in exile") {
            (Filter::InZone(ZoneKind::Exile), r)
        } else if let Some(r) = t.strip_prefix("on the battlefield") {
            (Filter::InZone(ZoneKind::Battlefield), r)
        } else if let Some(r) = t.strip_prefix("with flying") {
            (Filter::HasKeyword(KeywordKind::Flying), r)
        } else if let Some(r) = t.strip_prefix("without flying") {
            (Filter::not(Filter::HasKeyword(KeywordKind::Flying)), r)
        } else if let Some(r) = t.strip_prefix("with defender") {
            (Filter::HasKeyword(KeywordKind::Defender), r)
        } else if let Some(r) = t.strip_prefix("with a +1/+1 counter on it") {
            (
                Filter::HasCounter(Some(crate::types::counters::PLUS1.into())),
                r,
            )
        } else if let Some(r) = t.strip_prefix("with a counter on it") {
            (Filter::HasCounter(None), r)
        } else if let Some((f, r)) = parse_stat_suffix(t) {
            (f, r)
        } else if let Some((f, r)) = parse_with_suffix(t) {
            (f, r)
        } else if let Some(r) = t
            .strip_prefix("that's attacking")
            .or_else(|| t.strip_prefix("that is attacking"))
        {
            (Filter::Attacking, r)
        } else if let Some(r) = t.strip_prefix("that's blocking") {
            (Filter::Blocking, r)
        } else if let Some(r) = t
            .strip_prefix("that was dealt damage this turn")
            .or_else(|| t.strip_prefix("that were dealt damage this turn"))
        {
            (Filter::DealtDamageThisTurn, r)
        } else if let Some(r) = t
            .strip_prefix("that was activated this turn")
            .or_else(|| t.strip_prefix("that were activated this turn"))
        {
            // CR 700.10.
            (
                Filter::Custom(crate::game_terms::ACTIVATED_THIS_TURN.into()),
                r,
            )
        } else if let Some(r) = t.strip_prefix("defending player controls") {
            (Filter::ControlledBy(PlayerRel::Defending), r)
        } else if let Some(r) = t.strip_prefix("blocking or blocked by ~") {
            (
                Filter::Or(vec![Filter::BlockingSource, Filter::BlockedBySource]),
                r,
            )
        } else if let Some(r) = t.strip_prefix("blocking ~") {
            (Filter::BlockingSource, r)
        } else if let Some(r) = t.strip_prefix("blocked by ~") {
            (Filter::BlockedBySource, r)
        } else if let Some((f, r)) = parse_chosen_suffix(t) {
            (f, r)
        } else if let Some((f, r)) = parse_originally_printed_suffix(t) {
            (f, r)
        } else {
            break;
        };
        parts.push(f);
        s = rest;
    }
    Some((Filter::and(parts), plural, s))
}

/// "target player controls" / "target opponent controls" after an object phrase. The
/// phrase parser leaves this suffix unparsed because it introduces a target of its own;
/// callers that can add targets bind it (see `effects::bind_target_player`).
pub fn target_player_controls(s: &str) -> Option<(PlayerFilter, &'static str, &str)> {
    let t = s.trim_start();
    if let Some(r) = t.strip_prefix("target player controls") {
        return Some((PlayerFilter::Any, "target player", r));
    }
    if let Some(r) = t.strip_prefix("target opponent controls") {
        return Some((PlayerFilter::Opponent, "target opponent", r));
    }
    None
}

/// "with a name originally printed in the Arabian Nights expansion" (CR 206.3).
fn parse_originally_printed_suffix(t: &str) -> Option<(Filter, &str)> {
    let r = t.strip_prefix("with a name originally printed in the ")?;
    let (set, rest) = r.split_once(" expansion")?;
    // Only the expansions whose names the Comprehensive Rules list are known (CR 206.3a-c).
    if !crate::names::has_listed_names(set) {
        return None;
    }
    Some((Filter::NameOriginallyPrintedIn(set.trim().into()), rest))
}

/// References to a choice made for the source (CR 607.2d): "of the chosen type",
/// "of the chosen color", "with the chosen name", "of the chosen card type".
fn parse_chosen_suffix(t: &str) -> Option<(Filter, &str)> {
    for (p, f) in [
        ("of the chosen creature type", Filter::ChosenType),
        ("of the chosen card type", Filter::ChosenCardType),
        ("of the chosen type", Filter::ChosenType),
        ("of the chosen color", Filter::ChosenColor),
        ("that's the chosen color", Filter::ChosenColor),
        ("that are the chosen color", Filter::ChosenColor),
        ("with the chosen name", Filter::ChosenName),
        // Double agenda's names (CR 702.106f).
        (
            "with one of the chosen names",
            Filter::Custom(crate::kw::hidden_agenda::ONE_OF_CHOSEN_NAMES.into()),
        ),
        (
            "with the other chosen name",
            Filter::Custom(crate::kw::hidden_agenda::OTHER_CHOSEN_NAME.into()),
        ),
        // "Choose a creature type. ... creatures of that type": the choice just made.
        ("of that type", Filter::ChosenType),
        ("of that color", Filter::ChosenColor),
        ("with that name", Filter::ChosenName),
        (
            "that aren't of the chosen type",
            Filter::not(Filter::ChosenType),
        ),
        (
            "that isn't of the chosen type",
            Filter::not(Filter::ChosenType),
        ),
        (
            "that aren't the chosen color",
            Filter::not(Filter::ChosenColor),
        ),
        (
            "that isn't the chosen color",
            Filter::not(Filter::ChosenColor),
        ),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((f, r));
            }
        }
    }
    None
}

/// "with deathtouch", "without first strike", "with a -1/-1 counter on it": a keyword
/// without parameters, or a counter of a kind.
fn parse_with_suffix(t: &str) -> Option<(Filter, &str)> {
    let (negate, rest) = if let Some(r) = t.strip_prefix("without ") {
        (true, r)
    } else {
        (false, t.strip_prefix("with ")?)
    };
    // "with no abilities" (CR 113.12: granted abilities count, characteristics and
    // qualities don't).
    if let Some(tail) = rest.strip_prefix("no abilities") {
        return Some((Filter::not(Filter::HasAbilities), tail));
    }
    // "with a cycling ability" (typecycling abilities are cycling abilities, CR 702.29f),
    // "with a morph ability" (megamorph is morph, CR 702.37b), "with a kicker ability"
    // (multikicker is kicker, CR 702.33c).
    for (word, kind) in [
        ("cycling", KeywordKind::Cycling),
        ("morph", KeywordKind::Morph),
        ("kicker", KeywordKind::Kicker),
    ] {
        let one = format!("a {word} ability");
        let many = format!("{word} abilities");
        if let Some(tail) = rest
            .strip_prefix(one.as_str())
            .or_else(|| rest.strip_prefix(many.as_str()))
        {
            let f = Filter::HasKeyword(kind);
            return Some((if negate { Filter::not(f) } else { f }, tail));
        }
    }
    // "with an activated ability that isn't a mana ability" (an ability such as cycling
    // exists in every zone, CR 702.29b).
    if let Some(tail) = rest
        .strip_prefix("an activated ability that isn't a mana ability")
        .or_else(|| rest.strip_prefix("activated abilities that aren't mana abilities"))
    {
        let f = Filter::Custom(crate::custom::HAS_NONMANA_ACTIVATED_ABILITY.into());
        return Some((if negate { Filter::not(f) } else { f }, tail));
    }
    // "with a +1/+1 counter on it", "without a +1/+1 counter on it" (Arcus Acolyte).
    if let Some(r) = rest.strip_prefix("a ").or_else(|| {
        (!negate)
            .then(|| rest.strip_prefix("one or more "))
            .flatten()
    }) {
        let (kind, r2) = split_word(r);
        if let Some(tail) = r2
            .strip_prefix("counter on it")
            .or_else(|| r2.strip_prefix("counters on it"))
        {
            if kind.starts_with('+')
                || kind.starts_with('-')
                || kind.chars().all(|c| c.is_alphabetic())
            {
                let f = Filter::HasCounter(Some(kind.into()));
                return Some((if negate { Filter::not(f) } else { f }, tail));
            }
        }
    }
    // Two-word keywords first ("first strike", "double strike").
    let words: Vec<&str> = rest.splitn(3, ' ').collect();
    for n in [2usize, 1] {
        if words.len() < n {
            continue;
        }
        let name = words[..n].join(" ");
        let name = name.trim_end_matches(',');
        let Some(k) = KeywordKind::from_name(name) else {
            continue;
        };
        let consumed: usize = words[..n].iter().map(|w| w.len()).sum::<usize>() + (n - 1);
        let tail = &rest[consumed.min(rest.len())..];
        let f = Filter::HasKeyword(k);
        return Some((if negate { Filter::not(f) } else { f }, tail));
    }
    None
}

/// "with power 2 or less", "with mana value 3 or greater", "with toughness 4 or greater".
fn parse_stat_suffix(t: &str) -> Option<(Filter, &str)> {
    // "with power greater than its base power" (CR 208.4b).
    for (p, cmp) in [
        ("with power greater than its base power", Cmp::Gt),
        ("each with power greater than its base power", Cmp::Gt),
        ("with power different from its base power", Cmp::Ne),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::PowerVsBase(cmp), r));
        }
    }
    let (stat, rest) = if let Some(r) = t.strip_prefix("with power ") {
        ("power", r)
    } else if let Some(r) = t.strip_prefix("with toughness ") {
        ("toughness", r)
    } else if let Some(r) = t.strip_prefix("with mana value ") {
        ("mv", r)
    } else {
        return None;
    };
    // "with mana value equal to the chosen number" (CR 607.2d)
    for (p, cmp) in [
        ("equal to the chosen number", Cmp::Eq),
        ("greater than or equal to the chosen number", Cmp::Ge),
        ("less than or equal to the chosen number", Cmp::Le),
    ] {
        if let Some(r) = rest.strip_prefix(p) {
            let v = Box::new(Value::Chosen);
            let f = match stat {
                "power" => Filter::Power(cmp, v),
                "toughness" => Filter::Toughness(cmp, v),
                _ => Filter::ManaValue(cmp, v),
            };
            return Some((f, r));
        }
    }
    // "with mana value equal to the number of charge counters on ~" (read as the effect
    // checks each object; the source's last known information if it's gone).
    for (p, cmp) in [
        ("equal to the number of ", Cmp::Eq),
        ("less than or equal to the number of ", Cmp::Le),
    ] {
        if let Some(r) = rest.strip_prefix(p) {
            let (kind, r) = split_word(r);
            let r = r
                .strip_prefix("counters on ~")
                .or_else(|| r.strip_prefix("counter on ~"))?;
            if !kind
                .chars()
                .all(|c| c.is_alphabetic() || c == '+' || c == '-' || c == '/')
            {
                return None;
            }
            let v = Box::new(Value::CountersOn(Box::new(Sel::This), Some(kind.into())));
            let f = match stat {
                "power" => Filter::Power(cmp, v),
                "toughness" => Filter::Toughness(cmp, v),
                _ => Filter::ManaValue(cmp, v),
            };
            return Some((f, r));
        }
    }
    let (n, rest) = parse_number(rest)?;
    let rest = rest.trim_start();
    let (cmp, rest) = if let Some(r) = rest.strip_prefix("or less") {
        (Cmp::Le, r)
    } else if let Some(r) = rest
        .strip_prefix("or greater")
        .or_else(|| rest.strip_prefix("or more"))
    {
        (Cmp::Ge, r)
    } else {
        (Cmp::Eq, rest)
    };
    let v = Box::new(n);
    let f = match stat {
        "power" => Filter::Power(cmp, v),
        "toughness" => Filter::Toughness(cmp, v),
        _ => Filter::ManaValue(cmp, v),
    };
    Some((f, rest))
}

/// Parses a target phrase. Returns (spec, rest).
pub fn parse_target(s: &str) -> Option<(TargetSpec, &str)> {
    let mut s = s.trim_start();
    let mut min = 1u32;
    let mut max = Value::Const(1);
    let mut another = false;
    if let Some(r) = strip(s, "up to ") {
        let (n, r2) = parse_number(r)?;
        min = 0;
        max = n;
        s = r2;
    } else if let Some(r) = strip(s, "one or two ") {
        min = 1;
        max = Value::Const(2);
        s = r;
    } else if let Some((n, r)) = parse_number(s) {
        if strip(r, "target").is_some() || strip(r, "other target").is_some() {
            if let Value::Const(k) = n {
                min = k as u32;
            }
            max = n;
            s = r;
        }
    }
    if let Some(r) = strip(s, "any number of ") {
        min = 0;
        max = Value::Const(99);
        s = r;
    }
    if let Some(r) = strip(s, "another target ").or_else(|| strip(s, "other target ")) {
        another = true;
        s = r;
    } else {
        s = strip(s, "target ")
            .or_else(|| strip(s, "targets "))
            .or_else(|| {
                // "any target"
                None
            })?;
    }
    let (what, rest) = if let Some(r) = strip(s, "player or planeswalker") {
        (
            TargetKind::ObjectOrPlayer(Filter::Type(CardType::Planeswalker), PlayerFilter::Any),
            r,
        )
    } else if let Some(r) = strip(s, "opponent or planeswalker") {
        (
            TargetKind::ObjectOrPlayer(
                Filter::Type(CardType::Planeswalker),
                PlayerFilter::Opponent,
            ),
            r,
        )
    } else if let Some(r) = strip(s, "creature or player") {
        (
            TargetKind::ObjectOrPlayer(Filter::creature(), PlayerFilter::Any),
            r,
        )
    } else if let Some(r) = strip(s, "players").or_else(|| strip(s, "player")) {
        (TargetKind::Player(PlayerFilter::Any), r)
    } else if let Some(r) = strip(s, "opponents").or_else(|| strip(s, "opponent")) {
        (TargetKind::Player(PlayerFilter::Opponent), r)
    } else if let Some(r) =
        strip(s, "spell or ability").or_else(|| strip(s, "activated or triggered ability"))
    {
        (TargetKind::SpellOrAbility(Filter::Any), r)
    } else if let Some(r) = strip(s, "activated ability").or_else(|| strip(s, "triggered ability"))
    {
        (TargetKind::Ability(Filter::Any), r)
    } else {
        let (f, _plural, r) = parse_object_phrase(s)?;
        // "target planeswalker that was activated this turn or tapped creature": an
        // alternative description after the first one's suffixes, ending the phrase. Not
        // after a list ("target Spirit, creature with disturb, or enchantment"), whose
        // suffixes belong to its last item only.
        let listed = s[..s.len() - r.len()].contains(',') || r.trim_start().starts_with(',');
        let alternative = if listed {
            None
        } else {
            strip(r, "or ").and_then(parse_object_phrase)
        };
        let (f, r) = match alternative {
            Some((f2, _, tail)) if end(tail).is_empty() && !filter_mentions_spell(&f2) => {
                (Filter::Or(vec![f, f2]), tail)
            }
            _ => (f, r),
        };
        let is_spell = filter_mentions_spell(&f);
        let f = if another {
            Filter::and(vec![f, Filter::Other])
        } else {
            f
        };
        if is_spell {
            (TargetKind::Spell(f), r)
        } else {
            (TargetKind::Object(f), r)
        }
    };
    let spec = TargetSpec {
        what,
        min,
        max,
        distinct_from: vec![],
        divide: None,
        chosen_by_opponent: false,
        text: String::new(),
        condition: None,
    };
    Some((spec, rest))
}

fn filter_mentions_spell(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_mentions_spell),
        _ => false,
    }
}

/// Parses "any target" or a target phrase.
pub fn parse_any_target(s: &str) -> Option<(TargetSpec, &str)> {
    if let Some(r) = strip(s, "any target") {
        return Some((TargetSpec::any_target(), r));
    }
    if let Some(r) = strip(s, "any other target") {
        let mut t = TargetSpec::any_target();
        t.what = TargetKind::AnyTarget;
        return Some((t, r));
    }
    parse_target(s)
}

/// Player phrases: "you", "each player", "each opponent", "target player", "that player",
/// "its controller", "its owner", "defending player". Returns (ref, target spec if any, rest).
pub fn parse_player(s: &str) -> Option<(PlayerRef, Option<TargetSpec>, &str)> {
    let t = s.trim_start();
    let pairs: [(&str, PlayerRef); 12] = [
        ("you ", PlayerRef::You),
        ("each player ", PlayerRef::EachPlayer),
        ("each opponent ", PlayerRef::EachOpponent),
        ("each other player ", PlayerRef::EachOtherPlayer),
        ("that player ", PlayerRef::TriggerPlayer),
        ("defending player ", PlayerRef::DefendingPlayer),
        (
            "its controller ",
            PlayerRef::ControllerOf(Box::new(Sel::Target(0))),
        ),
        ("its owner ", PlayerRef::OwnerOf(Box::new(Sel::Target(0)))),
        (
            "their controller ",
            PlayerRef::ControllerOf(Box::new(Sel::Target(0))),
        ),
        ("the active player ", PlayerRef::ActivePlayer),
        ("your opponents ", PlayerRef::EachOpponent),
        ("each of your opponents ", PlayerRef::EachOpponent),
    ];
    for (p, r) in pairs {
        if let Some(rest) = t.strip_prefix(p) {
            return Some((r, None, rest));
        }
    }
    if let Some(rest) = t.strip_prefix("target player ") {
        return Some((
            PlayerRef::Target(0),
            Some(TargetSpec::player(PlayerFilter::Any, "target player")),
            rest,
        ));
    }
    if let Some(rest) = t.strip_prefix("target opponent ") {
        return Some((
            PlayerRef::Target(0),
            Some(TargetSpec::player(
                PlayerFilter::Opponent,
                "target opponent",
            )),
            rest,
        ));
    }
    None
}

/// Maps a count phrase like "a card", "two cards", "x cards" to a value.
pub fn parse_card_count(s: &str) -> Option<(Value, &str)> {
    let (n, rest) = parse_number(s)?;
    let rest = strip(rest, "cards").or_else(|| strip(rest, "card"))?;
    Some((n, rest))
}

/// Trims trailing period and whitespace.
pub fn end(s: &str) -> &str {
    s.trim().trim_end_matches('.').trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_phrases() {
        let (f, plural, rest) = parse_object_phrase("nontoken creatures you control get").unwrap();
        assert!(plural);
        assert_eq!(rest.trim(), "get");
        assert!(format!("{f:?}").contains("Token"));
        let (f, _, _) = parse_object_phrase("artifact or enchantment").unwrap();
        assert!(matches!(f, Filter::Or(_)));
        let (f, _, _) = parse_object_phrase("creature card in your graveyard").unwrap();
        assert_eq!(f.zone(), Some(ZoneKind::Graveyard));
        assert!(parse_object_phrase("goblins you control").is_some());
    }

    #[test]
    fn targets() {
        let (t, rest) = parse_target("target creature you control.").unwrap();
        assert!(matches!(t.what, TargetKind::Object(_)));
        assert_eq!(end(rest), "");
        let (t, _) = parse_target("up to two target creatures").unwrap();
        assert_eq!(t.min, 0);
        let (t, _) = parse_any_target("any target").unwrap();
        assert!(matches!(t.what, TargetKind::AnyTarget));
        let (t, _) = parse_target("target spell").unwrap();
        assert!(matches!(t.what, TargetKind::Spell(_)));
    }
}
