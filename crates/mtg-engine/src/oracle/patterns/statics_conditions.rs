//! Conditions of conditional static abilities ("as long as ...", CR 611.3a): the
//! state of the source or the object it's attached to ("as long as it's attacking",
//! "as long as enchanted creature is white"), graveyard thresholds (threshold,
//! delirium, descend), devotion (CR 700.5), life totals, and what players control.
//!
//! [`parse_static_condition`] also returns what a following "it" refers to. The forms
//! that don't depend on a referent are registered as [`ConditionPattern`]s too, so
//! intervening-if clauses can use them.

use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// The object a condition talks about: (selection, rest, whether the verb was
/// contracted into the subject as in "it's attacking").
fn condition_subject<'a>(c: &'a str, it: Option<&Sel>) -> Option<(Sel, &'a str, bool)> {
    for (p, sel) in [
        ("enchanted creature ", Sel::AttachedTo),
        ("equipped creature ", Sel::AttachedTo),
        ("enchanted permanent ", Sel::AttachedTo),
        ("enchanted land ", Sel::AttachedTo),
        ("enchanted artifact ", Sel::AttachedTo),
        ("~ ", Sel::This),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            return Some((sel, r, false));
        }
    }
    let it = it?;
    for p in ["it's ", "he's ", "she's ", "they're "] {
        if let Some(r) = c.strip_prefix(p) {
            return Some((it.clone(), r, true));
        }
    }
    for p in ["it ", "he ", "she ", "they "] {
        if let Some(r) = c.strip_prefix(p) {
            return Some((it.clone(), r, false));
        }
    }
    None
}

/// Parses a state predicate about one object: "is attacking", "isn't a creature",
/// "is equipped", "is white", "is a Human", "has a +1/+1 counter on it",
/// "has flying", "entered this turn". `contracted` means the verb was already consumed
/// ("it's attacking").
fn object_state(r: &str, sel: &Sel, contracted: bool) -> Option<Condition> {
    let r = end(r);
    // Verb.
    let (neg, state) = if contracted {
        if let Some(x) = r.strip_prefix("not ") {
            (true, x)
        } else {
            (false, r)
        }
    } else if let Some(x) = r
        .strip_prefix("isn't ")
        .or_else(|| r.strip_prefix("is not "))
        .or_else(|| r.strip_prefix("aren't "))
    {
        (true, x)
    } else if let Some(x) = r.strip_prefix("is ").or_else(|| r.strip_prefix("are ")) {
        (false, x)
    } else {
        return object_has(r, sel);
    };
    let f = state_filter(state)?;
    let c = Condition::SelMatches(sel.clone(), f);
    Some(if neg { Condition::Not(Box::new(c)) } else { c })
}

/// "attacking", "tapped", "equipped", "a creature", "white", "red or green",
/// "legendary", "a basic Mountain", "face down".
fn state_filter(s: &str) -> Option<Filter> {
    let s = end(s);
    let simple = match s {
        "attacking" => Some(Filter::Attacking),
        "blocking" => Some(Filter::Blocking),
        "attacking or blocking" => Some(Filter::Or(vec![Filter::Attacking, Filter::Blocking])),
        "tapped" => Some(Filter::Tapped),
        "untapped" => Some(Filter::Untapped),
        "equipped" => Some(Filter::Equipped),
        "enchanted" => Some(Filter::Enchanted),
        "modified" => Some(Filter::Modified),
        "face down" | "face-down" => Some(Filter::FaceDown),
        "colorless" => Some(Filter::Colorless),
        "multicolored" => Some(Filter::Multicolored),
        "monocolored" => Some(Filter::Monocolored),
        "legendary" => Some(Filter::Supertype(Supertype::Legendary)),
        _ => None,
    };
    if simple.is_some() {
        return simple;
    }
    // Color lists: "white", "red or green", "black or red".
    let colors: Option<Vec<Filter>> = s
        .split(" or ")
        .map(|w| Color::from_word(w.trim()).map(Filter::Color))
        .collect();
    if let Some(mut v) = colors {
        return Some(if v.len() == 1 {
            v.pop()?
        } else {
            Filter::Or(v)
        });
    }
    // "a creature", "an Equipment", "a Human", "a basic Mountain".
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(f)
}

