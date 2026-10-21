//! Qualities of objects named after the noun of an object phrase:
//!
//! - "with {X} in their mana cost" (CR 107.3, 202.1);
//! - "that has an activated ability with {T} in its cost" (CR 602.1, 107.5);
//! - "that doesn't have a +1/+1 counter on it", "that don't have counters on them";
//! - "that didn't attack or enter this turn", "that attacked or entered this turn";
//! - "that don't have a name", "that doesn't have a name" (CR 201.2a, 708.2a);
//! - "with two or more card types" (CR 205.2a);
//! - "with a legendary Equipment attached to it" (CR 301.5);
//! - "whose controller controls an Island";
//! - "that doesn't have the same name as a token you control", "that doesn't have the
//!   same name as another permanent you control" (CR 201.2).
//!
//! Each quality is checked on the object the phrase describes ([`vars::TESTED`] in
//! [`Filter::ValueCmp`]) or by a named predicate in `kw/filter_qualities_rules.rs`.

use super::FilterSuffixPattern;
use crate::ability::*;
use crate::oracle::patterns::statics_conditions::counters_on_it;
use crate::oracle::phrases::*;

/// `Filter::Custom`: an object with no name (CR 201.2a; a face-down permanent, CR 708.2a).
pub const NO_NAME: &str = "has no name";
/// `Filter::Custom`: an object with an activated ability with {T} in its cost.
pub const TAP_ABILITY: &str = "has an activated ability with {T} in its cost";
/// `Filter::Custom`: an object that has a color that's the most common color among all
/// permanents or tied for most common (each permanent counts once for each of its
/// colors, CR 105.2).
pub const MOST_COMMON_COLOR: &str = "shares a color with the most common color among all permanents";
/// `Filter::Custom` prefix: "that you've put one or more [kind] counters on this turn"
/// (CR 122.6: the player who put them), followed by the kind.
pub const COUNTERS_PUT_BY_YOU_THIS_TURN: &str = "you put counters on it this turn:";
/// `Filter::Custom`: an object with two or more card types.
pub const TWO_CARD_TYPES: &str = "has two or more card types";

/// The player "that player" refers to in "a card that player owns", until the effect
/// parser puts in what it means there ([`that_player_owns`]).
pub const REFERENCED_PLAYER: Var = vars::USER + 9330;

/// "You may put a card that has an Adventure that player owns from exile into that
/// player's graveyard", "remove a hit counter from a card that player owns in exile": the
/// player the text named before.
fn that_player_owns(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    use super::oracle_hardening_referents::is_no_player_referent;
    let l = end(l);
    if !l.contains("that player owns") || is_no_player_referent(&b.it_player) {
        return None;
    }
    let who = match &b.it_player {
        PlayerRef::You => return None,
        p => p.clone(),
    };
    // A card that player owns goes to its owner's graveyard: that player's.
    let s = l
        .replace("that player owns", "the referenced player owns")
        .replace("into that player's graveyard", "into its owner's graveyard");
    let saved = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&s, b)?;
    let needle = serde_json::to_string(&PlayerRef::Var(REFERENCED_PLAYER)).ok()?;
    let with = serde_json::to_string(&who).ok()?;
    let j = serde_json::to_string(&e).ok()?;
    if !j.contains(&needle) {
        b.targets.truncate(saved);
        return None;
    }
    serde_json::from_str(&j.replace(&needle, &with)).ok()
}

inventory::submit! { super::EffectPattern { name: "[instruction about objects] that player owns", priority: 40, parse: that_player_owns } }

fn tested() -> Sel {
    Sel::Var(vars::TESTED)
}

/// A filter that's true when any object matches `f` (evaluated with the tested object
/// bound, so `f` may mention it).
fn any_matching(f: Filter) -> Filter {
    Filter::ValueCmp(Box::new(Value::Count(f)), Cmp::Ge, Box::new(Value::c(1)))
}

