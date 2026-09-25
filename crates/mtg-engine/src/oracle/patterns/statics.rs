//! Static abilities of permanents (CR 604, 611.3, 613) as a small grammar:
//!
//! ```text
//! [as long as COND, | during your turn, | during turns other than yours,]*
//!     SUBJECT PREDICATE {, | and | , and} PREDICATE ...  [, where X is VALUE]
//!     [as long as COND | during your turn]
//! ```
//!
//! * SUBJECT: `~`, the object the source is attached to ("enchanted creature",
//!   "equipped creature", "enchanted land", ...), a pronoun for an object named in the
//!   leading condition ("as long as enchanted creature is white, it ..."), or a group
//!   ("creatures you control", "other Elves you control", "all Sliver creatures",
//!   "each creature you control with a +1/+1 counter on it", "~ and other Knights you
//!   control").
//! * PREDICATE: P/T changes (layer 7c, CR 613.4c), optionally "for each ..."; base P/T
//!   (layer 7b); keyword and quoted-ability grants (layer 6, CR 613.1f), where a quoted
//!   ability is compiled recursively and "this creature" in it means the object that has
//!   it (see [`granted_abilities`]; the controller of that object activates it, CR
//!   301.5d, 303.4e); losing abilities; type changes (layer 4, CR 205.1, 305.7); color
//!   changes (layer 5); and restrictions/requirements on the affected objects ("can't
//!   block", "attacks each combat if able", CR 613.11).
//! * COND: see [`super::statics_conditions`]. A conditional static applies whenever its
//!   condition is true (CR 611.3a).

use super::statics_conditions::parse_static_condition;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::StaticPattern;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

// ---------------------------------------------------------------------------
// Quotes
// ---------------------------------------------------------------------------

/// Replaces each quoted ability with a `"#k"` placeholder so that splitting on commas
/// and "and" never looks inside quotes. Returns the masked text and the quoted texts.
pub(crate) fn mask_quotes(l: &str) -> Option<(String, Vec<String>)> {
    let mut out = String::with_capacity(l.len());
    let mut quotes = Vec::new();
    let mut rest = l;
    while let Some(i) = rest.find('"') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let j = after.find('"')?;
        out.push_str(&format!("\"#{}\"", quotes.len()));
        quotes.push(after[..j].to_string());
        rest = &after[j + 1..];
    }
    out.push_str(rest);
    Some((out, quotes))
}

/// The quoted segments of a text, in order.
fn quoted_segments(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('"') {
        let after = &rest[i + 1..];
        let Some(j) = after.find('"') else { break };
        out.push(&after[..j]);
        rest = &after[j + 1..];
    }
    out
}

/// Whether a quoted ability (normalized) names the card itself rather than "this
/// creature": normalization writes both as `~`, but in an ability granted to another
/// object only "this creature" means that object; the card's name still means the card
/// (CR 201.5a: "Equipped creature has '{T}, Sacrifice Blazing Torch: ...'"). Such
/// abilities are left unsupported. Unknown provenance counts as naming the card.
fn quote_names_card(normalized: &str, ctx: &CompileContext) -> bool {
    if !normalized.contains('~') {
        return false;
    }
    let raw = crate::oracle::raw_text();
    let tl = TypeLine::default();
    let anonymous = CompileContext {
        card_name: "\u{1}",
        full_name: "\u{1}",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    for q in quoted_segments(&raw) {
        let with_name = crate::oracle::normalize(q, ctx);
        if with_name.trim() == normalized.trim() {
            return with_name != crate::oracle::normalize(q, &anonymous);
        }
    }
    true
}

/// Compiles a quoted ability granted to the objects matched by a static ability
/// (CR 613.1f). `~` in it refers to the object that has the ability.
fn granted_abilities(
    quote_lower: &str,
    text: &str,
    hint: CardType,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let orig = quoted_segments(text)
        .into_iter()
        .find(|q| q.to_lowercase() == quote_lower)?;
    if quote_names_card(orig, ctx) {
        return None;
    }
    let mut tl = TypeLine::default();
    tl.card_types.insert(hint);
    let gctx = CompileContext {
        card_name: "\u{1}",
        full_name: "\u{1}",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let blocks = crate::oracle::split_abilities(orig);
    if blocks.len() != 1 {
        return None;
    }
    let v = crate::oracle::parse_ability(&blocks[0], &gctx)?;
    if v.is_empty()
        || v.iter()
            .any(|a| matches!(a.kind, AbilityKind::Unsupported(_)))
    {
        return None;
    }
    Some(v)
}

// ---------------------------------------------------------------------------
// Object phrases
// ---------------------------------------------------------------------------

/// Suffixes the shared object-phrase parser doesn't know.
fn extra_suffix(t: &str) -> Option<(Filter, &str)> {
    for (p, f) in [
        ("on the battlefield", Filter::Any),
        ("in all graveyards", Filter::InZone(ZoneKind::Graveyard)),
        ("in graveyards", Filter::InZone(ZoneKind::Graveyard)),
        ("that are enchanted", Filter::Enchanted),
        ("that's enchanted", Filter::Enchanted),
        ("that are equipped", Filter::Equipped),
        ("that's equipped", Filter::Equipped),
        ("that are tapped", Filter::Tapped),
        ("that are untapped", Filter::Untapped),
        ("that are attacking", Filter::Attacking),
        ("that are modified", Filter::Modified),
        ("that's modified", Filter::Modified),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',']) {
                return Some((f, r));
            }
        }
    }
    // "with +1/+1 counters on them", "with one or more +1/+1 counters on it",
    // "with counters on them", "with a +1/+1 counter on it".
    if let Some(r) = t.strip_prefix("with ") {
        for tail in [" on them", " on it"] {
            if let Some(i) = r.find(tail) {
                let body = &r[..i];
                let rest = &r[i + tail.len()..];
                if !(rest.is_empty() || rest.starts_with([' ', ','])) {
                    continue;
                }
                let body = body
                    .strip_prefix("one or more ")
                    .or_else(|| body.strip_prefix("a "))
                    .unwrap_or(body);
                if body == "counters" || body == "counter" {
                    return Some((Filter::HasCounter(None), rest));
                }
                if let Some((kind, c)) = body.split_once(' ') {
                    if (c == "counters" || c == "counter")
                        && (kind.starts_with('+')
                            || kind.starts_with('-')
                            || kind.chars().all(|ch| ch.is_alphabetic()))
                    {
                        return Some((Filter::HasCounter(Some(kind.into())), rest));
                    }
                }
            }
        }
    }
    // "with deathtouch", "without flying", "with first strike".
    let (neg, r) = if let Some(r) = t.strip_prefix("without ") {
        (true, r)
    } else {
        (false, t.strip_prefix("with ")?)
    };
    let words: Vec<&str> = r.splitn(3, ' ').collect();
    for n in [2usize, 1] {
        if words.len() < n {
            continue;
        }
        let name = words[..n].join(" ");
        let name = name.trim_end_matches(',');
        if let Some(k) = KeywordKind::from_name(name) {
            let consumed: usize = words[..n].iter().map(|w| w.len()).sum::<usize>() + (n - 1);
            let consumed = if r[..consumed.min(r.len())].ends_with(',') {
                consumed - 1
            } else {
                consumed
            };
            let f = Filter::HasKeyword(k);
            return Some((if neg { Filter::not(f) } else { f }, &r[consumed..]));
        }
    }
    None
}