/// "has a +1/+1 counter on it", "has N or more quest counters on it", "has flying",
/// "entered this turn", "attacked this turn".
fn object_has(r: &str, sel: &Sel) -> Option<Condition> {
    let r = end(r);
    match r {
        "entered this turn" | "entered the battlefield this turn" => {
            return Some(Condition::SelMatches(sel.clone(), Filter::EnteredThisTurn))
        }
        "attacked this turn" => {
            return Some(Condition::SelMatches(sel.clone(), Filter::AttackedThisTurn))
        }
        _ => {}
    }
    let h = r.strip_prefix("has ").or_else(|| r.strip_prefix("have "))?;
    if let Some((cmp, n, kind)) = counters_on_it(h) {
        let count = Value::CountersOn(Box::new(sel.clone()), kind);
        return Some(Condition::Compare(count, cmp, n));
    }
    let k = KeywordKind::from_name(h)?;
    Some(Condition::SelMatches(sel.clone(), Filter::HasKeyword(k)))
}

/// "a +1/+1 counter on it" → (>=, 1, "+1/+1"); "two or more ki counters on it";
/// "a counter on it" → any kind; "no counters on it" → (=, 0, None).
pub(crate) fn counters_on_it(h: &str) -> Option<(Cmp, Value, Option<CounterKind>)> {
    let h = end(h);
    let body = h
        .strip_suffix(" on it")
        .or_else(|| h.strip_suffix(" on ~"))
        .or_else(|| h.strip_suffix(" on him"))
        .or_else(|| h.strip_suffix(" on her"))?;
    let (cmp, n, rest) = if let Some(r) = body.strip_prefix("no ") {
        (Cmp::Eq, Value::c(0), r)
    } else if let Some(r) = body.strip_prefix("one or more ") {
        (Cmp::Ge, Value::c(1), r)
    } else {
        let (n, r) = parse_number(body)?;
        if let Some(r2) = strip(r, "or more") {
            (Cmp::Ge, n, r2)
        } else if matches!(n, Value::Const(1)) {
            (Cmp::Ge, n, r)
        } else {
            return None;
        }
    };
    let rest = rest.trim();
    if rest == "counter" || rest == "counters" {
        return Some((cmp, n, None));
    }
    let (kind, tail) = rest.split_once(' ')?;
    if tail != "counter" && tail != "counters" {
        return None;
    }
    if !(kind.starts_with('+') || kind.starts_with('-') || kind.chars().all(|c| c.is_alphabetic()))
    {
        return None;
    }
    Some((cmp, n, Some(kind.into())))
}

/// "N or more", "N or less", "N or fewer", "N or greater", "at least N", "N".
fn amount_cmp(s: &str) -> Option<(Cmp, Value, &str)> {
    if let Some(r) = s.strip_prefix("at least ") {
        let (n, r) = parse_number(r)?;
        return Some((Cmp::Ge, n, r));
    }
    if let Some(r) = s.strip_prefix("no ") {
        return Some((Cmp::Eq, Value::c(0), r));
    }
    let (n, r) = parse_number(s)?;
    if let Some(r2) = strip(r, "or more").or_else(|| strip(r, "or greater")) {
        return Some((Cmp::Ge, n, r2));
    }
    if let Some(r2) = strip(r, "or less").or_else(|| strip(r, "or fewer")) {
        return Some((Cmp::Le, n, r2));
    }
    None
}

fn your_graveyard() -> Filter {
    Filter::and(vec![
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
    ])
}