fn qualities<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for p in [
        "with {x} in their mana costs",
        "with {x} in their mana cost",
        "with {x} in its mana cost",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::HasX, r));
        }
    }
    for p in [
        "that has an activated ability with {t} in its cost",
        "that have an activated ability with {t} in its cost",
        "with an activated ability with {t} in its cost",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::Custom(TAP_ABILITY.into()), r));
        }
    }
    for p in [
        "that don't have a name",
        "that doesn't have a name",
        "without a name",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::Custom(NO_NAME.into()), r));
        }
    }
    // "that you've put one or more +1/+1 counters on this turn"
    if let Some(x) = t.strip_prefix("that you've put one or more ") {
        let (kind, r) = x.split_once(" counters on this turn")?;
        if kind.contains(' ') {
            return None;
        }
        return Some((
            Filter::Custom(format!("{COUNTERS_PUT_BY_YOU_THIS_TURN}{kind}").into()),
            r,
        ));
    }
    // "cards of each of the sacrificed creature's colors": a card with any of its colors
    // (CR 105.2).
    if let Some(x) = t.strip_prefix("of each of ") {
        let i = x.find("'s colors")?;
        let (sel, r) = super::filters_relational::object_in(&x[..i])?;
        if !r.trim().is_empty() {
            return None;
        }
        return Some((Filter::SharesColor(Box::new(sel)), &x[i + "'s colors".len()..]));
    }
    // "with doctor's companion" (CR 702.124m).
    if let Some(r) = t.strip_prefix("with doctor's companion") {
        return Some((
            Filter::Custom(crate::kw::dig_filters::DOCTORS_COMPANION.into()),
            r,
        ));
    }
    // "without mana value of the chosen quality" (Ashling's Prerogative).
    if let Some(r) = t.strip_prefix("without mana value of the chosen quality") {
        return Some((Filter::not(Filter::ManaValueOfChosenQuality), r));
    }
    // "with mana value, power, or toughness equal to the chosen number": any of them.
    if let Some(r) = t.strip_prefix("with mana value, power, or toughness equal to the chosen number") {
        let v = || Box::new(Value::Chosen);
        return Some((
            Filter::Or(vec![
                Filter::ManaValue(Cmp::Eq, v()),
                Filter::Power(Cmp::Eq, v()),
                Filter::Toughness(Cmp::Eq, v()),
            ]),
            r,
        ));
    }
    // "that shares a color or mana value with the exiled card": either.
    for p in [
        "that shares a color or mana value with ",
        "that share a color or mana value with ",
    ] {
        if let Some(x) = t.strip_prefix(p) {
            let (sel, rest) = super::filters_relational::object_in(x)?;
            return Some((
                Filter::Or(vec![
                    Filter::SharesColor(Box::new(sel.clone())),
                    Filter::ManaValue(Cmp::Eq, Box::new(Value::ManaValueOf(Box::new(sel)))),
                ]),
                rest,
            ));
        }
    }
    if let Some(r) = t.strip_prefix("with two or more card types") {
        return Some((Filter::Custom(TWO_CARD_TYPES.into()), r));
    }
    // "that doesn't have a +1/+1 counter on it", "that don't have counters on them"
    for p in ["that doesn't have ", "that don't have ", "without "] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        for end_ in [" on it", " on them"] {
            let Some(i) = x.find(end_) else {
                continue;
            };
            let body = &x[..i];
            let r = &x[i + end_.len()..];
            let body = body
                .strip_prefix("any ")
                .map(|k| format!("a {}", k.trim_end_matches('s')))
                .unwrap_or_else(|| body.to_string());
            let (cmp, n, kind) = counters_on_it(&format!("{body} on it"))?;
            if cmp != Cmp::Ge || !matches!(n, Value::Const(1)) {
                return None;
            }
            return Some((Filter::not(Filter::HasCounter(kind)), r));
        }
    }
    // "that didn't attack or enter this turn", "that attacked or entered this turn"
    for (p, neg) in [("that didn't ", true), ("that attacked or entered", false)] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        let r = if neg {
            x.strip_prefix("attack or enter this turn")?
        } else {
            x.strip_prefix(" this turn")?
        };
        let f = Filter::Or(vec![Filter::AttackedThisTurn, Filter::EnteredThisTurn]);
        return Some((if neg { Filter::not(f) } else { f }, r));
    }
    // "with a legendary Equipment attached to it"
    if let Some(x) = t.strip_prefix("with a ").or_else(|| t.strip_prefix("with an ")) {
        if let Some(i) = x.find(" attached to it") {
            let (f, plural, rest) = parse_object_phrase(&x[..i])?;
            if plural || !rest.trim().is_empty() {
                return None;
            }
            let attached = Filter::and(vec![f, Filter::AttachedTo(Box::new(tested()))]);
            return Some((any_matching(attached), &x[i + " attached to it".len()..]));
        }
    }
    // "whose controller controls an Island"
    if let Some(x) = t.strip_prefix("whose controller controls ") {
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))?;
        let (f, plural, rest) = parse_object_phrase(x)?;
        if plural {
            return None;
        }
        return Some((
            Filter::ControllerMatches(Box::new(PlayerFilter::Controls(
                Box::new(f),
                Cmp::Ge,
                Box::new(Value::c(1)),
            ))),
            rest,
        ));
    }
    // "that doesn't have the same name as a token you control", "... as another
    // permanent you control": no such other object has a name in common with it.
    for p in [
        "that doesn't have the same name as ",
        "that don't have the same name as ",
    ] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an ")).unwrap_or(x);
        let (f, plural, rest) = parse_object_phrase(x)?;
        if plural {
            return None;
        }
        // "another": other than the object described (not the source).
        let f = other_than_tested(f);
        let same = Filter::and(vec![
            f,
            Filter::SameNameAs(Box::new(tested())),
            Filter::not(Filter::In(Box::new(tested()))),
        ]);
        return Some((Filter::not(any_matching(same)), rest));
    }
    None
}

