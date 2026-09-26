//! Pump spells and combat tricks: temporary changes to creatures made by resolving spells
//! and abilities (CR 611.2, layers 6 and 7, CR 613.1f, 613.4).
//!
//! - A leading duration applies to the whole sentence: "Until end of turn, target
//!   creature gets +1/+1 and another target creature gets -1/-1.", "Until your next turn,
//!   creatures you control get +1/+1 and gain lifelink."
//! - A list of predicates about one subject, in any order: "gains trample and gets
//!   +X/+X, where X is ...", "has base power and toughness 1/1 and gains flying", "loses
//!   all abilities and has base power and toughness 1/1", "each get +1/+1 and gain
//!   \"When this creature dies, draw a card.\"". Quoted abilities are compiled
//!   recursively; "this creature" in them is the object that has the ability, and the
//!   ability belongs to it (CR 113.6, 613.1f). A quote that names the card itself is
//!   left unsupported (see `statics::quote_names_card`).
//! - One choice among alternatives, made by the controller as the effect is created:
//!   "gains your choice of flying, vigilance, or lifelink", "gets +1/-1 or -1/+1".
//! - "Switch target creature's power and toughness until end of turn" (layer 7d, CR
//!   613.4d).
//! - "Whenever ~ becomes blocked, it gets +1/+1 until end of turn for each creature
//!   blocking it [beyond the first]": "it" there is the source, so "creature blocking
//!   it" is a creature blocking the source.
//!
//! Values ("for each", "where X is") are determined once, as the effect is created
//! (CR 608.2h), and the affected set is locked in then (CR 611.2c) — the resolver does
//! both for [`Effect::Modify`].

use super::statics::{mask_quotes, quote_names_card};
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{
    duration_suffix, keyword_mods, object_ref, parse_clause, parse_pt_mod, Builder,
};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;
use crate::oracle::CompileContext;
use crate::types::*;

// ---------------------------------------------------------------------------
// Leading durations
// ---------------------------------------------------------------------------

const LEADING_DURATIONS: &[(&str, Duration)] = &[
    ("until end of turn, ", Duration::EndOfTurn),
    ("until your next turn, ", Duration::UntilYourNextTurn),
    ("until end of combat, ", Duration::EndOfCombat),
    (
        "until the end of your next turn, ",
        Duration::UntilEndOfYourNextTurn,
    ),
];

/// Gives each effect of `e` the duration `dur`. Only effects that create continuous
/// effects without a duration of their own qualify (a duration already written for the
/// same period is kept); anything else means the sentence isn't a plain "until ..., X".
fn with_duration(e: Effect, dur: &Duration) -> Option<Effect> {
    let same = |d: &Duration| {
        matches!(d, Duration::Permanent)
            // (The leading durations are all unit variants.)
            || std::mem::discriminant(d) == std::mem::discriminant(dur)
            || (matches!(dur, Duration::EndOfTurn) && matches!(d, Duration::ThisTurn))
    };
    Some(match e {
        Effect::Seq(v) => Effect::Seq(
            v.into_iter()
                .map(|x| with_duration(x, dur))
                .collect::<Option<Vec<_>>>()?,
        ),
        Effect::Modify {
            what,
            mods,
            duration,
        } if same(&duration) => Effect::Modify {
            what,
            mods,
            duration: dur.clone(),
        },
        Effect::AddRestriction {
            restriction,
            duration,
        } if same(&duration) => Effect::AddRestriction {
            restriction,
            duration: match duration {
                Duration::Permanent => dur.clone(),
                d => d,
            },
        },
        _ => return None,
    })
}

/// "until end of turn, [clause]", "until your next turn, [clause]" (CR 611.2a).
fn leading_duration(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (dur, rest) = LEADING_DURATIONS
        .iter()
        .find_map(|(p, d)| l.strip_prefix(p).map(|r| (d.clone(), r)))?;
    // "until end of turn, whenever ..." (a delayed trigger) and "until end of turn, you
    // may ..." (a permission) are other patterns' business: only continuous effects on
    // objects qualify here.
    let e = parse_clause(rest, b)?;
    with_duration(e, &dur)
}