/// [`parse_object_phrase`] plus [`extra_suffix`]es, in any order.
pub(crate) fn object_phrase(s: &str) -> Option<(Filter, bool, &str)> {
    let (f, plural, mut rest) = parse_object_phrase(s)?;
    let mut parts = vec![f];
    loop {
        let t = rest.trim_start();
        if t.is_empty() {
            rest = t;
            break;
        }
        if let Some((g, r)) = extra_suffix(t) {
            parts.push(g);
            rest = r;
            continue;
        }
        // The shared parser's suffixes, after one of ours ("with flying you control").
        const SUFFIX_STARTS: &[&str] = &[
            "you ",
            "you control",
            "an opponent",
            "your opponents",
            "target ",
            "in ",
            "from ",
            "with ",
            "without ",
            "that",
            "defending",
        ];
        if SUFFIX_STARTS.iter().any(|p| t.starts_with(p)) {
            let probe = format!("card {t}");
            if let Some((g, _, r)) = parse_object_phrase(&probe) {
                let consumed = t.len().saturating_sub(r.len());
                if consumed > 0 && r.len() <= t.len() {
                    parts.push(g);
                    rest = &t[consumed..];
                    continue;
                }
            }
        }
        break;
    }
    Some((Filter::and(parts), plural, rest))
}

/// An object phrase that must be the whole text.
fn whole_object_phrase(s: &str) -> Option<(Filter, bool)> {
    let (f, plural, rest) = object_phrase(s)?;
    end(rest).is_empty().then_some((f, plural))
}

/// Lists of nouns sharing suffixes: "Wolves and Werewolves you control" means "Wolves
/// or Werewolves you control" (either kind is affected).
fn union_nouns(s: &str) -> String {
    s.replace(", and ", ", or ").replace(" and ", " or ")
}

// ---------------------------------------------------------------------------
// Subjects
// ---------------------------------------------------------------------------

/// What a static ability affects.
#[derive(Clone, Debug)]
pub(crate) struct Subject {
    pub filter: Filter,
    /// The single object "it"/"its" refers to, when the subject is one object.
    pub it: Option<Sel>,
    /// A permanent type the affected objects have (for compiling granted abilities).
    pub hint: CardType,
    /// The affected objects are lands (so basic land types can be set, CR 305.7).
    pub lands: bool,
    /// The affected objects are creatures.
    pub creatures: bool,
}

fn filter_mentions(f: &Filter, pred: &dyn Fn(&Filter) -> bool) -> bool {
    if pred(f) {
        return true;
    }
    match f {
        Filter::And(v) => v.iter().any(|x| filter_mentions(x, pred)),
        Filter::Or(v) => !v.is_empty() && v.iter().all(|x| filter_mentions(x, pred)),
        _ => false,
    }
}

fn is_land_filter(f: &Filter) -> bool {
    filter_mentions(f, &|x| match x {
        Filter::Type(CardType::Land) => true,
        Filter::Subtype(s) => subtype_kind(s) == Some(SubtypeKind::Land),
        _ => false,
    })
}

fn is_creature_filter(f: &Filter) -> bool {
    filter_mentions(f, &|x| match x {
        Filter::Type(CardType::Creature) => true,
        Filter::Subtype(s) => is_creature_type(s),
        _ => false,
    })
}

fn group_subject(filter: Filter) -> Subject {
    let lands = is_land_filter(&filter);
    let creatures = is_creature_filter(&filter);
    let hint = if lands {
        CardType::Land
    } else if creatures {
        CardType::Creature
    } else if filter_mentions(&filter, &|x| matches!(x, Filter::Type(CardType::Artifact))) {
        CardType::Artifact
    } else if filter_mentions(&filter, &|x| {
        matches!(x, Filter::Type(CardType::Enchantment))
    }) {
        CardType::Enchantment
    } else {
        CardType::Creature
    };
    Subject {
        filter,
        it: None,
        hint,
        lands,
        creatures,
    }
}