/// A list of card names "A, B, and C" at the start of `x` (lowercase; "~" is the card's
/// own name), each name from the card database, and the rest of the text.
fn name_list(x: &str) -> Option<(Vec<smol_str::SmolStr>, &str)> {
    let raw = crate::oracle::raw_text();
    let me = crate::oracle::card_name();
    let (head, last_and_rest) = x.split_once(", and ")?;
    let mut names: Vec<String> = head.split(", ").map(|n| n.to_string()).collect();
    // The last name ends where the original text's name ends: find it in the raw text.
    let lower_raw = raw.to_lowercase();
    let last = names_from_raw(&lower_raw, last_and_rest, &me)?;
    let rest = &last_and_rest[last.len()..];
    names.push(last);
    let mut out = vec![];
    for n in names {
        let n = if n == "~" { me.to_lowercase() } else { n };
        // Recover the printed capitalization from the raw text.
        let i = lower_raw.find(&n)?;
        out.push(smol_str::SmolStr::new(&raw[i..i + n.len()]));
    }
    Some((out, rest))
}

/// Every card name (lowercase), from the card data.
fn card_names() -> &'static std::collections::HashSet<String> {
    static N: std::sync::OnceLock<std::collections::HashSet<String>> = std::sync::OnceLock::new();
    N.get_or_init(|| {
        mtg_data::cards()
            .iter()
            .flat_map(|c| {
                let mut v = vec![c.name.to_lowercase()];
                v.extend(c.name.split(" // ").map(|n| n.to_lowercase()));
                v
            })
            .collect()
    })
}

/// The longest prefix of `s` that is a name in the raw text followed by a non-name word.
fn names_from_raw(lower_raw: &str, s: &str, me: &str) -> Option<String> {
    if s.starts_with("~") {
        return Some("~".to_string());
    }
    let _ = me;
    let words: Vec<&str> = s.split(' ').collect();
    for k in (1..=words.len().min(5)).rev() {
        let joined = words[..k].join(" ");
        let cand = joined.trim_end_matches([',', '.']).to_string();
        if lower_raw.contains(&cand) && card_names().contains(&cand) {
            return Some(cand);
        }
    }
    None
}

fn other_than_tested(f: Filter) -> Filter {
    match f {
        Filter::Other => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(other_than_tested).collect()),
        f => f,
    }
}

inventory::submit! { FilterSuffixPattern { name: "filter qualities", priority: 120, parse: qualities } }

/// "named Sword of Kaldra, ~, and Helm of Kaldra": any of the names (CR 201.2). Before the
/// single-name qualifiers, which would read only the first name.
fn named_list<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let x = t.strip_prefix("named ")?;
    if !x.contains(", and ") {
        return None;
    }
    let (list, rest) = name_list(x)?;
    Some((Filter::Or(list.into_iter().map(Filter::Named).collect()), rest))
}

inventory::submit! { FilterSuffixPattern { name: "filter qualities: named A, B, and C", priority: 1, parse: named_list } }