inventory::submit! { EffectPattern { name: "pump: leading duration", priority: 90, parse: leading_duration } }

// ---------------------------------------------------------------------------
// Predicate lists
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verb {
    /// "gets +N/+N"
    Get,
    /// "gains [keywords and quoted abilities]"
    Gain,
    /// "has base power and toughness N/N"
    BasePt,
    /// "loses all abilities"
    LoseAll,
}

/// The verb starting at `s` (a word boundary), and the text after it.
fn verb_at(s: &str) -> Option<(Verb, &str)> {
    for (p, v) in [
        ("gets ", Verb::Get),
        ("get ", Verb::Get),
        ("gains ", Verb::Gain),
        ("gain ", Verb::Gain),
        ("has base power and toughness ", Verb::BasePt),
        ("have base power and toughness ", Verb::BasePt),
        ("loses all abilities", Verb::LoseAll),
        ("lose all abilities", Verb::LoseAll),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if v == Verb::Get && !r.starts_with(['+', '-']) {
                return None;
            }
            if v == Verb::LoseAll && !(r.is_empty() || r.starts_with([',', ' '])) {
                return None;
            }
            return Some((v, r));
        }
    }
    None
}

/// Splits "gets +1/+1, gains flying, and gains \"#0\"" into its predicates. Each new
/// predicate starts with a verb after a connector ("," / "and" / ", and"). The text
/// must start with a verb.
fn split_predicates(s: &str) -> Option<Vec<(Verb, String)>> {
    let s = s.trim();
    let (mut verb, r) = verb_at(s)?;
    let mut body_start = s.len() - r.len();
    let mut out: Vec<(Verb, String)> = Vec::new();
    let mut i = body_start;
    while i < s.len() {
        let rest = &s[i..];
        let conn = [", and ", ", ", " and "]
            .into_iter()
            .find(|c| rest.starts_with(c));
        if let Some((v, r)) = conn.and_then(|c| verb_at(&rest[c.len()..])) {
            out.push((verb, s[body_start..i].trim().to_string()));
            verb = v;
            body_start = s.len() - r.len();
            i = body_start;
            continue;
        }
        i += rest.chars().next().map_or(1, char::len_utf8);
    }
    out.push((verb, s[body_start..].trim().to_string()));
    Some(out)
}