/// Parses a subject. `referent` is the object a pronoun refers to.
fn parse_subject(s: &str, referent: Option<&Sel>, ctx: &CompileContext) -> Option<Subject> {
    let s = s.trim();
    let this_hint = || {
        if ctx.type_line.card_types.contains(CardType::Creature) {
            CardType::Creature
        } else if ctx.type_line.card_types.contains(CardType::Land) {
            CardType::Land
        } else if ctx.type_line.card_types.contains(CardType::Artifact) {
            CardType::Artifact
        } else {
            CardType::Creature
        }
    };
    if s == "~" {
        return Some(Subject {
            filter: Filter::Source,
            it: Some(Sel::This),
            hint: this_hint(),
            lands: ctx.type_line.card_types.contains(CardType::Land),
            creatures: ctx.type_line.card_types.contains(CardType::Creature),
        });
    }
    if matches!(s, "it" | "he" | "she") {
        let sel = referent?.clone();
        return match sel {
            Sel::This => parse_subject("~", None, ctx),
            Sel::AttachedTo => Some(Subject {
                filter: Filter::AttachedToSource,
                it: Some(Sel::AttachedTo),
                hint: CardType::Creature,
                lands: false,
                creatures: false,
            }),
            _ => None,
        };
    }
    for (p, hint, lands, creatures) in [
        ("enchanted creature", CardType::Creature, false, true),
        ("equipped creature", CardType::Creature, false, true),
        ("enchanted permanent", CardType::Creature, false, false),
        ("enchanted land", CardType::Land, true, false),
        ("fortified land", CardType::Land, true, false),
        ("enchanted artifact", CardType::Artifact, false, false),
        ("enchanted enchantment", CardType::Enchantment, false, false),
        (
            "enchanted planeswalker",
            CardType::Planeswalker,
            false,
            false,
        ),
    ] {
        if s == p {
            return Some(Subject {
                filter: Filter::AttachedToSource,
                it: Some(Sel::AttachedTo),
                hint,
                lands,
                creatures,
            });
        }
    }
    // "~ and other Knights you control"
    if let Some(r) = s.strip_prefix("~ and ") {
        let g = parse_group(r)?;
        let mut sub = group_subject(Filter::Or(vec![Filter::Source, g.filter]));
        sub.creatures = g.creatures && ctx.type_line.card_types.contains(CardType::Creature);
        sub.lands = false;
        return Some(sub);
    }
    parse_group(s)
}

/// Whether a filter can match anything but permanents (spells, cards in other zones):
/// static abilities here only affect permanents.
fn mentions_other_zones(f: &Filter) -> bool {
    match f {
        Filter::Spell | Filter::PermanentCard | Filter::Card => true,
        Filter::InZone(z) => *z != ZoneKind::Battlefield,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_other_zones),
        Filter::Not(x) => mentions_other_zones(x),
        _ => false,
    }
}

/// "creatures you control", "all Sliver creatures", "each creature you control with a
/// +1/+1 counter on it", "other Wolves and Werewolves you control", "Goblins you
/// control and Elementals you control".
fn parse_group(s: &str) -> Option<Subject> {
    let (s, quantified) =
        if let Some(r) = s.strip_prefix("each ").or_else(|| s.strip_prefix("all ")) {
            (r, true)
        } else {
            (s, false)
        };
    // Whole phrases joined by "and": "Goblins you control and Elementals you control".
    let parts = split_list(s);
    let f = if parts.len() >= 2 {
        let whole: Option<Vec<Filter>> = parts
            .iter()
            .map(|p| {
                whole_object_phrase(p).and_then(|(f, plural)| (plural || quantified).then_some(f))
            })
            .collect();
        whole.filter(|v| {
            // Only when each part stands alone (names its controller); "Wolves and
            // Werewolves you control" shares one suffix.
            v.iter()
                .all(|f| filter_mentions(f, &|x| matches!(x, Filter::ControlledBy(_))))
        })
    } else {
        None
    };
    let f = match f {
        Some(v) => Filter::Or(v),
        None => {
            let u = union_nouns(s);
            let (f, plural) = whole_object_phrase(&u)?;
            if !plural && !quantified {
                return None;
            }
            f
        }
    };
    // Only permanents: groups in other zones need zone-aware grants.
    if mentions_other_zones(&f) {
        return None;
    }
    Some(group_subject(f))
}

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

/// Replaces X with a value ("gets +X/+0, where X is ...").
fn subst_x(v: Value, x: &Value) -> Value {
    match v {
        Value::X => x.clone(),
        Value::Diff(a, b) => Value::Diff(Box::new(subst_x(*a, x)), Box::new(subst_x(*b, x))),
        Value::Mul(a, b) => Value::Mul(Box::new(subst_x(*a, x)), Box::new(subst_x(*b, x))),
        Value::Sum(v) => Value::Sum(v.into_iter().map(|y| subst_x(y, x)).collect()),
        other => other,
    }
}

fn mentions_x(v: &Value) -> bool {
    match v {
        Value::X => true,
        Value::Diff(a, b) | Value::Mul(a, b) => mentions_x(a) || mentions_x(b),
        Value::Sum(v) => v.iter().any(mentions_x),
        _ => false,
    }
}