/// "If you control Equipment named ~, Sword of Kaldra, and Shield of Kaldra, create
/// Kaldra, ... token. Attach those Equipment to it." (Helm of Kaldra): you control an
/// object with each of the names; "those [objects]" are the ones you control with any of
/// them.
fn if_you_control_named_list(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if you control ")?;
    let (noun, x) = r.split_once(" named ")?;
    let (kind, _, tail) = parse_object_phrase(noun)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let (names, rest) = name_list(x)?;
    let rest = rest.strip_prefix(", ")?;
    let mine = |f: Filter| Filter::and(vec![kind.clone(), f, Filter::ControlledBy(PlayerRel::You)]);
    let cond = Condition::And(
        names
            .iter()
            .map(|n| Condition::Exists(mine(Filter::Named(n.clone()))))
            .collect(),
    );
    let all = Sel::All(mine(Filter::Or(names.into_iter().map(Filter::Named).collect())));
    b.named.push((format!("those {noun}"), all));
    let then = crate::oracle::effects::parse_clause(rest, b)?;
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { super::EffectPattern { name: "if you control [objects] named A, B, and C, ...", priority: 50, parse: if_you_control_named_list } }

/// "Attach those Equipment to it." after the objects were named and a token created
/// (Helm of Kaldra): each of them to the token (CR 701.3).
fn attach_those_to_it(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("attach ")?;
    let (those, to) = r.split_once(" to ")?;
    if to != "it" || !those.starts_with("those ") {
        return None;
    }
    let what = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == those)
        .map(|(_, s)| s.clone())?;
    if !matches!(b.it, Sel::Var(v) if v == vars::CREATED) {
        return None;
    }
    Some(Effect::Attach {
        what,
        to: b.it.clone(),
    })
}

inventory::submit! { super::EffectPattern { name: "attach those [objects] to it", priority: 50, parse: attach_those_to_it } }

/// "Each creature without mana value of the chosen quality enters tapped." (CR 614.1c):
/// the same as "[objects] enter tapped".
fn each_enters_tapped(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("each ")?.strip_suffix(" enters tapped")?;
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural || !rest.trim().is_empty() || super::filters_relational::mentions_referent(&f) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(f),
            action: ReplacementAction::EnterTapped,
            self_replacement: false,
            optional: false,
        }))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "each [object] enters tapped", priority: 100, parse: each_enters_tapped } }

/// "a creature an opponent controls with the greatest power among creatures that player
/// controls" in a trigger condition (Kraven the Hunter): "that player" is the controller
/// of the object the condition is about. Top-level, the object is trivially controlled by
/// its controller; inside a comparison, it's the tested object's controller.
pub fn they_as_tested_controller(f: Filter) -> Filter {
    let they = Filter::ControlledBy(super::filters_relational::THEY);
    let they_json = match serde_json::to_string(&they) {
        Ok(j) => j,
        Err(_) => return f,
    };
    let Ok(j) = serde_json::to_string(&f) else {
        return f;
    };
    if !j.contains(&they_json) {
        return f;
    }
    let tested_ctl = Filter::ControlledByPlayer(Box::new(PlayerRef::ControllerOf(Box::new(
        Sel::Var(vars::TESTED),
    ))));
    fn walk(f: Filter, top: bool, they: &Filter, inner: &Filter) -> Filter {
        let same = |x: &Filter| format!("{x:?}") == format!("{they:?}");
        match f {
            x if same(&x) => {
                if top {
                    Filter::Any
                } else {
                    inner.clone()
                }
            }
            Filter::And(v) => Filter::and(v.into_iter().map(|x| walk(x, top, they, inner)).collect()),
            Filter::ValueCmp(a, c, b) => {
                let fix = |v: Value| -> Value {
                    let Ok(j) = serde_json::to_string(&v) else {
                        return v;
                    };
                    let tj = serde_json::to_string(they).unwrap_or_default();
                    let ij = serde_json::to_string(inner).unwrap_or_default();
                    serde_json::from_str(&j.replace(&tj, &ij)).unwrap_or(v)
                };
                Filter::ValueCmp(Box::new(fix(*a)), c, Box::new(fix(*b)))
            }
            x => x,
        }
    }
    walk(f, true, &they, &tested_ctl)
}