/// Graveyard thresholds: "there are seven or more cards in your graveyard" (threshold),
/// "seven or more cards are in your graveyard", "there are four or more card types
/// among cards in your graveyard" (delirium), "there are four or more permanent cards
/// in your graveyard" (descend 4).
fn graveyard_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    // "there are N or more X in your graveyard" / "N or more X are in your graveyard"
    let (cmp, n, rest) = if let Some(r) = c.strip_prefix("there are ") {
        let (cmp, n, r) = amount_cmp(r)?;
        (cmp, n, r.trim().to_string())
    } else {
        let (cmp, n, r) = amount_cmp(c)?;
        let r = r.trim();
        let body = r
            .strip_suffix(" are in your graveyard")
            .or_else(|| r.strip_suffix(" is in your graveyard"))?;
        (cmp, n, format!("{body} in your graveyard"))
    };
    let rest = rest.as_str();
    if rest == "cards in your graveyard" || rest == "card in your graveyard" {
        return Some(Condition::Compare(
            Value::GraveyardSize(PlayerRef::You),
            cmp,
            n,
        ));
    }
    if rest == "card types among cards in your graveyard" {
        return Some(Condition::Compare(
            Value::CardTypesAmong(your_graveyard()),
            cmp,
            n,
        ));
    }
    // "permanent cards in your graveyard", "creature cards in your graveyard"
    let (f, _, tail) = parse_object_phrase(rest)?;
    if !end(tail).is_empty() || f.zone() != Some(ZoneKind::Graveyard) {
        return None;
    }
    Some(Condition::Compare(Value::Count(f), cmp, n))
}

/// Devotion (CR 700.5): "your devotion to black is less than five", "your devotion to
/// white and black is less than seven".
fn devotion_condition(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("your devotion to ")?;
    let (colors, rest) = r.split_once(" is ")?;
    let mut set = ColorSet::NONE;
    for w in colors.split(" and ") {
        set.insert(Color::from_word(w.trim())?);
    }
    let (cmp, rest) = if let Some(x) = rest.strip_prefix("less than ") {
        (Cmp::Lt, x)
    } else if let Some(x) = rest.strip_prefix("greater than ") {
        (Cmp::Gt, x)
    } else {
        let (cmp, n, tail) = amount_cmp(rest)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Compare(Value::Devotion(set), cmp, n));
    };
    let (n, tail) = parse_number(rest)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(Value::Devotion(set), cmp, n))
}

/// Life totals: "an opponent has 10 or less life", "your life total is less than or
/// equal to half your starting life total", "you have at least 7 life more than your
/// starting life total", "you have 30 or more life".
fn life_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    if c == "your life total is less than or equal to half your starting life total" {
        // CR 119.1: compare twice the life total with the starting total (no rounding).
        return Some(Condition::Compare(
            Value::Mul(
                Box::new(Value::c(2)),
                Box::new(Value::LifeTotal(PlayerRef::You)),
            ),
            Cmp::Le,
            Value::StartingLife,
        ));
    }
    if c == "your life total is greater than your starting life total"
        || c == "you have more life than your starting life total"
    {
        return Some(Condition::Compare(
            Value::LifeTotal(PlayerRef::You),
            Cmp::Gt,
            Value::StartingLife,
        ));
    }
    if let Some(r) = c.strip_prefix("you have at least ") {
        let (n, r) = parse_number(r)?;
        if end(r) == "life more than your starting life total" {
            return Some(Condition::Compare(
                Value::LifeTotal(PlayerRef::You),
                Cmp::Ge,
                Value::Sum(vec![Value::StartingLife, n]),
            ));
        }
        if end(r) == "life" {
            return Some(Condition::Compare(
                Value::LifeTotal(PlayerRef::You),
                Cmp::Ge,
                n,
            ));
        }
        return None;
    }
    for (p, who) in [
        ("an opponent has ", PlayerRef::EachOpponent),
        ("you have ", PlayerRef::You),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            let (cmp, n, tail) = amount_cmp(r)?;
            if end(tail) != "life" {
                return None;
            }
            return Some(match who {
                PlayerRef::You => Condition::Compare(Value::LifeTotal(PlayerRef::You), cmp, n),
                other => Condition::PlayerMatches(other, PlayerFilter::Life(cmp, Box::new(n))),
            });
        }
    }
    None
}