/// "for each [...]": the number of things counted. `it` is the single object the
/// subject is, if any.
pub(crate) fn parse_for_each(s: &str, it: Option<&Sel>) -> Option<Value> {
    let s = end(s);
    match s {
        "card in your hand" => return Some(Value::HandSize(PlayerRef::You)),
        "card in your graveyard" => return Some(Value::GraveyardSize(PlayerRef::You)),
        "basic land type among lands you control" => return Some(Value::Domain),
        "card type among cards in your graveyard" => {
            return Some(Value::CardTypesAmong(Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ])))
        }
        _ => {}
    }
    // "+1/+1 counter on it", "charge counter on ~", "counter on it".
    for (tail, sel) in [
        (" on ~", Some(Sel::This)),
        (" on it", it.cloned()),
        (" on enchanted creature", Some(Sel::AttachedTo)),
        (" on equipped creature", Some(Sel::AttachedTo)),
    ] {
        if let Some(body) = s.strip_suffix(tail) {
            let Some(sel) = sel else { return None };
            if body == "counter" {
                return Some(Value::CountersOn(Box::new(sel), None));
            }
            let (kind, c) = body.split_once(' ')?;
            if c != "counter" {
                return None;
            }
            return Some(Value::CountersOn(Box::new(sel), Some(kind.into())));
        }
    }
    // "experience counter you have"
    if let Some(body) = s.strip_suffix(" counter you have") {
        if !body.contains(' ') {
            return Some(Value::PlayerCounters(PlayerRef::You, body.into()));
        }
        return None;
    }
    // "Aura attached to it", "Aura and Equipment attached to ~".
    for (tail, sel) in [
        (" attached to ~", Some(Sel::This)),
        (" attached to it", it.cloned()),
    ] {
        if let Some(body) = s.strip_suffix(tail) {
            let sel = sel?;
            if !matches!(sel, Sel::This) {
                return None;
            }
            let (f, _) = whole_object_phrase(&union_nouns(body))?;
            return Some(Value::Count(Filter::and(vec![
                f,
                Filter::In(Box::new(Sel::AttachedToThis)),
            ])));
        }
    }
    let (f, _) = whole_object_phrase(&union_nouns(s))?;
    Some(Value::Count(f))
}

// ---------------------------------------------------------------------------
// Predicates
// ---------------------------------------------------------------------------

const VERBS: &[&str] = &[
    "get", "gets", "have", "has", "is", "are", "isn't", "aren't", "can't", "can", "lose", "loses",
    "attack", "attacks", "block", "blocks", "doesn't", "don't", "must",
];

fn starts_with_verb(s: &str) -> bool {
    let (w, _) = split_word(s);
    VERBS.contains(&w)
}

/// Splits "gets +1/+1, has flying, and is a Demon" into predicates at commas and "and"
/// that are followed by a verb.
fn split_predicates(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let sep = [", and ", " and ", ", "]
            .into_iter()
            .find(|sep| rest.starts_with(sep) && starts_with_verb(&rest[sep.len()..]));
        if let Some(sep) = sep {
            out.push(s[start..i].trim());
            i += sep.len();
            start = i;
            continue;
        }
        i += 1;
        while i < s.len() && !s.is_char_boundary(i) {
            i += 1;
        }
    }
    out.push(s[start..].trim());
    out
}

/// One thing a predicate does.
enum Out {
    Mod(Modification),
    Restr(Restriction),
}