/// A list of kinds before a shared noun where an item has words of its own (CR 205: a
/// supertype, subtype or card type each describes objects; a list of them joined by "or"
/// or "and" means objects of any of the kinds): "legendary or Rat card" (Nashi, Moon's
/// Legacy: legendary cards and Rat cards), "Spider and legendary creature you control"
/// (Miles Morales: Spiders and legendary creatures), "instant, sorcery, and/or Tamiyo
/// planeswalker cards" (The Tale of Tamiyo). Each item is read with the last item's head
/// noun and what follows it; what all of them share (the zone, the controller) is said
/// once. Lists whose items are single words of one kind ("red or white creature",
/// "Spider and Rogue") are the core grammar's.
pub fn listed_kinds(s: &str) -> Option<(Filter, bool, &str)> {
    let s = s.trim_start();
    let words: Vec<&str> = s.split(' ').collect();
    let kind_word = |w: &str| head_noun(w).is_some() || adjective(w).is_some();
    // The items before the last: single kind words, separated by commas and a final
    // conjunction.
    let mut items: Vec<&str> = Vec::new();
    let mut i = 0;
    let mut conj = None;
    while i < words.len() {
        let w = words[i];
        if matches!(w, "or" | "and" | "and/or") && !items.is_empty() {
            conj = Some(w);
            i += 1;
            break;
        }
        let bare = w.trim_end_matches(',');
        if bare.is_empty() || !kind_word(bare) || matches!(bare, "token" | "tokens") {
            return None;
        }
        // Without a comma, only one item before the conjunction ("legendary or").
        if !w.ends_with(',')
            && words
                .get(i + 1)
                .is_some_and(|n| !matches!(*n, "or" | "and" | "and/or"))
        {
            return None;
        }
        items.push(bare);
        i += 1;
    }
    let conj = conj?;
    // The last item's words up to its head noun: the longest run of kind words.
    let core_len = words[i..]
        .iter()
        .take_while(|w| !w.ends_with(',') && kind_word(w))
        .count();
    if core_len < 2 {
        return None;
    }
    let core = &words[i..i + core_len];
    let head = core[core_len - 1];
    if head_noun(head).is_none() {
        return None;
    }
    // Which lists: items that are adjectives only, before a last item starting with a
    // noun ("legendary or Rat card"); or items that are nouns, joined by "and" ("and/or"),
    // before a qualified last item ("instant, sorcery, and/or Tamiyo planeswalker cards")
    // or one starting with an adjective ("Spider and legendary creature"). Not a list of
    // adjectives of one kind ("blue or black noncreature permanent": both noncreature),
    // nor complete alternatives joined by "or" ("land or nonblack creature", "spell or
    // nonland permanent": the core grammar's).
    let noun_only = |w: &str| head_noun(w).is_some() && adjective(w).is_none();
    let adjective_only = |w: &str| head_noun(w).is_none() && adjective(w).is_some();
    let ok = if items.iter().all(|w| adjective_only(w)) {
        noun_only(core[0])
    } else if items.iter().all(|w| head_noun(w).is_some()) {
        conj != "or" && (core_len >= 3 || adjective_only(core[0]))
    } else {
        false
    };
    if !ok {
        return None;
    }
    // A generic head ("card", "permanent", "spell") goes with every item ("legendary or
    // Rat card": legendary cards; "instant ... cards": instant cards); a type or subtype
    // head doesn't go with an item that's a noun itself ("Spider and legendary creature":
    // Spiders).
    let generic = matches!(
        head_noun(head),
        Some(Filter::Card | Filter::Permanent | Filter::Spell)
    );
    let last_text = words[i..].join(" ");
    let (f_last, plural, rest) = parse_object_phrase(&last_text)?;
    let consumed = &last_text[..last_text.len() - rest.len()];
    let tail = consumed
        .strip_prefix(&core.join(" "))
        .unwrap_or("")
        .to_string();
    let mut alts = Vec::new();
    for x in &items {
        let phrase = if generic || head_noun(x).is_none() {
            format!("{x} {head}{tail}")
        } else {
            format!("{x}{tail}")
        };
        let (f, _, r) = parse_object_phrase(&phrase)?;
        if !r.trim().is_empty() {
            return None;
        }
        alts.push(f);
    }
    alts.push(f_last);
    // What every alternative says (the zone, the controller, "card"): said once.
    let parts = |f: &Filter| -> Vec<Filter> {
        match f {
            Filter::And(v) => v.clone(),
            f => vec![f.clone()],
        }
    };
    let key = |f: &Filter| serde_json::to_string(f).unwrap_or_default();
    let common: Vec<Filter> = parts(&alts[0])
        .into_iter()
        .filter(|c| alts.iter().all(|a| parts(a).iter().any(|x| key(x) == key(c))))
        .collect();
    let own: Vec<Filter> = alts
        .iter()
        .map(|a| {
            Filter::and(
                parts(a)
                    .into_iter()
                    .filter(|x| !common.iter().any(|c| key(c) == key(x)))
                    .collect(),
            )
        })
        .collect();
    if own.iter().any(|f| matches!(f, Filter::Any)) {
        return None;
    }
    let mut v = common;
    v.push(Filter::Or(own));
    let rest_start = s.len() - rest.len();
    Some((Filter::and(v), plural, &s[rest_start..]))
}

