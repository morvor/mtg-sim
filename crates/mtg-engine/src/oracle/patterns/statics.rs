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
pub(crate) fn quote_names_card(normalized: &str, ctx: &CompileContext) -> bool {
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

/// Compiles a quoted ability granted to the objects matched by a static ability or by
/// a resolving effect (CR 613.1f). `~` in it refers to the object that has the ability.
pub(crate) fn granted_abilities(
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
        ("with no abilities", Filter::not(Filter::HasAbilities)),
        (
            "that are enchanted or equipped",
            Filter::Or(vec![Filter::Enchanted, Filter::Equipped]),
        ),
        (
            "that share a color with equipped creature",
            Filter::SharesColor(Box::new(Sel::AttachedTo)),
        ),
        (
            "that share a color with enchanted creature",
            Filter::SharesColor(Box::new(Sel::AttachedTo)),
        ),
        (
            "that share a creature type with equipped creature",
            Filter::SharesCreatureType(Box::new(Sel::AttachedTo)),
        ),
        (
            "that share a creature type with enchanted creature",
            Filter::SharesCreatureType(Box::new(Sel::AttachedTo)),
        ),
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
    // Relative clauses naming kinds: "that's a Fungus or Saproling", "that's a
    // Barbarian, a Warrior, or a Berserker", "that are Zombies and/or tokens".
    for p in ["that's a ", "that's an ", "that are "] {
        if let Some(r) = t.strip_prefix(p) {
            let list = r
                .replace(", a ", ", ")
                .replace(", an ", ", ")
                .replace(" or a ", " or ")
                .replace(" or an ", " or ");
            // The clause ends the phrase.
            let (f, _, rest) = parse_object_phrase(&list)?;
            if !rest.trim().is_empty() {
                return None;
            }
            // Only kinds of objects, not a controller or zone.
            if filter_mentions(&f, &|x| {
                matches!(
                    x,
                    Filter::ControlledBy(_) | Filter::OwnedBy(_) | Filter::InZone(_)
                )
            }) {
                return None;
            }
            return Some((f, ""));
        }
    }
    // "named ~": the same name as this object (CR 201.2).
    if let Some(r) = t.strip_prefix("named ~") {
        if r.is_empty() || r.starts_with([' ', ',']) {
            return Some((Filter::SameNameAs(Box::new(Sel::This)), r));
        }
    }
    // "with power greater than the number of cards in your hand", "with power less than
    // the number of Islands you control"
    for (p, cmp) in [
        ("with power greater than ", Cmp::Gt),
        ("with power less than ", Cmp::Lt),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.starts_with("the number of ") {
                // The amount runs to the end of the phrase.
                let v = parse_amount(r, None)?;
                return Some((Filter::Power(cmp, Box::new(v)), ""));
            }
        }
    }
    // "creatures blocking or blocked by ~"
    for (p, f) in [
        (
            "blocking or blocked by ~",
            Filter::Or(vec![Filter::BlockingSource, Filter::BlockedBySource]),
        ),
        ("blocking ~", Filter::BlockingSource),
        ("blocked by ~", Filter::BlockedBySource),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',']) {
                return Some((f, r));
            }
        }
    }
    // "with power or toughness 1 or less"
    if let Some(r) = t.strip_prefix("with power or toughness ") {
        let (n, r2) = parse_number(r)?;
        for (tail, cmp) in [("or less", Cmp::Le), ("or greater", Cmp::Ge)] {
            if let Some(r3) = r2.trim_start().strip_prefix(tail) {
                if r3.is_empty() || r3.starts_with([' ', ',']) {
                    return Some((
                        Filter::Or(vec![
                            Filter::Power(cmp, Box::new(n.clone())),
                            Filter::Toughness(cmp, Box::new(n)),
                        ]),
                        r3,
                    ));
                }
            }
        }
        return None;
    }
    // "with toughness greater than its power" (each object compared with itself).
    if let Some(r) = t.strip_prefix("with toughness greater than its power") {
        if r.is_empty() || r.starts_with([' ', ',']) {
            return Some((Filter::Custom("toughness_gt_power".into()), r));
        }
    }
    // Comparisons with this object's power: "with power less than ~'s power", "with
    // greater power" (than ~).
    let this_power = || Box::new(Value::PowerOf(Box::new(Sel::This)));
    for (p, cmp) in [
        ("with power less than ~'s power", Cmp::Lt),
        ("with power greater than ~'s power", Cmp::Gt),
        ("with greater power", Cmp::Gt),
        ("with lesser power", Cmp::Lt),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',']) {
                return Some((Filter::Power(cmp, this_power()), r));
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
    // "commander creatures you own", "commanders you control" (CR 903.3).
    if let Some(r) = s.strip_prefix("commander ") {
        let (f, plural, rest) = object_phrase(r)?;
        return Some((Filter::and(vec![Filter::Commander, f]), plural, rest));
    }
    if let Some(r) = s.strip_prefix("commanders") {
        if r.is_empty() || r.starts_with(' ') {
            let probe = format!("permanents{r}");
            let (f, _, rest) = object_phrase(&probe)?;
            let rest = &r[r.len() - rest.len()..];
            return Some((Filter::and(vec![Filter::Commander, f]), true, rest));
        }
    }
    let (f, plural, mut rest) = parse_object_phrase(s)?;
    let mut parts = match f {
        Filter::And(v) => v,
        other => vec![other],
    };
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
        // "without flying or reach": neither keyword.
        if let (Some(r), Some(Filter::Not(inner))) = (t.strip_prefix("or "), parts.last()) {
            if let Filter::HasKeyword(k1) = **inner {
                let probe = format!("with {r}");
                let parsed = extra_suffix(&probe).map(|(f, r2)| (f, r.len() - r2.len()));
                if let Some((Filter::HasKeyword(k2), used)) = parsed {
                    if k1 != KeywordKind::Landwalk && k2 != KeywordKind::Landwalk {
                        parts.pop();
                        parts.push(Filter::not(Filter::Or(vec![
                            Filter::HasKeyword(k1),
                            Filter::HasKeyword(k2),
                        ])));
                        rest = &r[used..];
                        continue;
                    }
                }
            }
        }
        // "with flying or reach": another keyword joins the last "with" keyword.
        if let (Some(r), Some(Filter::HasKeyword(_))) = (t.strip_prefix("or "), parts.last()) {
            let probe = format!("with {r}");
            let parsed = extra_suffix(&probe).map(|(f, r2)| (f, r.len() - r2.len()));
            if let Some((Filter::HasKeyword(k2), used)) = parsed {
                let k1 = parts.pop()?;
                parts.push(Filter::Or(vec![k1, Filter::HasKeyword(k2)]));
                rest = &r[used..];
                continue;
            }
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
pub(crate) fn whole_object_phrase(s: &str) -> Option<(Filter, bool)> {
    let (f, plural, rest) = object_phrase(s)?;
    end(rest).is_empty().then_some((f, plural))
}

/// Lists of nouns sharing suffixes: "Wolves and Werewolves you control" means "Wolves
/// or Werewolves you control" (either kind is affected).
pub(crate) fn union_nouns(s: &str) -> String {
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

pub(crate) fn filter_mentions(f: &Filter, pred: &dyn Fn(&Filter) -> bool) -> bool {
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
        ("enchanted artifact creature", CardType::Creature, false, true),
        ("enchanted equipment", CardType::Artifact, false, false),
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
    // "enchanted Mountain", "enchanted Equipment": the object the source is attached
    // to, of a kind the enchant ability already restricts it to.
    if let Some(r) = s.strip_prefix("enchanted ") {
        if let Some((f, false)) = whole_object_phrase(r) {
            if !filter_mentions(&f, &|x| {
                matches!(
                    x,
                    Filter::ControlledBy(_) | Filter::OwnedBy(_) | Filter::InZone(_)
                )
            }) {
                let g = group_subject(f);
                return Some(Subject {
                    filter: Filter::AttachedToSource,
                    it: Some(Sel::AttachedTo),
                    hint: g.hint,
                    lands: g.lands,
                    creatures: g.creatures,
                });
            }
        }
    }
    // "~ and enchanted creature" (a bestowed Aura is not a creature, CR 702.103).
    if let Some(r) = s.strip_prefix("~ and ") {
        if matches!(r, "enchanted creature" | "equipped creature") {
            let mut sub = group_subject(Filter::Or(vec![
                Filter::Source,
                Filter::AttachedToSource,
            ]));
            sub.creatures = true;
            return Some(sub);
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
pub(crate) fn mentions_other_zones(f: &Filter) -> bool {
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

/// Amounts: "the number of creatures you control", "2 plus the number of ...", "twice
/// the number of ...", "half ..., rounded up", "your life total", "the greatest mana
/// value among ...", "the number of +1/+1 counters on creatures you control".
pub(crate) fn parse_amount(s: &str, it: Option<&Sel>) -> Option<Value> {
    let s = end(s);
    if let Some((a, b)) = s.split_once(" plus ") {
        return Some(Value::Sum(vec![parse_amount(a, it)?, parse_amount(b, it)?]));
    }
    if let Some(r) = s.strip_prefix("twice ") {
        return Some(Value::Mul(
            Box::new(Value::c(2)),
            Box::new(parse_amount(r, it)?),
        ));
    }
    if let Some(r) = s.strip_prefix("half ") {
        let (body, up) = if let Some(b) = r.strip_suffix(", rounded up") {
            (b, true)
        } else if let Some(b) = r.strip_suffix(", rounded down") {
            (b, false)
        } else {
            return None;
        };
        return Some(Value::Div(Box::new(parse_amount(body, it)?), 2, up));
    }
    if let Some((n, rest)) = parse_number(s) {
        if rest.trim().is_empty() && !matches!(n, Value::X) && !s.starts_with(['a', 'A']) {
            return Some(n);
        }
    }
    // "your devotion to green", "your devotion to black and red" (CR 700.5)
    if let Some(r) = s.strip_prefix("your devotion to ") {
        let mut set = ColorSet::NONE;
        for w in r.split(" and ") {
            set.insert(Color::from_word(w.trim())?);
        }
        return Some(Value::Devotion(set));
    }
    match s {
        "your life total" => return Some(Value::LifeTotal(PlayerRef::You)),
        "the number of cards you've drawn this turn" => {
            return Some(Value::CardsDrawnThisTurn(PlayerRef::You))
        }
        "the number of creatures that died this turn" => return Some(Value::CreaturesDiedThisTurn),
        _ => {}
    }
    if let Some(r) = s
        .strip_prefix("the number of ")
        .or_else(|| s.strip_prefix("the total number of "))
    {
        return parse_for_each(r, it);
    }
    for (p, power) in [
        ("the greatest mana value among ", false),
        ("the greatest power among ", true),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            let (f, plural) = whole_object_phrase(&union_nouns(r))?;
            if !plural {
                return None;
            }
            return Some(if power {
                Value::GreatestPower(f)
            } else {
                Value::GreatestManaValue(f)
            });
        }
    }
    None
}

/// "a +1/+1 counter", "charge counters", "counter" → Some(kind or None for any kind).
fn counter_words(body: &str) -> Option<Option<CounterKind>> {
    let body = body.trim();
    if body == "counter" || body == "counters" {
        return Some(None);
    }
    let (kind, c) = body.split_once(' ')?;
    if c != "counter" && c != "counters" {
        return None;
    }
    if !(kind.starts_with('+') || kind.starts_with('-') || kind.chars().all(|c| c.is_alphabetic()))
    {
        return None;
    }
    Some(Some(kind.into()))
}

/// What's counted by "for each [...]" or "the number of [...]" (singular or plural
/// nouns). `it` is the single object the subject is, if any.
pub(crate) fn parse_for_each(s: &str, it: Option<&Sel>) -> Option<Value> {
    let s = end(s);
    // Only cards count: a token in a graveyard isn't a card (CR 108.2b).
    let your_graveyard = || {
        Filter::and(vec![
            Filter::Card,
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ])
    };
    match s {
        "card in your hand" | "cards in your hand" => return Some(Value::HandSize(PlayerRef::You)),
        "card in your graveyard" | "cards in your graveyard" => {
            return Some(Value::CardsInGraveyard(PlayerRef::You, Filter::Card))
        }
        "card in all players' hands" | "cards in all players' hands" => {
            return Some(Value::Count(Filter::InZone(ZoneKind::Hand)))
        }
        "card in all graveyards" | "cards in all graveyards" => {
            return Some(Value::Count(Filter::and(vec![
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
            ])))
        }
        "basic land type among lands you control" | "basic land types among lands you control" => {
            return Some(Value::Domain)
        }
        // CR 700.8a.
        "creature in your party" | "creatures in your party" => {
            return Some(Value::Custom(crate::game_terms::PARTY_SIZE.into()))
        }
        "card type among cards in your graveyard" | "card types among cards in your graveyard" => {
            return Some(Value::CardTypesAmong(your_graveyard()))
        }
        "card type among cards in all graveyards" | "card types among cards in all graveyards" => {
            return Some(Value::CardTypesAmong(Filter::and(vec![
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
            ])))
        }
        _ => {}
    }
    // "creature you control and each creature card in your graveyard": a sum.
    if let Some((a, b)) = s.split_once(" and each ") {
        if let (Some(va), Some(vb)) = (parse_for_each(a, it), parse_for_each(b, it)) {
            return Some(Value::Sum(vec![va, vb]));
        }
    }
    // "of its colors": how many colors it has.
    if s == "of its colors" {
        let which = match it? {
            Sel::This => "source",
            Sel::AttachedTo => "host",
            Sel::Var(v) if *v == vars::AFFECTED => "affected",
            _ => return None,
        };
        return Some(Value::Custom(format!("colors_of:{which}").into()));
    }
    // "other creature on the battlefield that shares a creature type with it": other
    // than it (not other than the source).
    if let (Some(sel), Some(r)) = (it, s.strip_prefix("other ")) {
        let body = r
            .strip_suffix(" that shares a creature type with it")
            .or_else(|| r.strip_suffix(" that shares at least one creature type with it"));
        if let Some(body) = body {
            let (f, _) = whole_object_phrase(body)?;
            return Some(Value::Count(Filter::and(vec![
                f,
                Filter::SharesCreatureType(Box::new(sel.clone())),
                Filter::not(Filter::In(Box::new(sel.clone()))),
            ])));
        }
    }
    // "+1/+1 counter on it", "charge counters on ~", "counter on it".
    for (tail, sel) in [
        (" on ~", Some(Sel::This)),
        (" on it", it.cloned()),
        (" on them", it.cloned()),
        (" on enchanted creature", Some(Sel::AttachedTo)),
        (" on equipped creature", Some(Sel::AttachedTo)),
    ] {
        if let Some(body) = s.strip_suffix(tail) {
            // Not "creature with a +1/+1 counter on it" (a filter, below).
            if let Some(kind) = counter_words(body) {
                return Some(Value::CountersOn(Box::new(sel?), kind));
            }
        }
    }
    // "card in its controller's hand", "creature card in its controller's graveyard":
    // the controller of the object "it" refers to.
    if let Some(sel) = it {
        let who = || PlayerRef::ControllerOf(Box::new(sel.clone()));
        match s {
            "card in its controller's hand" | "cards in its controller's hand" => {
                return Some(Value::HandSize(who()))
            }
            "card in its controller's graveyard" | "cards in its controller's graveyard" => {
                return Some(Value::CardsInGraveyard(who(), Filter::Card))
            }
            _ => {}
        }
        for tail in [" in its controller's graveyard"] {
            if let Some(body) = s.strip_suffix(tail) {
                let (f, _) = whole_object_phrase(&union_nouns(body))?;
                if filter_mentions(&f, &|x| {
                    matches!(x, Filter::InZone(_) | Filter::Spell | Filter::Permanent)
                }) {
                    return None;
                }
                return Some(Value::CardsInGraveyard(who(), f));
            }
        }
    }
    // "opponent whose life total is less than half their starting life total"
    if s == "opponent whose life total is less than half their starting life total" {
        return Some(Value::CountPlayers(PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::Life(
                Cmp::Lt,
                Box::new(Value::Div(Box::new(Value::StartingLife), 2, true)),
            ),
        ])));
    }
    // "poison counter your opponents have"
    for tail in [" counter your opponents have", " counters your opponents have"] {
        if let Some(kind) = s.strip_suffix(tail) {
            if kind.is_empty() || kind.contains(' ') {
                return None;
            }
            return Some(Value::Custom(format!("opponents_counters:{kind}").into()));
        }
    }
    // "creature card in your opponents' graveyards"
    for (tail, zone) in [
        (" in your opponents' graveyards", ZoneKind::Graveyard),
        (" your opponents own in exile", ZoneKind::Exile),
    ] {
        if let Some(body) = s.strip_suffix(tail) {
            let (f, _) = whole_object_phrase(&union_nouns(body))?;
            // Only kinds of cards ("creature card"), no other zone.
            if filter_mentions(&f, &|x| {
                matches!(x, Filter::InZone(_) | Filter::Spell | Filter::Permanent)
            }) {
                return None;
            }
            return Some(Value::Count(Filter::and(vec![
                f,
                Filter::InZone(zone),
                Filter::OwnedBy(PlayerRel::Opponent),
            ])));
        }
    }
    if matches!(s, "card in your opponents' hands" | "cards in your opponents' hands") {
        return Some(Value::Count(Filter::and(vec![
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::Opponent),
        ])));
    }
    // "+1/+1 counters on creatures you control" (summed over the group).
    if let Some((body, group)) = s.split_once(" on ") {
        if let Some(kind) = counter_words(body) {
            let (f, plural) = whole_object_phrase(&union_nouns(group))?;
            if !plural || mentions_other_zones(&f) {
                return None;
            }
            return Some(Value::CountersOn(Box::new(Sel::All(f)), kind));
        }
    }
    // "experience counter you have"
    for tail in [" counter you have", " counters you have"] {
        if let Some(body) = s.strip_suffix(tail) {
            if !body.contains(' ') && !body.is_empty() {
                return Some(Value::PlayerCounters(PlayerRef::You, body.into()));
            }
            return None;
        }
    }
    // "Aura attached to it", "Aura and Equipment attached to ~".
    for (tail, sel) in [
        (" attached to ~", Some(Sel::This)),
        (" attached to it", it.cloned()),
    ] {
        if let Some(body) = s.strip_suffix(tail) {
            let attached = match sel? {
                Sel::This => Filter::In(Box::new(Sel::AttachedToThis)),
                // Attached to the object the source is attached to.
                Sel::AttachedTo => Filter::Custom("attached_to_host".into()),
                // Attached to each affected object.
                Sel::Var(v) if v == vars::AFFECTED => {
                    Filter::Custom("attached_to_affected".into())
                }
                _ => return None,
            };
            let (f, _) = whole_object_phrase(&union_nouns(body))?;
            return Some(Value::Count(Filter::and(vec![f, attached])));
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
    "attack", "attacks", "block", "blocks", "doesn't", "don't", "must", "assign", "assigns",
];

fn starts_with_verb(s: &str) -> bool {
    let (w, _) = split_word(s);
    VERBS.contains(&w)
}

/// Splits "gets +1/+1, has flying, and is a Demon" into predicates at commas and "and"
/// that are followed by a verb (the subject may be repeated as "it": "... and it can't
/// be blocked").
fn split_predicates(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let sep = [
            ", and it ",
            " and it ",
            ", and they ",
            " and they ",
            ", and ",
            " and ",
            ", ",
        ]
        .into_iter()
        .find(|sep| {
            rest.starts_with(sep)
                && (starts_with_verb(&rest[sep.len()..])
                    || rest[sep.len()..].starts_with("its activated abilities ")
                    || rest[sep.len()..].starts_with("their activated abilities "))
        });
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
    /// A static effect that isn't about the subject objects (players' abilities).
    Other(StaticEffect),
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
    // "..., and protection from black and from red": one keyword.
    let mut items: Vec<String> = Vec::new();
    for item in split_list(r) {
        match items.last_mut() {
            Some(last) if item.starts_with("from ") && last.starts_with("protection from ") => {
                last.push_str(" and ");
                last.push_str(item);
            }
            _ => items.push(item.to_string()),
        }
    }
    for item in items.iter().map(String::as_str) {
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
    /// Supertypes are added (CR 205.4; setting card types keeps them, CR 205.1a).
    supertypes: Vec<Supertype>,
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
        supertypes: vec![],
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
        for item in split_list(&s[i + " with ".len()..]) {
            tw.keywords.extend(keyword_mods(item)?);
        }
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
        } else if let Some(st) = Supertype::from_word(w) {
            tw.supertypes.push(st);
        } else if let Some(st) = subtype_word(w) {
            tw.subtypes.push(st);
        } else {
            return None;
        }
    }
    Some(tw)
}

/// The layer modifications of the "is"/"are" type predicate `r` ("a 2/2 Elemental
/// creature that's still a land"), for one-shot effects that make objects become that
/// ("target land becomes a 2/2 Elemental creature"): None unless every part is a
/// modification of the subject objects.
pub(crate) fn type_predicate_mods(r: &str, subj: &Subject) -> Option<Vec<Modification>> {
    type_predicate(r, subj)?
        .into_iter()
        .map(|o| match o {
            Out::Mod(m) => Some(m),
            _ => None,
        })
        .collect()
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
    if r == "every creature type" || r == "all creature types" {
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
    // "is a black Zombie in addition to its other colors and types": colors are added
    // too (CR 105.3, 205.1b).
    for tail in [
        " in addition to its other colors and types",
        " in addition to their other colors and types",
    ] {
        if let Some(x) = r.strip_suffix(tail) {
            let tw = type_words(x)?;
            let cs = tw.colors?;
            let mut out = type_predicate(&format!("{x} in addition to its other types"), subj)?;
            for o in &mut out {
                if let Out::Mod(Modification::SetColors(c)) = o {
                    if *c == cs {
                        *o = Out::Mod(Modification::AddColors(cs));
                    }
                }
            }
            return Some(out);
        }
    }
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
            if !tw.supertypes.is_empty() {
                mods.push(Modification::AddSupertypes(tw.supertypes.clone()));
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
    if !tw.supertypes.is_empty() {
        // "is legendary", "is snow" (layer 4).
        if tw.supertypes.contains(&Supertype::Basic) {
            return None;
        }
        mods.push(Modification::AddSupertypes(tw.supertypes.clone()));
    }
    // Colors alone: "All creatures are black" (layer 5).
    if tw.card_types.is_empty() && tw.subtypes.is_empty() && tw.pt.is_none() {
        if !tw.keywords.is_empty() {
            return None;
        }
        if let Some(cs) = tw.colors {
            mods.push(Modification::SetColors(cs));
        }
        if mods.is_empty() {
            return None;
        }
        return m(mods);
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
            // keeps its types and subtypes (CR 205.1b).
            mods.push(Modification::AddTypes(tw.card_types.clone()));
            if !tw.subtypes.is_empty() {
                // "a 2/2 blue Elemental creature that's still a land"
                if !creature || !tw.subtypes.iter().all(|s| is_creature_type(s)) {
                    return None;
                }
                mods.push(Modification::AddSubtypes(tw.subtypes.clone()));
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
pub(crate) fn restriction_predicate(p: &str, f: &Filter) -> Option<Vec<Restriction>> {
    let fc = f.clone();
    match p {
        // CR 701.15b; a static "is goaded" goads for the source's controller.
        "is goaded" | "are goaded" => return Some(vec![Restriction::Goaded(fc)]),
        "can't attack you" | "can't attack you or planeswalkers you control" => {
            return Some(vec![Restriction::CantAttackPlayer {
                attackers: fc,
                defender: PlayerFilter::You,
                planeswalkers: p.ends_with("planeswalkers you control"),
                battles: false,
            }])
        }
        // CR 115.4 ("can't be the target of"): hexproof-like, shroud-like, or by the
        // qualities of the spell or the ability's source.
        "can't be the target of spells or abilities your opponents control"
        | "can't be the targets of spells or abilities your opponents control" => {
            return Some(vec![Restriction::CantBeTargeted {
                what: fc,
                by: TargetRestriction::Opponents,
            }])
        }
        "can't be the target of spells or abilities"
        | "can't be the targets of spells or abilities" => {
            return Some(vec![Restriction::CantBeTargeted {
                what: fc,
                by: TargetRestriction::Any,
            }])
        }
        "assigns combat damage equal to its toughness rather than its power"
        | "assign combat damage equal to their toughness rather than their power" => {
            return Some(vec![Restriction::DamageByToughness(fc)])
        }
        "attack or block each combat if able" | "attacks or blocks each combat if able" => {
            return Some(vec![
                Restriction::MustAttack(fc.clone()),
                Restriction::MustBlock(fc),
            ])
        }
        // CR 602.5a: activated abilities (mana abilities included) can't be activated.
        "its activated abilities can't be activated"
        | "their activated abilities can't be activated" => {
            return Some(vec![Restriction::CantActivate {
                who: PlayerFilter::Any,
                sources: fc,
                include_mana: true,
            }])
        }
        "its activated abilities can't be activated unless they're mana abilities"
        | "their activated abilities can't be activated unless they're mana abilities" => {
            return Some(vec![Restriction::CantActivate {
                who: PlayerFilter::Any,
                sources: fc,
                include_mana: false,
            }])
        }
        _ => {}
    }
    if let Some(x) = p
        .strip_prefix("can't be the target of ")
        .or_else(|| p.strip_prefix("can't be the targets of "))
    {
        return Some(vec![Restriction::CantBeTargeted {
            what: fc,
            by: TargetRestriction::Sources(targeting_sources(x)?),
        }]);
    }
    // "can't block it", "can't block creatures with power 2 or greater", "Cowards can't
    // block Warriors": the subject can't block the named attackers.
    if let Some(x) = p.strip_prefix("can't block ") {
        let attacker = match x {
            "it" | "~" => Filter::Source,
            _ => {
                let (a, plural) = whole_object_phrase(&union_nouns(x))?;
                if !plural {
                    return None;
                }
                a
            }
        };
        return Some(vec![Restriction::CantBeBlockedBy {
            attacker,
            blocker: fc,
        }]);
    }
    // "can't be blocked except by creatures with flying"
    if let Some(x) = p.strip_prefix("can't be blocked except by ") {
        if x.contains(" or more ") {
            return single_restriction(p, f).map(|r| vec![r]);
        }
        if let Some(b) = and_or_phrase(x) {
            return Some(vec![Restriction::CantBeBlockedBy {
                attacker: fc,
                blocker: Filter::not(b),
            }]);
        }
        // "Walls and/or creatures with flying": the suffix belongs to the last noun only,
        // unlike "Walls and creatures you control"; leave such lists alone.
        if let Some((nouns, _)) = x.split_once(" with ") {
            if nouns.contains(" or ") || nouns.contains(" and") || nouns.contains(',') {
                return None;
            }
        }
        let (b, plural) = whole_object_phrase(&union_nouns(x))?;
        if !plural {
            return None;
        }
        return Some(vec![Restriction::CantBeBlockedBy {
            attacker: fc,
            blocker: Filter::not(b),
        }]);
    }
    if let Some(r) = single_restriction(p, f) {
        return Some(vec![r]);
    }
    // "can't attack you or block creatures you control": two restrictions.
    let x = p.strip_prefix("can't ")?;
    for (i, _) in x.match_indices(" or ") {
        let (a, b) = (&x[..i], &x[i + " or ".len()..]);
        // A bare verb shares the second part's object ("can't block or be blocked by
        // creatures with power 2 or greater").
        if !a.contains(' ') {
            continue;
        }
        if let (Some(mut ra), Some(rb)) = (
            restriction_predicate(&format!("can't {a}"), f),
            restriction_predicate(&format!("can't {b}"), f),
        ) {
            ra.extend(rb);
            return Some(ra);
        }
    }
    None
}

/// Which spells and abilities can't target: "spells", "Aura spells", "white spells or
/// abilities from white sources", "blue or black spells", "abilities from artifact
/// sources". An ability's qualities are those of its source (CR 113.7).
fn targeting_sources(x: &str) -> Option<Filter> {
    let quality = |w: &str| -> Option<Filter> {
        if let Some(c) = w.strip_prefix("non").and_then(Color::from_word) {
            return Some(Filter::not(Filter::Color(c)));
        }
        if let Some((a, b)) = w.split_once(" or ") {
            return Some(Filter::Or(vec![
                Filter::Color(Color::from_word(a)?),
                Filter::Color(Color::from_word(b)?),
            ]));
        }
        if let Some(c) = Color::from_word(w) {
            return Some(Filter::Color(c));
        }
        let (f, _) = whole_object_phrase(w)?;
        if mentions_other_zones(&f) || filter_mentions(&f, &|x| matches!(x, Filter::ControlledBy(_))) {
            return None;
        }
        Some(f)
    };
    if x == "spells" {
        return Some(Filter::Spell);
    }
    // "white spells or abilities from white sources"
    if let Some((a, b)) = x.split_once(" spells or abilities from ") {
        let src = b.strip_suffix(" sources")?;
        if a != src {
            return None;
        }
        return quality(a);
    }
    if let Some(src) = x
        .strip_prefix("abilities from ")
        .and_then(|r| r.strip_suffix(" sources"))
    {
        return Some(Filter::and(vec![quality(src)?, Filter::not(Filter::Spell)]));
    }
    let q = x.strip_suffix(" spells")?;
    Some(Filter::and(vec![quality(q)?, Filter::Spell]))
}

/// Plural object phrases joined by "and/or": "artifact creatures and/or white
/// creatures", "Walls and/or creatures with flying", "black and/or red creatures".
fn and_or_phrase(x: &str) -> Option<Filter> {
    let parts: Vec<&str> = x.split(" and/or ").collect();
    if parts.len() < 2 {
        return None;
    }
    let whole: Option<Vec<Filter>> = parts
        .iter()
        .map(|p| whole_object_phrase(p).filter(|(_, pl)| *pl).map(|(f, _)| f))
        .collect();
    if let Some(v) = whole {
        return Some(Filter::Or(v));
    }
    // Colors before one noun.
    if parts.len() == 2 {
        let c1 = Color::from_word(parts[0])?;
        let (w, rest) = parts[1].split_once(' ')?;
        let c2 = Color::from_word(w)?;
        let (f, plural) = whole_object_phrase(rest)?;
        if plural {
            return Some(Filter::and(vec![
                Filter::Or(vec![Filter::Color(c1), Filter::Color(c2)]),
                f,
            ]));
        }
    }
    None
}

fn single_restriction(p: &str, f: &Filter) -> Option<Restriction> {
    let f = f.clone();
    Some(match p {
        "can't block" => Restriction::CantBlock(f),
        "can't attack" => Restriction::CantAttack(f),
        "can't attack or block" => Restriction::CantAttackOrBlock(f),
        "can't be blocked" => Restriction::CantBeBlocked(f),
        "attack each combat if able" | "attacks each combat if able" => Restriction::MustAttack(f),
        "block each combat if able" | "blocks each combat if able" => Restriction::MustBlock(f),
        "must be blocked if able" => Restriction::MustBeBlocked(f),
        "doesn't untap during its controller's untap step"
        | "doesn't untap during your untap step"
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
        "can't be blocked by more than one creature" => {
            Restriction::MaxBlockedBy { attacker: f, n: 1 }
        }
        // Overrides CR 702.3b.
        "can attack as though it didn't have defender"
        | "can attack as though they didn't have defender" => Restriction::AttackDespiteDefender(f),
        _ => {
            let r = p.strip_prefix("can't be blocked by ")?;
            if let Some(b) = and_or_phrase(r) {
                return Some(Restriction::CantBeBlockedBy {
                    attacker: f,
                    blocker: b,
                });
            }
            // "with greater power" compares with ~ itself; other subjects would need a
            // comparison with each attacker.
            if (r.contains("with greater power") || r.contains("with lesser power"))
                && !matches!(f, Filter::Source)
            {
                return None;
            }
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
    // "Creatures you control also get +1/+0 and have trample as long as ...": "also"
    // only says it's in addition to other effects.
    let p = p.strip_prefix("also ").unwrap_or(p);
    // P/T changes (layer 7c).
    if let Some(r) = p.strip_prefix("gets ").or_else(|| p.strip_prefix("get ")) {
        let r = r.strip_prefix("an additional ").unwrap_or(r);
        let (pv, tv, tail) = crate::oracle::effects::parse_pt_mod(r)?;
        let (mut pv, mut tv) = (pv, tv);
        let tail = tail.trim();
        if let Some(fe) = tail.strip_prefix("for each ") {
            // In a group, "it" is each affected object.
            let each = Sel::Var(vars::AFFECTED);
            let n = parse_for_each(fe, Some(subj.it.as_ref().unwrap_or(&each)))?;
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
            if let Some(a) = pt.strip_prefix("each equal to ") {
                let v = parse_amount(a, subj.it.as_ref())?;
                return Some(vec![Out::Mod(Modification::SetPT(Some(v.clone()), Some(v)))]);
            }
            let (bp, bt) = base_pt(pt)?;
            return Some(vec![Out::Mod(Modification::SetPT(Some(bp), Some(bt)))]);
        }
        for (p, power) in [("base power ", true), ("base toughness ", false)] {
            if let Some(n) = r.strip_prefix(p) {
                let n: i32 = n.parse().ok()?;
                let v = Some(Value::c(n));
                return Some(vec![Out::Mod(if power {
                    Modification::SetPT(v, None)
                } else {
                    Modification::SetPT(None, v)
                })]);
            }
        }
        return grant_list(r, subj, quotes, text, ctx);
    }
    // "can't have or gain flying": applied after other layer-6 effects.
    if let Some(r) = p.strip_prefix("can't have or gain ") {
        return Some(vec![Out::Mod(Modification::CantHaveKeyword(
            KeywordKind::from_name(r)?,
        ))]);
    }
    // Losing abilities (layer 6).
    if let Some(r) = p.strip_prefix("loses ").or_else(|| p.strip_prefix("lose ")) {
        // "loses all other abilities": all but the ones this effect grants (the removal
        // is ordered first in the effect, see [`build`]).
        if r == "all abilities" || r == "all other abilities" {
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
    if matches!(p, "is goaded" | "are goaded") {
        return Some(
            restriction_predicate(p, &subj.filter)?
                .into_iter()
                .map(Out::Restr)
                .collect(),
        );
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
    Some(
        restriction_predicate(p, &subj.filter)?
            .into_iter()
            .map(Out::Restr)
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// Lines
// ---------------------------------------------------------------------------

/// A parsed body: subject and what happens to it.
struct Body {
    subject: Subject,
    outs: Vec<Out>,
    /// Further bodies with their own subjects sharing the line's condition ("~ gets
    /// +2/+2 and creatures you control have vigilance").
    also: Vec<Body>,
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
    // "it's a creature" is "it is a creature".
    let owned;
    let s = match s.strip_prefix("it's ") {
        Some(r) => {
            owned = format!("it is {r}");
            owned.as_str()
        }
        None => s,
    };
    // "Enchanted creature's activated abilities can't be activated": the possessive
    // subject is the pronoun of the "its activated abilities" predicate.
    if let Some((subj_text, pred)) = s.split_once("'s activated abilities ") {
        let subject = parse_subject(subj_text, referent, ctx)?;
        if subject.it.is_none() {
            return None;
        }
        let p = format!("its activated abilities {pred}");
        let outs = restriction_predicate(&p, &subject.filter)?
            .into_iter()
            .map(Out::Restr)
            .collect();
        return Some(Body {
            subject,
            outs,
            also: vec![],
        });
    }
    // "You control enchanted creature" (layer 2, CR 613.1b).
    if let Some(r) = s.strip_prefix("you control ") {
        let subject = parse_subject(r, referent, ctx)?;
        if !matches!(subject.filter, Filter::AttachedToSource) {
            return None;
        }
        return Some(Body {
            subject,
            outs: vec![Out::Mod(Modification::SetController(PlayerRef::You))],
            also: vec![],
        });
    }
    if let Some(body) = parse_player_body(s) {
        return Some(body);
    }
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
        // "~ and enchanted creature each get +1/+1"
        let rest = match rest.strip_prefix("each ") {
            Some(r) if starts_with_verb(r) && subject_text.contains(" and ") => r,
            _ => rest,
        };
        // "Creatures you control also get +1/+0 ..."
        let rest = match rest.strip_prefix("also ") {
            Some(r) if starts_with_verb(r) => r,
            _ => rest,
        };
        if !starts_with_verb(rest) {
            continue;
        }
        let Some(subject) = parse_subject(subject_text, referent, ctx) else {
            continue;
        };
        let x = match x_text {
            Some(xt) if parse_amount(xt, subject.it.as_ref()).is_some() => {
                parse_amount(xt, subject.it.as_ref())
            }
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
            return Some(Body {
            subject,
            outs,
            also: vec![],
        });
        }
    }
    None
}

/// "during your turn" / "during turns other than yours".
fn turn_condition(s: &str) -> Option<Condition> {
    match s {
        "during combat" => Some(Condition::Phase(PhaseCond::Combat)),
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
            let Some((cond, it)) = parse_static_condition(c, referent.as_ref(), ctx) else {
                continue;
            };
            let mut conds2 = conds.clone();
            conds2.push(cond);
            // "As long as you control exactly one creature, that creature gets +2/+2":
            // while the condition holds, that's every creature you control.
            let owned;
            let rest = match (
                c.strip_prefix("you control exactly one "),
                rest.strip_prefix("that creature "),
            ) {
                (Some(noun), Some(tail)) if !noun.contains(' ') => {
                    owned = format!("each {noun} you control {tail}");
                    owned.as_str()
                }
                _ => rest,
            };
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
        let Some(mut body) = parse_body(b, referent.as_ref(), quotes, text, ctx) else {
            continue;
        };
        // "Each land gets +2/+2 as long as it's a creature": "it" is each affected
        // object, so the state narrows the group.
        if body.subject.it.is_none() && body.also.is_empty() {
            if let Some(f) = super::statics_conditions::pronoun_state(c) {
                body.subject.filter = Filter::and(vec![body.subject.filter.clone(), f]);
                return Some((body, and_all(conds)));
            }
        }
        let Some((cond, _)) = parse_static_condition(c, body.subject.it.as_ref(), ctx) else {
            continue;
        };
        let mut conds2 = conds.clone();
        conds2.push(cond);
        return Some((body, and_all(conds2)));
    }
    // Restrictions with a condition: "~ can't attack unless defending player controls an
    // Island", "~ doesn't untap during your untap step if it has a depletion counter on
    // it". The restriction applies while the condition holds (CR 508.1c).
    for (sep, negate) in [(" unless ", true), (" if ", false)] {
        for (i, _) in s.match_indices(sep) {
            let (b, c) = (&s[..i], &s[i + sep.len()..]);
            let Some(mut body) = parse_body(b, referent.as_ref(), quotes, text, ctx) else {
                continue;
            };
            // "~ can't attack unless defending player controls an Island": about the
            // player it would attack (CR 508.5), whether directly or through a
            // planeswalker or battle.
            if let Some(pf) = super::statics_conditions::defending_player_condition(c) {
                let [Out::Restr(Restriction::CantAttack(f))] = body.outs.as_slice() else {
                    return None;
                };
                let defender = if negate {
                    PlayerFilter::Not(Box::new(pf))
                } else {
                    pf
                };
                body.outs = vec![Out::Restr(Restriction::CantAttackPlayer {
                    attackers: f.clone(),
                    defender,
                    planeswalkers: true,
                    battles: true,
                })];
                return Some((body, and_all(conds)));
            }
            // "~ can't block unless you control more creatures than attacking player": about
            // the controller of each attacking creature it would block (CR 805.10c).
            if let Some(pf) = super::statics_conditions::attacking_player_condition(c) {
                let [Out::Restr(Restriction::CantBlock(f))] = body.outs.as_slice() else {
                    return None;
                };
                let pf = if negate {
                    pf
                } else {
                    PlayerFilter::Not(Box::new(pf))
                };
                body.outs = vec![Out::Restr(Restriction::CanBlockOnly {
                    blocker: f.clone(),
                    attackers: Filter::ControllerMatches(Box::new(pf)),
                })];
                return Some((body, and_all(conds)));
            }
            if !body.outs.iter().all(|o| matches!(o, Out::Restr(_))) {
                continue;
            }
            let Some((cond, _)) = parse_static_condition(c, body.subject.it.as_ref(), ctx) else {
                continue;
            };
            let mut conds2 = conds.clone();
            conds2.push(if negate {
                Condition::Not(Box::new(cond))
            } else {
                cond
            });
            return Some((body, and_all(conds2)));
        }
    }
    for tail in [
        " during your turn",
        " during turns other than yours",
        " during each opponent's turn",
        " during combat",
    ] {
        if let Some(b) = s.strip_suffix(tail) {
            let body = parse_body(b, referent.as_ref(), quotes, text, ctx)?;
            conds.push(turn_condition(tail.trim())?);
            return Some((body, and_all(conds)));
        }
    }
    let body = parse_body(s, referent.as_ref(), quotes, text, ctx)
        .or_else(|| compound_body(s, referent.as_ref(), quotes, text, ctx))?;
    Some((body, and_all(conds)))
}

/// "~ gets +2/+2 and other creatures you control get +2/+2 and have trample": bodies
/// with different subjects joined by "and".
fn compound_body(
    s: &str,
    referent: Option<&Sel>,
    quotes: &[String],
    text: &str,
    ctx: &CompileContext,
) -> Option<Body> {
    for sep in [", and ", " and "] {
        for (i, _) in s.match_indices(sep) {
            let (l, r) = (&s[..i], &s[i + sep.len()..]);
            // The second part names its own subject.
            if starts_with_verb(r) || r.starts_with("it ") || r.starts_with("they ") {
                continue;
            }
            let Some(mut left) = parse_body(l, referent, quotes, text, ctx) else {
                continue;
            };
            let Some(right) = parse_body(r, referent, quotes, text, ctx)
                .or_else(|| compound_body(r, referent, quotes, text, ctx))
            else {
                continue;
            };
            left.also.push(right);
            return Some(left);
        }
    }
    None
}

/// Builds the abilities for a parsed line.
fn build(body: Body, cond: Option<Condition>, zone: FunctionZone, text: &str) -> Vec<Ability> {
    let mut mods = Vec::new();
    let mut out = Vec::new();
    let also = body.also;
    let mut restrictions = Vec::new();
    for o in body.outs {
        match o {
            Out::Mod(m) => mods.push(m),
            Out::Restr(r) => restrictions.push(StaticEffect::Restriction(r)),
            Out::Other(e) => restrictions.push(e),
        }
    }
    // Abilities an effect grants survive its own "loses all (other) abilities": the
    // removal applies first within the effect.
    mods.sort_by_key(|m| !matches!(m, Modification::RemoveAllAbilities));
    let mk = |effect: StaticEffect| {
        let mut s = StaticAbility::new(effect);
        s.condition = cond.clone();
        s.zone = zone;
        AbilityDef::new(AbilityKind::Static(s), text)
    };
    if !mods.is_empty() {
        out.push(mk(StaticEffect::Continuous {
            affected: body.subject.filter.clone(),
            mods,
        }));
    }
    for e in restrictions {
        out.push(mk(e));
    }
    for b in also {
        out.extend(build(b, cond.clone(), zone, text));
    }
    out
}

/// Entry point: a static ability line on a permanent.
pub(crate) fn parse_static_line(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let (mut masked, quotes) = mask_quotes(end(l))?;
    // "As long as ~ is in your graveyard [and C], ...": the ability functions only in
    // the graveyard (CR 113.6).
    let mut zone = FunctionZone::Battlefield;
    if let Some(r) = masked.strip_prefix("as long as ~ is in your graveyard") {
        let rest = if let Some(x) = r.strip_prefix(" and ") {
            format!("as long as {x}")
        } else {
            r.strip_prefix(", ")?.to_string()
        };
        zone = FunctionZone::Graveyard;
        masked = rest;
    }
    // "... creature. It's still a land." (CR 205.1b)
    for (tail, repl) in [
        (". it's still a land", " that's still a land"),
        (". they're still lands", " that are still lands"),
    ] {
        if let Some(b) = masked.strip_suffix(tail) {
            masked = format!("{b}{repl}");
        }
    }
    let mut sentences = masked.split(". ");
    let (mut body, cond) = parse_line(sentences.next()?, vec![], None, &quotes, text, ctx)?;
    let same_subject = |a: &Body, b: &Body| format!("{:?}", a.subject.filter) == format!("{:?}", b.subject.filter);
    let mut otherwise = Vec::new();
    let mut first_unless: Option<Condition> = None;
    for sentence in sentences {
        let it = body.subject.it.clone()?;
        // "... as long as it's a Human. Otherwise, it can't attack or block." (CR 611.3a)
        if let Some(r) = sentence.strip_prefix("otherwise, ") {
            let c = cond.clone()?;
            let (b2, c2) = parse_line(r, vec![], Some(it), &quotes, text, ctx)?;
            if c2.is_some() || !same_subject(&body, &b2) {
                return None;
            }
            otherwise.extend(build(b2, Some(Condition::Not(Box::new(c))), zone, text));
            continue;
        }
        // "Enchanted creature gets -2/-2. It gets -5/-5 instead as long as you've
        // completed a dungeon.": the first effect applies only while C is false.
        if let Some(i) = sentence.find(" instead as long as ") {
            if cond.is_some() || first_unless.is_some() {
                return None;
            }
            let rewritten = format!("{}{}", &sentence[..i], &sentence[i + " instead".len()..]);
            let (b2, c2) = parse_line(&rewritten, vec![], Some(it), &quotes, text, ctx)?;
            let c2 = c2?;
            if !same_subject(&body, &b2) {
                return None;
            }
            first_unless = Some(c2.clone());
            otherwise.extend(build(b2, Some(c2), zone, text));
            continue;
        }
        // More about the same object: "Enchanted creature is a Turtle with base power and
        // toughness 0/1. It can't attack and loses all abilities." One effect, or one
        // with its own condition ("As long as it's legendary, it gets an additional
        // +2/+2.").
        let (b2, c2) = parse_line(sentence, vec![], Some(it), &quotes, text, ctx)?;
        if !same_subject(&body, &b2) {
            return None;
        }
        match c2 {
            None => {
                if cond.is_some() || !otherwise.is_empty() || first_unless.is_some() {
                    return None;
                }
                body.outs.extend(b2.outs);
            }
            Some(c2) => {
                let c = match &cond {
                    Some(c) => Condition::And(vec![c.clone(), c2]),
                    None => c2,
                };
                otherwise.extend(build(b2, Some(c), zone, text));
            }
        }
    }
    if zone != FunctionZone::Battlefield && body.subject.it.is_some() {
        // Only abilities about other objects work from the graveyard.
        return None;
    }
    let cond = match first_unless {
        Some(c) => Some(Condition::Not(Box::new(c))),
        None => cond,
    };
    let mut v = build(body, cond, zone, text);
    v.extend(otherwise);
    (!v.is_empty()).then_some(v)
}

/// "~ has flash as long as you control a Desert": the ability functions in every zone,
/// so the card can be cast as though it had flash while the condition holds (CR 601.3d).
fn conditional_flash(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("~ has flash as long as ")?;
    let (cond, _) = parse_static_condition(r, Some(&Sel::This), ctx)?;
    let mut st = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::AddKeyword(crate::keywords::Keyword::new(
            KeywordKind::Flash,
        ))],
    });
    st.condition = Some(cond);
    st.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

/// [`conditional_flash`] for any card, instants and sorceries included.
fn conditional_flash_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    conditional_flash(end(&lower), t, ctx)
}

inventory::submit! {
    crate::oracle::patterns::AbilityPattern {
        name: "statics: has flash as long as",
        priority: 50,
        parse: conditional_flash_block,
    }
}

inventory::submit! {
    StaticPattern {
        name: "statics: subject and predicates",
        priority: 50,
        parse: parse_static_line,
    }
}

/// "~ can attack this turn as though it didn't have defender" (and the same after "and"
/// with the subject left out), "creatures you control with defender can attack this
/// turn as though they didn't have defender": a rule-modifying effect until end of turn
/// that overrides CR 702.3b.
fn p_attack_despite_defender(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = end(l);
    let subject = [
        " can attack this turn as though it didn't have defender",
        " can attack this turn as though they didn't have defender",
    ]
    .iter()
    .find_map(|t| l.strip_suffix(t))
    .or_else(|| (l == "can attack this turn as though it didn't have defender").then_some(""))?;
    let filter = match subject {
        // Only objects the rule effect can still find once the ability has resolved.
        "~" => Filter::Source,
        "" | "it" if matches!(b.it, Sel::This) => Filter::Source,
        _ => {
            let (f, plural) = whole_object_phrase(&union_nouns(subject))?;
            if !plural || mentions_other_zones(&f) {
                return None;
            }
            f
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::AttackDespiteDefender(filter),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! {
    crate::oracle::patterns::EffectPattern {
        name: "statics: can attack this turn as though it didn't have defender",
        priority: 50,
        parse: p_attack_despite_defender,
    }
}

// ---------------------------------------------------------------------------
// Players as subjects
// ---------------------------------------------------------------------------

/// Removes "spell" from a filter: "can't cast creature spells" checks the card being
/// cast, which isn't on the stack yet.
pub(crate) fn without_spell(f: Filter) -> Option<Filter> {
    match f {
        Filter::Spell => Some(Filter::Any),
        Filter::And(v) => Some(Filter::and(
            v.into_iter().map(without_spell).collect::<Option<Vec<_>>>()?,
        )),
        Filter::Or(v) => Some(Filter::Or(
            v.into_iter().map(without_spell).collect::<Option<Vec<_>>>()?,
        )),
        Filter::InZone(_) | Filter::Permanent => None,
        other => Some(other),
    }
}

/// Zones a spell is cast from: "graveyards or libraries", "graveyards", "exile",
/// "anywhere other than their hands". A card being cast is still in that zone while
/// prohibitions are first checked, and on the stack remembers it (CR 601.3).
fn cast_from_zones(z: &str) -> Option<Filter> {
    let zone = |k: ZoneKind| Filter::Or(vec![Filter::InZone(k), Filter::CastFrom(k)]);
    if matches!(
        z,
        "anywhere other than their hands" | "anywhere other than their hand"
    ) {
        return Some(Filter::not(zone(ZoneKind::Hand)));
    }
    let mut v = Vec::new();
    for w in z.split(" or ") {
        v.push(zone(match w {
            "graveyards" | "a graveyard" | "their graveyards" => ZoneKind::Graveyard,
            "libraries" | "their libraries" => ZoneKind::Library,
            "exile" => ZoneKind::Exile,
            _ => return None,
        }));
    }
    Some(if v.len() == 1 { v.pop()? } else { Filter::Or(v) })
}

/// "your opponents can't cast spells", "players have no maximum hand size", "you have
/// hexproof", "each opponent can't gain life", "your opponents can't cast spells or
/// activate abilities of artifacts, creatures, or enchantments" (CR 613.10, 613.11).
fn parse_player_body(s: &str) -> Option<Body> {
    // Maximum hand size (CR 402.2): "your maximum hand size is increased by one", "each
    // opponent's maximum hand size is reduced by two", "your maximum hand size is five".
    for (p, who) in [
        ("your maximum hand size is ", PlayerFilter::You),
        ("each opponent's maximum hand size is ", PlayerFilter::Opponent),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            let m = if let Some(x) = r.strip_prefix("increased by ") {
                let (n, t) = parse_number(x)?;
                let Value::Const(n) = n else { return None };
                if !t.trim().is_empty() {
                    return None;
                }
                PlayerModification::HandSizeDelta(n)
            } else if let Some(x) = r.strip_prefix("reduced by ") {
                let (n, t) = parse_number(x)?;
                let Value::Const(n) = n else { return None };
                if !t.trim().is_empty() {
                    return None;
                }
                PlayerModification::HandSizeDelta(-n)
            } else {
                let (n, t) = parse_number(r)?;
                if !t.trim().is_empty() || matches!(n, Value::X) || r.starts_with('a') {
                    return None;
                }
                PlayerModification::MaxHandSize(Some(n))
            };
            return Some(Body {
                subject: Subject {
                    filter: Filter::Any,
                    it: None,
                    hint: CardType::Creature,
                    lands: false,
                    creatures: false,
                },
                outs: vec![Out::Other(StaticEffect::PlayerEffect {
                    affected: who,
                    effect: m,
                })],
                also: vec![],
            });
        }
    }
    let (who, rest) = [
        (
            "enchanted creature's controller ",
            PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo)))),
        ),
        ("you ", PlayerFilter::You),
        ("your opponents ", PlayerFilter::Opponent),
        ("each opponent ", PlayerFilter::Opponent),
        ("players ", PlayerFilter::Any),
        ("each player ", PlayerFilter::Any),
        ("all players ", PlayerFilter::Any),
    ]
    .into_iter()
    .find_map(|(p, f)| s.strip_prefix(p).map(|r| (f, r)))?;
    let player_effect = |e: PlayerModification| {
        Out::Other(StaticEffect::PlayerEffect {
            affected: who.clone(),
            effect: e,
        })
    };
    let mut outs = Vec::new();
    match rest {
        "have no maximum hand size" | "has no maximum hand size" => {
            outs.push(player_effect(PlayerModification::MaxHandSize(None)))
        }
        "have hexproof" | "has hexproof" => outs.push(player_effect(PlayerModification::Hexproof)),
        "have shroud" | "has shroud" => outs.push(player_effect(PlayerModification::Shroud)),
        "can't gain life" => outs.push(Out::Restr(Restriction::CantGainLife(who.clone()))),
        "can't lose life" => outs.push(Out::Restr(Restriction::CantLoseLife(who.clone()))),
        "can't search libraries" => outs.push(Out::Restr(Restriction::CantSearch(who.clone()))),
        "can't play lands" => outs.push(Out::Restr(Restriction::CantPlayLands(who.clone()))),
        "can't cast more than one spell each turn" => {
            outs.push(Out::Restr(Restriction::MaxSpellsPerTurn(who.clone(), 1)))
        }
        "can't draw more than one card each turn" => {
            outs.push(Out::Restr(Restriction::MaxDrawsPerTurn(who.clone(), 1)))
        }
        "can't draw cards" => outs.push(Out::Restr(Restriction::MaxDrawsPerTurn(who.clone(), 0))),
        // CR 307.1 timing for all their spells.
        "can cast spells only any time they could cast a sorcery"
        | "can cast spells only any time you could cast a sorcery" => {
            outs.push(Out::Restr(Restriction::SorcerySpeedOnly(who.clone())))
        }
        // "can't untap more than one land during their untap steps" (CR 502.3)
        _ if rest.starts_with("can't untap more than ") => {
            let r = rest.strip_prefix("can't untap more than ")?;
            let (n, r) = parse_number(r)?;
            let Value::Const(n) = n else { return None };
            let (f, _, tail) = parse_object_phrase(r)?;
            if !matches!(
                tail.trim(),
                "during their untap steps" | "during their untap step" | "during your untap step"
            ) || mentions_other_zones(&f)
            {
                return None;
            }
            outs.push(Out::Restr(Restriction::MaxUntaps {
                who: who.clone(),
                what: Filter::and(vec![f, Filter::Permanent]),
                n: n as u32,
            }));
        }
        _ => {
            // "can't cast [X] spells[ or activate abilities of Y]"
            let r = rest.strip_prefix("can't ")?;
            // "can't cast spells or activate abilities that aren't mana abilities"
            let (r, non_mana) = match r
                .strip_suffix(" or activate abilities that aren't mana abilities")
            {
                Some(c) => (c, true),
                None => (r, false),
            };
            let (cast, activate) = match r.split_once(" or activate abilities of ") {
                Some((c, a)) => (Some(c), Some(a)),
                None => match r.strip_prefix("activate abilities of ") {
                    Some(a) => (None, Some(a)),
                    None => (Some(r), None),
                },
            };
            if non_mana {
                if activate.is_some() {
                    return None;
                }
                outs.push(Out::Restr(Restriction::CantActivate {
                    who: who.clone(),
                    sources: Filter::Any,
                    include_mana: false,
                }));
            }
            if let Some(c) = cast {
                let what = c.strip_prefix("cast ")?;
                // "... from graveyards or libraries", "... from anywhere other than
                // their hands": the zone the card is cast from.
                let (what, zone) = match what.split_once(" from ") {
                    Some((w, z)) => (w, Some(cast_from_zones(z)?)),
                    None => (what, None),
                };
                let what = if what == "spells" {
                    Filter::Any
                } else {
                    let (f, plural) = whole_object_phrase(&union_nouns(what))?;
                    if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Spell)) {
                        return None;
                    }
                    without_spell(f)?
                };
                let what = match zone {
                    Some(z) => Filter::and(vec![what, z]),
                    None => what,
                };
                outs.push(Out::Restr(Restriction::CantCast {
                    who: who.clone(),
                    what,
                }));
            }
            if let Some(a) = activate {
                // Mana abilities are activated abilities too (CR 605.1a).
                let (sources, plural) = whole_object_phrase(&union_nouns(a))?;
                if !plural || mentions_other_zones(&sources) {
                    return None;
                }
                outs.push(Out::Restr(Restriction::CantActivate {
                    who: who.clone(),
                    // Only abilities of permanents, not of cards in other zones.
                    sources: Filter::and(vec![sources, Filter::Permanent]),
                    include_mana: true,
                }));
            }
        }
    }
    Some(Body {
        subject: Subject {
            filter: Filter::Any,
            it: None,
            hint: CardType::Creature,
            lands: false,
            creatures: false,
        },
        outs,
        also: vec![],
    })
}