/// What players control, beyond the built-in "you control a/N or more X":
/// "you control no X", "you don't control a X", "an opponent controls a X",
/// "defending player controls a X", "you control N or fewer X".
fn control_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let phrase = |r: &str| -> Option<Filter> {
        let (f, _, tail) = parse_object_phrase(r)?;
        end(tail).is_empty().then_some(f)
    };
    let article = |r: &'_ str| -> Option<String> {
        r.strip_prefix("a ")
            .or_else(|| r.strip_prefix("an "))
            .map(str::to_string)
    };
    if let Some(r) = c.strip_prefix("you control no ") {
        let f = phrase(r)?;
        return Some(Condition::Not(Box::new(Condition::Exists(f.you_control()))));
    }
    if let Some(r) = c.strip_prefix("you don't control ") {
        let f = phrase(&article(r)?)?;
        return Some(Condition::Not(Box::new(Condition::Exists(f.you_control()))));
    }
    for (p, rel) in [
        ("an opponent controls ", PlayerRel::Opponent),
        ("defending player controls ", PlayerRel::Defending),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            if let Some((cmp, n, rest)) = amount_cmp(r) {
                let f = phrase(rest)?;
                // "an opponent controls N or more X" needs one opponent to control them
                // all; that's only expressible for a single opponent.
                if rel != PlayerRel::Defending {
                    return None;
                }
                return Some(Condition::Compare(
                    Value::Count(Filter::and(vec![f, Filter::ControlledBy(rel)])),
                    cmp,
                    n,
                ));
            }
            let f = phrase(&article(r)?)?;
            return Some(Condition::Exists(Filter::and(vec![
                f,
                Filter::ControlledBy(rel),
            ])));
        }
    }
    if let Some(r) = c.strip_prefix("you control ") {
        let (cmp, n, rest) = amount_cmp(r)?;
        if cmp != Cmp::Le {
            return None;
        }
        let f = phrase(rest)?;
        return Some(Condition::Compare(Value::Count(f.you_control()), cmp, n));
    }
    None
}

/// Turn history: "you gained life this turn", "you've drawn two or more cards this
/// turn", "a creature died this turn", "you've gained N or more life this turn".
fn history_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    match c {
        "you gained life this turn" | "you've gained life this turn" => {
            return Some(Condition::Compare(
                Value::LifeGainedThisTurn(PlayerRef::You),
                Cmp::Gt,
                Value::c(0),
            ))
        }
        "you lost life this turn" | "you've lost life this turn" => {
            return Some(Condition::Compare(
                Value::LifeLostThisTurn(PlayerRef::You),
                Cmp::Gt,
                Value::c(0),
            ))
        }
        "a creature died this turn" => {
            return Some(Condition::Compare(
                Value::CreaturesDiedThisTurn,
                Cmp::Gt,
                Value::c(0),
            ))
        }
        "you have the initiative" => return Some(Condition::HasInitiative),
        "you have max speed" => return Some(Condition::MaxSpeed),
        _ => {}
    }
    if let Some(r) = c
        .strip_prefix("you've drawn ")
        .or_else(|| c.strip_prefix("you have drawn "))
    {
        let (cmp, n, tail) = amount_cmp(r)?;
        if end(tail) != "cards this turn" {
            return None;
        }
        return Some(Condition::Compare(
            Value::CardsDrawnThisTurn(PlayerRef::You),
            cmp,
            n,
        ));
    }
    if let Some(r) = c.strip_prefix("you've gained ") {
        let (cmp, n, tail) = amount_cmp(r)?;
        if end(tail) != "life this turn" {
            return None;
        }
        return Some(Condition::Compare(
            Value::LifeGainedThisTurn(PlayerRef::You),
            cmp,
            n,
        ));
    }
    None
}