/// Splits a grant list at ", and ", ", ", " and " (quotes are masked), keeping
/// "protection from X and from Y" together.
fn grant_items(s: &str) -> Vec<String> {
    let mut v: Vec<&str> = vec![s];
    for sep in [", and ", ", ", " and "] {
        v = v
            .into_iter()
            .flat_map(|p| p.split(sep))
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
    }
    let mut items: Vec<String> = Vec::new();
    for item in v {
        match items.last_mut() {
            Some(last) if item.starts_with("from ") && last.starts_with("protection from ") => {
                last.push_str(" and ");
                last.push_str(item);
            }
            _ => items.push(item.to_string()),
        }
    }
    items
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

/// Compiles a quoted ability granted by an effect. `~` in it is the object that gets
/// the ability (CR 113.6); the card itself can't be named there (the quote is left
/// unsupported instead).
fn quoted_abilities(quote_lower: &str, hint: CardType, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let norm = crate::oracle::normalize(&crate::oracle::raw_text(), ctx);
    let orig = quoted_segments(&norm)
        .into_iter()
        .find(|q| q.to_lowercase() == quote_lower)?
        .to_string();
    if quote_names_card(&orig, ctx) {
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
    let blocks = crate::oracle::split_abilities(&orig);
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

fn grants(
    body: &str,
    quotes: &[String],
    hint: CardType,
    ctx: &CompileContext,
) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    for item in grant_items(body) {
        if let Some(k) = item
            .strip_prefix("\"#")
            .and_then(|x| x.strip_suffix('"'))
            .and_then(|x| x.parse::<usize>().ok())
        {
            for a in quoted_abilities(quotes.get(k)?, hint, ctx)? {
                out.push(Modification::AddAbility(a));
            }
        } else if item == "all creature types" {
            // CR 205.3m: every creature type (layer 4).
            out.push(Modification::AllCreatureTypes);
        } else {
            if item.contains('"') {
                return None;
            }
            out.extend(keyword_mods(&item)?);
        }
    }
    (!out.is_empty()).then_some(out)
}

fn mentions_x(v: &Value) -> bool {
    match v {
        Value::X => true,
        Value::Diff(a, b) => mentions_x(a) || mentions_x(b),
        _ => false,
    }
}

fn subst_x(v: Value, with: &Value) -> Value {
    match v {
        Value::X => with.clone(),
        Value::Diff(a, b) => Value::Diff(Box::new(subst_x(*a, with)), Box::new(subst_x(*b, with))),
        other => other,
    }
}

/// "for each [thing] [beyond the first]": how many there are.
fn for_each_count(s: &str, b: &mut Builder) -> Option<Value> {
    let s = s.trim();
    let (s, beyond_first) = match s.strip_suffix(" beyond the first") {
        Some(r) => (r, true),
        None => (s, false),
    };
    let v = if s == "basic land type among lands you control" {
        // Domain (CR 207.2c).
        Value::Domain
    } else {
        let (v, rest) = parse_value_phrase(&format!("the number of {s}"), b)?;
        if !end(&rest).trim().is_empty() {
            return None;
        }
        v
    };
    Some(if beyond_first {
        Value::Max(
            Box::new(Value::Diff(Box::new(v), Box::new(Value::c(1)))),
            Box::new(Value::c(0)),
        )
    } else {
        v
    })
}

/// A duration written between a P/T change and its "for each" ("gets +1/+1 until end
/// of turn for each creature blocking it").
fn inner_duration(s: &str) -> (Option<Duration>, &str) {
    for (p, d) in [
        ("until end of turn ", Duration::EndOfTurn),
        ("until your next turn ", Duration::UntilYourNextTurn),
        ("until end of combat ", Duration::EndOfCombat),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return (Some(d), r);
        }
    }
    (None, s)
}

/// "[subject] [each] PREDICATE (, | and | , and) PREDICATE ... [duration] [, where X is
/// VALUE]".
fn predicate_list(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (masked, quotes) = mask_quotes(l)?;
    // In "Whenever ~ becomes blocked, it gets ... for each creature blocking it", "it"
    // is the source.
    let masked = if matches!(b.it, Sel::This) {
        masked.replace("blocking it", "blocking ~")
    } else {
        masked
    };
    let (main, where_text) = match masked.split_once(", where x is ") {
        Some((m, w)) => (m.to_string(), Some(w.to_string())),
        None => (masked.clone(), None),
    };
    let (mut dur, main) = duration_suffix(&main);
    let main = main.to_string();
    let hint = if main
        .split_whitespace()
        .any(|w| matches!(w, "land" | "lands"))
    {
        CardType::Land
    } else {
        CardType::Creature
    };
    let (what, rest) = object_ref(&main, b)?;
    let rest = rest.trim();
    let rest = rest.strip_prefix("each ").unwrap_or(rest);
    let preds = split_predicates(rest)?;
    // The value of X, parsed after the subject ("its power" names the subject).
    let x = match &where_text {
        Some(w) => {
            let w = match w.strip_prefix("that creature's ") {
                Some(r) if !matches!(b.it, Sel::This | Sel::None) => format!("its {r}"),
                _ => w.clone(),
            };
            let (v, rest) = parse_value_phrase(&w, b)?;
            if !end(&rest).trim().is_empty() {
                return None;
            }
            Some(v)
        }
        None => None,
    };
    let mut used_x = false;
    let mut mods = Vec::new();
    // "gets +1/-1 or -1/+1", "gains your choice of flying or lifelink": the controller
    // chooses one alternative as the effect is created (at most one such choice).
    let mut choice: Option<Vec<(String, Modification)>> = None;
    for (verb, body) in &preds {
        match verb {
            Verb::Get => {
                let mut sub_x = |p: Value, t: Value| match &x {
                    Some(x) if mentions_x(&p) || mentions_x(&t) => {
                        used_x = true;
                        // CR 107.1b: a negative X uses 0 instead.
                        let x = Value::Max(Box::new(x.clone()), Box::new(Value::c(0)));
                        (subst_x(p, &x), subst_x(t, &x))
                    }
                    _ => (p, t),
                };
                let (p, t, tail) = parse_pt_mod(body)?;
                let (p, t) = sub_x(p, t);
                let tail = tail.trim();
                if let Some(alt) = tail.strip_prefix("or ") {
                    let (p2, t2, rest) = parse_pt_mod(alt)?;
                    if !rest.trim().is_empty() || choice.is_some() {
                        return None;
                    }
                    let (p2, t2) = sub_x(p2, t2);
                    let first = body[..body.len() - tail.len()].trim().to_string();
                    choice = Some(vec![
                        (first, Modification::ModifyPT(p, t)),
                        (alt.trim().to_string(), Modification::ModifyPT(p2, t2)),
                    ]);
                    continue;
                }
                let (p, t) = if tail.is_empty() {
                    (p, t)
                } else {
                    let (d, tail) = inner_duration(tail);
                    if let Some(d) = d {
                        if !matches!(dur, Duration::Permanent) {
                            return None;
                        }
                        dur = d;
                    }
                    let f = tail.strip_prefix("for each ")?;
                    let n = for_each_count(f, b)?;
                    let (Some(pc), Some(tc)) = (p.as_const(), t.as_const()) else {
                        return None;
                    };
                    (
                        Value::Mul(Box::new(Value::c(pc)), Box::new(n.clone())),
                        Value::Mul(Box::new(Value::c(tc)), Box::new(n)),
                    )
                };
                mods.push(Modification::ModifyPT(p, t));
            }
            Verb::Gain => {
                if let Some(list) = body.strip_prefix("your choice of ") {
                    if choice.is_some() {
                        return None;
                    }
                    choice = Some(keyword_choice(list)?);
                    continue;
                }
                mods.extend(grants(body, &quotes, hint, b.ctx)?);
            }
            Verb::BasePt => {
                // Layer 7b (CR 613.4b).
                let (p, t) = body.split_once('/')?;
                // X is the "where X is" value, or else the spell's X. (Setting a value
                // may use a negative number, CR 107.1b.)
                let mut val = |s: &str| -> Option<Value> {
                    if s == "x" {
                        used_x = true;
                        Some(x.clone().unwrap_or(Value::X))
                    } else {
                        Some(Value::c(s.parse().ok()?))
                    }
                };
                let pv = val(p)?;
                let tv = val(t)?;
                mods.push(Modification::SetPT(Some(pv), Some(tv)));
            }
            Verb::LoseAll => {
                if !body.is_empty() {
                    return None;
                }
                mods.push(Modification::RemoveAllAbilities);
            }
        }
    }
    if where_text.is_some() && !used_x {
        return None;
    }
    if let Some(options) = choice {
        return Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options: options
                .into_iter()
                .map(|(label, m)| {
                    let mut all = mods.clone();
                    all.push(m);
                    (
                        label,
                        Effect::Modify {
                            what: what.clone(),
                            mods: all,
                            duration: dur.clone(),
                        },
                    )
                })
                .collect(),
        });
    }
    if mods.is_empty() {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods,
        duration: dur,
    })
}