/// Keyword list items: "flying", "first strike", "protection from red", "ward {2}".
fn keyword_mods(item: &str) -> Option<Vec<Modification>> {
    let tl = TypeLine::default();
    let kctx = CompileContext {
        card_name: "\u{1}",
        full_name: "\u{1}",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let abilities = crate::oracle::keywords::parse_keyword_line(item, &kctx)?;
    let mut out = Vec::new();
    for a in abilities {
        match &a.kind {
            // "mobilize X, where X is ..." would need X bound per object.
            AbilityKind::Keyword(k) if k.n == Some(-1) => return None,
            AbilityKind::Keyword(k) if !functions_on_battlefield(k.kind) => return None,
            AbilityKind::Keyword(k) => out.push(Modification::AddKeyword(k.clone())),
            // Keywords that imply more (devoid's colorlessness) aren't granted this way.
            _ => return None,
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Keywords that matter only while casting a spell or in other zones: granting them to
/// permanents on the battlefield wouldn't do what the card says (e.g. "~ has flash as
/// long as you control a Desert" works from your hand).
fn functions_on_battlefield(k: KeywordKind) -> bool {
    use KeywordKind::*;
    !matches!(
        k,
        Flash
            | Buyback
            | Cycling
            | Kicker
            | Flashback
            | Madness
            | Morph
            | Storm
            | Affinity
            | Entwine
            | Splice
            | Offering
            | Ninjutsu
            | Epic
            | Convoke
            | Dredge
            | Transmute
            | Replicate
            | Forecast
            | Recover
            | Ripple
            | SplitSecond
            | Suspend
            | Delve
            | Gravestorm
            | Transfigure
            | Evoke
            | Prowl
            | Conspire
            | Retrace
            | Unearth
            | Cascade
            | Rebound
            | Miracle
            | Overload
            | Scavenge
            | Cipher
            | Fuse
            | Bestow
            | Dash
            | Awaken
            | Surge
            | Emerge
            | Escalate
            | Improvise
            | Aftermath
            | Embalm
            | Eternalize
            | Assist
            | JumpStart
            | Spectacle
            | Escape
            | Companion
            | Mutate
            | Encore
            | Foretell
            | Demonstrate
            | Disturb
            | Cleave
            | Blitz
            | Casualty
            | Prototype
            | MoreThanMeetsTheEye
            | Bargain
            | Craft
            | Disguise
            | Plot
            | Spree
            | Freerunning
            | Gift
            | Offspring
            | Impending
            | Squad
            | Partner
            | HiddenAgenda
            | Changeling
            | Devoid
            | Haunt
            | Soulshift
            | Reinforce
    )
}

/// Splits a list at ", and ", " and ", ", " (the text has no quotes inside it).
fn split_list(s: &str) -> Vec<&str> {
    let mut v: Vec<&str> = vec![s];
    for sep in [", and ", ", ", " and "] {
        v = v
            .into_iter()
            .flat_map(|p| p.split(sep))
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
    }
    v
}

/// "has flying, first strike, and "{T}: ..."" — keywords and quoted abilities.
fn grant_list(
    r: &str,
    subj: &Subject,
    quotes: &[String],
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Out>> {
    // "protection from black and from red" is one keyword.
    if r.starts_with("protection from ") && !r.contains('"') {
        return Some(keyword_mods(r)?.into_iter().map(Out::Mod).collect());
    }
    let mut out = Vec::new();
    for item in split_list(r) {
        if let Some(k) = item
            .strip_prefix("\"#")
            .and_then(|x| x.strip_suffix('"'))
            .and_then(|x| x.parse::<usize>().ok())
        {
            let q = quotes.get(k)?;
            for a in granted_abilities(q, text, subj.hint, ctx)? {
                out.push(Out::Mod(Modification::AddAbility(a)));
            }
        } else {
            out.extend(keyword_mods(item)?.into_iter().map(Out::Mod));
        }
    }
    (!out.is_empty()).then_some(out)
}

/// "N/N" base P/T.
fn base_pt(s: &str) -> Option<(Value, Value)> {
    let (p, t) = s.split_once('/')?;
    let p: i32 = p.parse().ok()?;
    let t: i32 = t.parse().ok()?;
    Some((Value::c(p), Value::c(t)))
}

/// Type words after "is"/"are": colors, card types and subtypes ("a blue Frog
/// creature", "Zombies", "an artifact creature", "Clues", "every basic land type").
struct TypeWords {
    colors: Option<ColorSet>,
    card_types: Vec<CardType>,
    subtypes: Vec<Subtype>,
    pt: Option<(Value, Value)>,
    keywords: Vec<Modification>,
    still_lands: bool,
}

fn type_words(s: &str) -> Option<TypeWords> {
    let mut s = s.trim();
    let mut tw = TypeWords {
        colors: None,
        card_types: vec![],
        subtypes: vec![],
        pt: None,
        keywords: vec![],
        still_lands: false,
    };
    // Trailing modifiers.
    for tail in [" that's still a land", " that are still lands"] {
        if let Some(r) = s.strip_suffix(tail) {
            tw.still_lands = true;
            s = r;
        }
    }
    if let Some(i) = s.find(" with base power and toughness ") {
        tw.pt = Some(base_pt(&s[i + " with base power and toughness ".len()..])?);
        s = &s[..i];
    } else if let Some(i) = s.find(" with ") {
        tw.keywords = keyword_mods(&s[i + " with ".len()..])?;
        s = &s[..i];
    }
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    if s == "every basic land type" {
        tw.subtypes = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .into_iter()
            .map(Subtype::from)
            .collect();
        return Some(tw);
    }
    let mut words: Vec<&str> = s
        .split([' ', ','])
        .filter(|w| !w.is_empty() && *w != "and")
        .collect();
    if words.is_empty() {
        return None;
    }
    if let Some(pt) = base_pt(words[0]) {
        if tw.pt.is_some() {
            return None;
        }
        tw.pt = Some(pt);
        words.remove(0);
    }
    for w in words {
        if let Some(c) = Color::from_word(w) {
            let mut cs = tw.colors.unwrap_or(ColorSet::NONE);
            cs.insert(c);
            tw.colors = Some(cs);
        } else if w == "colorless" {
            if tw.colors.is_some() {
                return None;
            }
            tw.colors = Some(ColorSet::NONE);
        } else if let Some(t) = CardType::from_word(w) {
            tw.card_types.push(t);
        } else if let Some(st) = subtype_word(w) {
            tw.subtypes.push(st);
        } else {
            return None;
        }
    }
    Some(tw)
}

/// "is"/"are" predicates: type, color and P/T changes (CR 205.1, 305.7, 613.1d-e).
fn type_predicate(r: &str, subj: &Subject) -> Option<Vec<Out>> {
    let r = r.trim();
    let m = |v: Vec<Modification>| Some(v.into_iter().map(Out::Mod).collect::<Vec<Out>>());
    // "is also a Cleric, Rogue, Warrior, and Wizard"
    if let Some(x) = r.strip_prefix("also ") {
        let tw = type_words(x)?;
        if tw.colors.is_some() || !tw.card_types.is_empty() || tw.pt.is_some() {
            return None;
        }
        if tw.subtypes.is_empty() || !tw.keywords.is_empty() {
            return None;
        }
        return m(vec![Modification::AddSubtypes(tw.subtypes)]);
    }
    if r == "every creature type" {
        // CR 205.3m / 702.73a: all creature types.
        if !subj.creatures {
            return None;
        }
        return m(vec![Modification::AllCreatureTypes]);
    }
    if r == "isn't a creature" || r == "aren't creatures" || r == "not a creature" {
        return m(vec![Modification::RemoveTypes(vec![CardType::Creature])]);
    }
    // "in addition to its other types" (CR 205.1b): types are added.
    for tail in [
        " in addition to its other types",
        " in addition to their other types",
        " in addition to its other creature types",
        " in addition to their other creature types",
        " in addition to its other land types",
        " in addition to their other land types",
    ] {
        if let Some(x) = r.strip_suffix(tail) {
            let tw = type_words(x)?;
            let mut mods = Vec::new();
            if !tw.card_types.is_empty() {
                mods.push(Modification::AddTypes(tw.card_types.clone()));
            }
            // A subtype must correspond to one of the object's card types (CR 205.3d).
            for st in &tw.subtypes {
                let kind = subtype_kind(st)?;
                let ok = match kind {
                    SubtypeKind::Creature => {
                        subj.creatures || tw.card_types.contains(&CardType::Creature)
                    }
                    SubtypeKind::Land => subj.lands || tw.card_types.contains(&CardType::Land),
                    SubtypeKind::Artifact => {
                        subj.hint == CardType::Artifact
                            || tw.card_types.contains(&CardType::Artifact)
                    }
                    SubtypeKind::Enchantment => {
                        subj.hint == CardType::Enchantment
                            || tw.card_types.contains(&CardType::Enchantment)
                    }
                    _ => false,
                };
                if !ok {
                    return None;
                }
            }
            if !tw.subtypes.is_empty() {
                mods.push(Modification::AddSubtypes(tw.subtypes.clone()));
            }
            if let Some(cs) = tw.colors {
                mods.push(Modification::SetColors(cs));
            }
            if let Some((p, t)) = tw.pt {
                mods.push(Modification::SetPT(Some(p), Some(t)));
            }
            mods.extend(tw.keywords);
            if mods.is_empty() {
                return None;
            }
            return m(mods);
        }
    }
    let tw = type_words(r)?;
    let mut mods = Vec::new();
    // Colors alone: "All creatures are black" (layer 5).
    if tw.card_types.is_empty() && tw.subtypes.is_empty() && tw.pt.is_none() {
        let cs = tw.colors?;
        if !tw.keywords.is_empty() {
            return None;
        }
        return m(vec![Modification::SetColors(cs)]);
    }
    if tw.card_types.is_empty() {
        if tw.subtypes.is_empty() {
            return None;
        }
        // Subtypes only: they replace the subtypes of their kind (CR 205.1a).
        if tw.subtypes.iter().all(|s| is_basic_land_type(s)) {
            // CR 305.7: a land's subtype set to basic land types.
            if !subj.lands || tw.pt.is_some() || tw.colors.is_some() || tw.still_lands {
                return None;
            }
            mods.push(Modification::SetBasicLandType(tw.subtypes));
        } else if tw.subtypes.iter().all(|s| is_creature_type(s)) {
            if !subj.creatures {
                return None;
            }
            mods.push(Modification::RemoveAllCreatureTypes);
            mods.push(Modification::AddSubtypes(tw.subtypes));
        } else {
            return None;
        }
    } else {
        let creature = tw.card_types.contains(&CardType::Creature);
        let artifact_creature =
            creature && tw.card_types.contains(&CardType::Artifact) && tw.card_types.len() == 2;
        if tw.still_lands || (artifact_creature && tw.subtypes.is_empty()) {
            // "1/1 creatures that are still lands", "an artifact creature": the object
            // keeps its types (CR 205.1b).
            mods.push(Modification::AddTypes(tw.card_types.clone()));
            if !tw.subtypes.is_empty() {
                return None;
            }
        } else if artifact_creature {
            // "[creature types] artifact creature": keeps other types, replaces creature
            // types (CR 205.1b).
            if !tw.subtypes.iter().all(|s| is_creature_type(s)) {
                return None;
            }
            mods.push(Modification::AddTypes(tw.card_types.clone()));
            mods.push(Modification::RemoveAllCreatureTypes);
            mods.push(Modification::AddSubtypes(tw.subtypes.clone()));
        } else {
            // The new card types replace the old ones, and the subtypes replace the
            // subtypes (CR 205.1a). Supertypes are kept.
            let ok = tw.subtypes.iter().all(|s| match subtype_kind(s) {
                Some(SubtypeKind::Creature) => creature,
                Some(SubtypeKind::Land) => tw.card_types.contains(&CardType::Land),
                Some(SubtypeKind::Artifact) => tw.card_types.contains(&CardType::Artifact),
                Some(SubtypeKind::Enchantment) => tw.card_types.contains(&CardType::Enchantment),
                _ => false,
            });
            if !ok {
                return None;
            }
            mods.push(Modification::SetTypes {
                types: tw.card_types.clone(),
                subtypes: tw.subtypes.clone(),
            });
            // CR 305.7: a land whose subtype is set to basic land types also loses the
            // abilities from its rules text ("a colorless Forest land").
            let land_types: Vec<Subtype> = tw
                .subtypes
                .iter()
                .filter(|s| subtype_kind(s) == Some(SubtypeKind::Land))
                .cloned()
                .collect();
            if tw.card_types.contains(&CardType::Land) && !land_types.is_empty() {
                if !land_types.iter().all(|s| is_basic_land_type(s)) {
                    return None;
                }
                mods.push(Modification::SetBasicLandType(land_types));
            }
        }
    }
    if let Some(cs) = tw.colors {
        mods.push(Modification::SetColors(cs));
    }
    if let Some((p, t)) = tw.pt {
        mods.push(Modification::SetPT(Some(p), Some(t)));
    }
    mods.extend(tw.keywords);
    m(mods)
}

/// Restrictions and requirements on the affected objects (CR 613.11).
fn restriction_predicate(p: &str, f: &Filter) -> Option<Restriction> {
    let f = f.clone();
    Some(match p {
        "can't block" => Restriction::CantBlock(f),
        "can't attack" => Restriction::CantAttack(f),
        "can't attack or block" => Restriction::CantAttackOrBlock(f),
        "can't be blocked" => Restriction::CantBeBlocked(f),
        "attack each combat if able" | "attacks each combat if able" => Restriction::MustAttack(f),
        "block each combat if able" | "blocks each combat if able" => Restriction::MustBlock(f),
        "doesn't untap during its controller's untap step"
        | "don't untap during their controllers' untap steps"
        | "don't untap during their controller's untap step" => Restriction::DoesntUntap(f),
        "can't be blocked except by two or more creatures" => {
            Restriction::MinBlockers { attacker: f, n: 2 }
        }
        "can't be blocked except by three or more creatures" => {
            Restriction::MinBlockers { attacker: f, n: 3 }
        }
        "can block an additional creature each combat" => Restriction::ExtraBlocks {
            blocker: f,
            n: Some(1),
        },
        "can block any number of creatures" => Restriction::ExtraBlocks {
            blocker: f,
            n: None,
        },
        "can block only creatures with flying" => Restriction::CanBlockOnly {
            blocker: f,
            attackers: Filter::HasKeyword(KeywordKind::Flying),
        },
        "can't be sacrificed" => Restriction::CantBeSacrificed(f),
        _ => {
            let r = p.strip_prefix("can't be blocked by ")?;
            let (b, _) = whole_object_phrase(r)?;
            Restriction::CantBeBlockedBy {
                attacker: f,
                blocker: b,
            }
        }
    })
}

/// Parses one predicate.
fn parse_predicate(
    p: &str,
    subj: &Subject,
    x: Option<&Value>,
    used_x: &std::cell::Cell<bool>,
    quotes: &[String],
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Out>> {
    let p = end(p);
    // P/T changes (layer 7c).
    if let Some(r) = p.strip_prefix("gets ").or_else(|| p.strip_prefix("get ")) {
        let r = r.strip_prefix("an additional ").unwrap_or(r);
        let (pv, tv, tail) = crate::oracle::effects::parse_pt_mod(r)?;
        let (mut pv, mut tv) = (pv, tv);
        let tail = tail.trim();
        if let Some(fe) = tail.strip_prefix("for each ") {
            let n = parse_for_each(fe, subj.it.as_ref())?;
            let mul = |v: Value| match v {
                Value::Const(0) => Value::Const(0),
                Value::Const(1) => n.clone(),
                other => Value::Mul(Box::new(other), Box::new(n.clone())),
            };
            pv = mul(pv);
            tv = mul(tv);
        } else if !tail.is_empty() {
            return None;
        }
        if mentions_x(&pv) || mentions_x(&tv) {
            let x = x?;
            pv = subst_x(pv, x);
            tv = subst_x(tv, x);
            used_x.set(true);
        }
        return Some(vec![Out::Mod(Modification::ModifyPT(pv, tv))]);
    }
    if let Some(r) = p.strip_prefix("has ").or_else(|| p.strip_prefix("have ")) {
        // Base P/T (layer 7b).
        if let Some(pt) = r.strip_prefix("base power and toughness ") {
            let (bp, bt) = base_pt(pt)?;
            return Some(vec![Out::Mod(Modification::SetPT(Some(bp), Some(bt)))]);
        }
        return grant_list(r, subj, quotes, text, ctx);
    }
    // Losing abilities (layer 6).
    if let Some(r) = p.strip_prefix("loses ").or_else(|| p.strip_prefix("lose ")) {
        if r == "all abilities" {
            return Some(vec![Out::Mod(Modification::RemoveAllAbilities)]);
        }
        let mut out = Vec::new();
        for item in split_list(r) {
            out.push(Out::Mod(Modification::RemoveKeyword(
                KeywordKind::from_name(item)?,
            )));
        }
        return Some(out);
    }
    if let Some(r) = p
        .strip_prefix("is ")
        .or_else(|| p.strip_prefix("are "))
        .map(str::to_string)
        .or_else(|| {
            p.strip_prefix("isn't ")
                .or_else(|| p.strip_prefix("aren't "))
                .map(|r| format!("not {r}"))
        })
    {
        let r = if r == "not a creature" {
            "isn't a creature".to_string()
        } else {
            r
        };
        return type_predicate(&r, subj);
    }
    Some(vec![Out::Restr(restriction_predicate(p, &subj.filter)?)])
}

// ---------------------------------------------------------------------------
// Lines
// ---------------------------------------------------------------------------

/// A parsed body: subject and what happens to it.
struct Body {
    subject: Subject,
    outs: Vec<Out>,
}

/// "SUBJECT PREDICATES [, where X is VALUE]".
fn parse_body(
    s: &str,
    referent: Option<&Sel>,
    quotes: &[String],
    text: &str,
    ctx: &CompileContext,
) -> Option<Body> {
    let s = end(s);
    // ", where X is [value]"
    let (s, x_text) = match s.find(", where x is ") {
        Some(i) => (&s[..i], Some(&s[i + ", where x is ".len()..])),
        None => (s, None),
    };
    // Find where the predicates start: the first verb after a subject that parses.
    let mut idx = 0;
    while let Some(off) = s[idx..].find(' ') {
        let i = idx + off;
        let (subject_text, rest) = (&s[..i], &s[i + 1..]);
        idx = i + 1;
        if !starts_with_verb(rest) {
            continue;
        }
        let Some(subject) = parse_subject(subject_text, referent, ctx) else {
            continue;
        };
        let x = match x_text {
            Some(xt) => {
                let mut b = crate::oracle::effects::Builder::new(ctx);
                if let Some(it) = &subject.it {
                    b.it = it.clone();
                }
                let (v, tail) = crate::oracle::statics::parse_value_phrase(xt, &mut b)?;
                if !end(&tail).is_empty() || !b.targets.is_empty() {
                    return None;
                }
                Some(v)
            }
            None => None,
        };
        let mut outs = Vec::new();
        let mut ok = true;
        let used_x = std::cell::Cell::new(false);
        for p in split_predicates(rest) {
            match parse_predicate(p, &subject, x.as_ref(), &used_x, quotes, text, ctx) {
                Some(v) => outs.extend(v),
                None => {
                    ok = false;
                    break;
                }
            }
        }
        // A "where X is ..." that nothing used means X appeared somewhere we don't bind.
        if x.is_some() && !used_x.get() {
            ok = false;
        }
        if ok && !outs.is_empty() {
            return Some(Body { subject, outs });
        }
    }
    None
}

/// "during your turn" / "during turns other than yours".
fn turn_condition(s: &str) -> Option<Condition> {
    match s {
        "during your turn" | "during each of your turns" => Some(Condition::YourTurn),
        // An opponent is the active player (not a teammate, in team games).
        "during turns other than yours"
        | "during each opponent's turn"
        | "during your opponents' turns"
        | "during an opponent's turn" => Some(Condition::PlayerMatches(
            PlayerRef::ActivePlayer,
            PlayerFilter::Opponent,
        )),
        _ => None,
    }
}

fn and_all(mut conds: Vec<Condition>) -> Option<Condition> {
    match conds.len() {
        0 => None,
        1 => conds.pop(),
        _ => Some(Condition::And(conds)),
    }
}

/// Parses a masked line with conditions.
fn parse_line(
    s: &str,
    mut conds: Vec<Condition>,
    referent: Option<Sel>,
    quotes: &[String],
    text: &str,
    ctx: &CompileContext,
) -> Option<(Body, Option<Condition>)> {
    let s = end(s);
    // Leading "during your turn, " / "as long as C, ".
    if let Some((head, rest)) = s.split_once(", ") {
        if let Some(c) = turn_condition(head) {
            let mut conds2 = conds.clone();
            conds2.push(c);
            if let Some(r) = parse_line(rest, conds2, referent.clone(), quotes, text, ctx) {
                return Some(r);
            }
        }
    }
    if let Some(r) = s.strip_prefix("as long as ") {
        let mut from = 0;
        while let Some(off) = r[from..].find(", ") {
            let i = from + off;
            from = i + 2;
            let (c, rest) = (&r[..i], &r[i + 2..]);
            let Some((cond, it)) = parse_static_condition(c, None, ctx) else {
                continue;
            };
            let mut conds2 = conds.clone();
            conds2.push(cond);
            if let Some(res) = parse_line(rest, conds2, it.or(referent.clone()), quotes, text, ctx)
            {
                return Some(res);
            }
        }
        return None;
    }
    // Trailing "... as long as C" / "... during your turn".
    for (i, _) in s.match_indices(" as long as ") {
        let (b, c) = (&s[..i], &s[i + " as long as ".len()..]);
        let Some(body) = parse_body(b, referent.as_ref(), quotes, text, ctx) else {
            continue;
        };
        let Some((cond, _)) = parse_static_condition(c, body.subject.it.as_ref(), ctx) else {
            continue;
        };
        let mut conds2 = conds.clone();
        conds2.push(cond);
        return Some((body, and_all(conds2)));
    }
    for tail in [
        " during your turn",
        " during turns other than yours",
        " during each opponent's turn",
    ] {
        if let Some(b) = s.strip_suffix(tail) {
            let body = parse_body(b, referent.as_ref(), quotes, text, ctx)?;
            conds.push(turn_condition(tail.trim())?);
            return Some((body, and_all(conds)));
        }
    }
    let body = parse_body(s, referent.as_ref(), quotes, text, ctx)?;
    Some((body, and_all(conds)))
}

/// Builds the abilities for a parsed line.
fn build(body: Body, cond: Option<Condition>, text: &str) -> Vec<Ability> {
    let mut mods = Vec::new();
    let mut out = Vec::new();
    let mut restrictions = Vec::new();
    for o in body.outs {
        match o {
            Out::Mod(m) => mods.push(m),
            Out::Restr(r) => restrictions.push(r),
        }
    }
    let mk = |effect: StaticEffect| {
        let mut s = StaticAbility::new(effect);
        s.condition = cond.clone();
        AbilityDef::new(AbilityKind::Static(s), text)
    };
    if !mods.is_empty() {
        out.push(mk(StaticEffect::Continuous {
            affected: body.subject.filter.clone(),
            mods,
        }));
    }
    for r in restrictions {
        out.push(mk(StaticEffect::Restriction(r)));
    }
    out
}

/// Entry point: a static ability line on a permanent.
pub(crate) fn parse_static_line(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let (masked, quotes) = mask_quotes(end(l))?;
    let (body, cond) = parse_line(&masked, vec![], None, &quotes, text, ctx)?;
    let v = build(body, cond, text);
    (!v.is_empty()).then_some(v)
}

inventory::submit! {
    StaticPattern {
        name: "statics: subject and predicates",
        priority: 50,
        parse: parse_static_line,
    }
}