/// A keyword at the start of `s` (the longest name that matches whole words).
fn keyword_at(s: &str) -> Option<(crate::keywords::KeywordKind, &str)> {
    crate::keywords::KeywordKind::ALL
        .iter()
        .filter_map(|k| {
            let n = k.name().to_lowercase();
            let r = s.strip_prefix(n.as_str())?;
            (r.is_empty() || r.starts_with([' ', ',', '.'])).then_some((n.len(), *k, r))
        })
        .max_by_key(|(n, _, _)| *n)
        .map(|(_, k, r)| (k, r))
}

/// "without first strike, double strike, or vigilance", "without flying or reach": none
/// of the keywords (CR 702).
pub fn without_keyword_list(t: &str) -> Option<(Filter, &str)> {
    let mut r = t.strip_prefix("without ")?;
    let mut kws = Vec::new();
    loop {
        let (k, rest) = keyword_at(r)?;
        kws.push(Filter::HasKeyword(k));
        if let Some(x) = rest
            .strip_prefix(", or ")
            .or_else(|| rest.strip_prefix(" or "))
        {
            let (k, rest) = keyword_at(x)?;
            kws.push(Filter::HasKeyword(k));
            r = rest;
            break;
        }
        r = rest.strip_prefix(", ")?;
    }
    if kws.len() < 2 {
        return None;
    }
    Some((Filter::not(Filter::Or(kws)), r))
}

/// The qualifier a phrase reads after a stand-in head noun ("card"), and the rest.
fn qualifier_after_card(q: &str) -> Option<(Filter, String)> {
    let probe = format!("card {q}");
    let (f, _, rest) = parse_object_phrase(&probe)?;
    let Filter::And(mut v) = f else { return None };
    if !matches!(v.first(), Some(Filter::Card)) {
        return None;
    }
    v.remove(0);
    Some((Filter::and(v), rest.to_string()))
}

/// "that's an artifact or that has mana value 3 or less" (Desdemona, Freedom's Edge):
/// either relative clause.
fn that_or_that<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let body = t.strip_prefix("that")?;
    let i = body.find(" or that ")?;
    let (a, b) = (&body[..i], &body[i + " or that".len()..]);
    let clause = |c: &str| -> Option<(Filter, String)> {
        let full = format!("that{c}");
        // "that has mana value 3 or less": "with mana value 3 or less".
        if let Some(x) = full
            .strip_prefix("that has ")
            .or_else(|| full.strip_prefix("that have "))
        {
            if let Some(r) = qualifier_after_card(&format!("with {x}")) {
                return Some(r);
            }
        }
        qualifier_after_card(&full)
    };
    let (fa, ra) = clause(a)?;
    if !ra.trim().is_empty() {
        return None;
    }
    let (fb, rb) = clause(b)?;
    if matches!(fa, Filter::Any) || matches!(fb, Filter::Any) {
        return None;
    }
    let n = rb.len();
    Some((Filter::Or(vec![fa, fb]), &t[t.len() - n..]))
}

/// Qualifiers the core phrase parser reads before its own (it would read a shorter part
/// of them): [`without_keyword_list`], [`that_or_that`].
pub fn early_suffix(t: &str) -> Option<(Filter, &str)> {
    without_keyword_list(t).or_else(|| that_or_that(t, &Filter::Any))
}

/// "each creature that player controls with power greater than the number of cards in
/// their hand" (Noetic Scales): "their" is the player who controls the creature, so its
/// controller's hand.
fn their_hand<'a>(t: &'a str, so_far: &Filter) -> Option<(Filter, &'a str)> {
    if !t.starts_with("with ") {
        return None;
    }
    let i = t.find(" their hand")?;
    let parts = match so_far {
        Filter::And(v) => v.clone(),
        f => vec![f.clone()],
    };
    let one_player = parts.iter().any(|p| {
        matches!(
            p,
            Filter::ControlledBy(
                PlayerRel::TriggerPlayer
                    | PlayerRel::Target(_)
                    | PlayerRel::Iterated
                    | PlayerRel::Var(_)
            ) | Filter::ControlledByPlayer(_)
        )
    });
    if !one_player {
        return None;
    }
    let reworded = format!("{} its controller's hand{}", &t[..i], &t[i + " their hand".len()..]);
    let (f, rest) = qualifier_after_card(&reworded)?;
    if matches!(f, Filter::Any) {
        return None;
    }
    let n = rest.len();
    Some((f, &t[t.len() - n..]))
}

inventory::submit! { FilterSuffixPattern { name: "with [stat] ... in their hand (its controller's)", priority: 500, parse: their_hand } }