/// "flying, vigilance, or lifelink", "double strike or trample": one keyword each.
fn keyword_choice(list: &str) -> Option<Vec<(String, Modification)>> {
    let items: Vec<&str> = list
        .split(", or ")
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if items.len() < 2 {
        return None;
    }
    items
        .into_iter()
        .map(|k| {
            let mut m = keyword_mods(k)?;
            (m.len() == 1).then(|| (k.to_string(), m.remove(0)))
        })
        .collect()
}

inventory::submit! { EffectPattern { name: "pump: predicate list", priority: 95, parse: predicate_list } }

// ---------------------------------------------------------------------------
// Switching power and toughness
// ---------------------------------------------------------------------------

/// "switch target creature's power and toughness until end of turn", "switch the power
/// and toughness of each of up to two target creatures until end of turn", "switch each
/// creature's power and toughness until end of turn" (CR 613.4d).
fn switch_pt(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("switch ")?;
    let (dur, r) = duration_suffix(r);
    let subject = if let Some(s) = r.strip_suffix("'s power and toughness") {
        s.to_string()
    } else if let Some(s) = r.strip_prefix("the power and toughness of ") {
        s.strip_prefix("each of ").unwrap_or(s).to_string()
    } else {
        return None;
    };
    let (what, rest) = object_ref(&subject, b)?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::SwitchPT],
        duration: dur,
    })
}