/// "~'s power is 4 or greater", "~'s toughness is 3 or less".
fn stat_condition(c: &str, it: Option<&Sel>) -> Option<Condition> {
    let c = end(c);
    let (sel, r) = if let Some(r) = c.strip_prefix("~'s ") {
        (Sel::This, r)
    } else if let Some(r) = c.strip_prefix("its ") {
        (it?.clone(), r)
    } else if let Some(r) = c
        .strip_prefix("enchanted creature's ")
        .or_else(|| c.strip_prefix("equipped creature's "))
    {
        (Sel::AttachedTo, r)
    } else {
        return None;
    };
    let (v, r) = if let Some(r) = r.strip_prefix("power is ") {
        (Value::PowerOf(Box::new(sel)), r)
    } else if let Some(r) = r.strip_prefix("toughness is ") {
        (Value::ToughnessOf(Box::new(sel)), r)
    } else {
        return None;
    };
    let (cmp, n, tail) = amount_cmp(r)?;
    end(tail)
        .is_empty()
        .then_some(Condition::Compare(v, cmp, n))
}

/// Hand size: "you have no cards in hand" (hellbent).
fn hand_condition(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you have ")?;
    let (cmp, n, tail) = amount_cmp(r)?;
    if end(tail) != "cards in hand" && end(tail) != "card in hand" {
        return None;
    }
    Some(Condition::Compare(Value::HandSize(PlayerRef::You), cmp, n))
}

/// Conditions that don't depend on what "it" refers to.
fn referent_free_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    if matches!(c, "it's your turn" | "it's not your turn") {
        return None;
    }
    graveyard_condition(c)
        .or_else(|| devotion_condition(c))
        .or_else(|| life_condition(c))
        .or_else(|| hand_condition(c))
        .or_else(|| control_condition(c))
        .or_else(|| history_condition(c))
        .or_else(|| stat_condition(c, None))
        .or_else(|| {
            let (sel, r, contracted) = condition_subject(c, None)?;
            object_state(r, &sel, contracted)
        })
}

/// Parses the condition of a conditional static ability. `it` is what "it" refers to
/// in the condition (the ability's subject for a trailing "as long as it's attacking").
/// Returns the condition and the object a following "it" refers to ("as long as
/// enchanted creature is white, it gets +1/+1").
pub(crate) fn parse_static_condition(
    c: &str,
    it: Option<&Sel>,
    ctx: &CompileContext,
) -> Option<(Condition, Option<Sel>)> {
    let c = end(c);
    // Pronouns first, so they're never read as some other ability's "it".
    if let Some((sel, r, contracted)) = condition_subject(c, it) {
        if let Some(cond) = object_state(r, &sel, contracted) {
            return Some((cond, Some(sel)));
        }
    }
    if let Some(cond) = stat_condition(c, it) {
        return Some((cond, it.cloned()));
    }
    if let Some(cond) = referent_free_condition(c) {
        return Some((cond, None));
    }
    // Built-in forms ("you control an artifact", "it's your turn") and other patterns.
    let cond = crate::oracle::statics::parse_condition(c, ctx)?;
    Some((cond, None))
}

/// The forms offered to other abilities (intervening "if" clauses). Conditions about the
/// enchanted/equipped object are left to static abilities: the effects after such an
/// "if" usually say "it", which the effect parser can't yet tie to that object.
fn condition_pattern(c: &str) -> Option<Condition> {
    let c = end(c);
    if c.starts_with("enchanted ") || c.starts_with("equipped ") {
        return None;
    }
    referent_free_condition(c)
}

inventory::submit! {
    ConditionPattern {
        name: "statics: graveyard, devotion, life, control, history, object state",
        priority: 50,
        parse: condition_pattern,
    }
}