inventory::submit! { EffectPattern { name: "pump: switch power and toughness", priority: 100, parse: switch_pt } }

// ---------------------------------------------------------------------------
// "When this creature dies, return it to the battlefield ..."
// ---------------------------------------------------------------------------

/// "with a +1/+1 counter on it", "with two +1/+1 counters on it".
fn with_counters_on_it(s: &str) -> Option<Vec<(CounterKind, Value)>> {
    let (n, r) = parse_number(s)?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r
        .strip_prefix("counters on it")
        .or_else(|| r.strip_prefix("counter on it"))?;
    r.is_empty().then(|| vec![(kind, n)])
}

/// "return it to the battlefield [tapped] under its owner's control [with a +1/+1
/// counter on it]", "return that card to the battlefield under your control": a specific
/// object named earlier (the creature that died, a target card) returns (CR 400.7e: a
/// dies trigger finds the card in the graveyard). The player who puts it onto the
/// battlefield controls it unless the effect says otherwise (CR 110.2a); counters it
/// enters with are placed as it enters (CR 122.6).
fn return_to_battlefield(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("return ")?;
    let (what, tail) = object_ref(r, b)?;
    // What returns must be findable where it is: the source, the object of a dies
    // trigger (CR 400.7e), or a target card in a graveyard. (A permanent exiled earlier
    // in the same effect is a new object in exile, CR 400.7; that's another pattern's.)
    let ok = match &what {
        Sel::This | Sel::TriggerObject => true,
        Sel::Target(slot) => b.targets.get(*slot as usize).is_some_and(
            |t| matches!(&t.what, TargetKind::Object(f) if f.zone() == Some(ZoneKind::Graveyard)),
        ),
        _ => false,
    };
    if !ok {
        return None;
    }
    let tail = tail.trim();
    let tail = tail
        .strip_prefix("from your graveyard")
        .map(str::trim_start)
        .unwrap_or(tail);
    let mut t = tail.strip_prefix("to the battlefield")?.trim_start();
    let mut d = Destination::battlefield().under_your_control();
    loop {
        if let Some(x) = t.strip_prefix("tapped") {
            d.tapped = true;
            t = x.trim_start();
        } else if let Some(x) = t.strip_prefix("under your control") {
            t = x.trim_start();
        } else if let Some(x) = t
            .strip_prefix("under its owner's control")
            .or_else(|| t.strip_prefix("under their owner's control"))
            .or_else(|| t.strip_prefix("under their owners' control"))
        {
            d.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
            t = x.trim_start();
        } else if let Some(x) = t.strip_prefix("with ") {
            d.with_counters = with_counters_on_it(x)?;
            t = "";
        } else {
            break;
        }
    }
    if !t.is_empty() {
        return None;
    }
    // "It deals 2 damage to each opponent." afterwards: the permanent it became.
    b.it = Sel::Var(vars::IT);
    Some(Effect::Move { what, to: d })
}

inventory::submit! { EffectPattern { name: "pump: return it to the battlefield under its owner's control", priority: 100, parse: return_to_battlefield } }
