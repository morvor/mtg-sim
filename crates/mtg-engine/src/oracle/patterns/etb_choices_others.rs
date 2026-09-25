//! Replacement effects that modify how *other* permanents enter (CR 614.1d, 614.12):
//! "Creatures your opponents control enter tapped."

use super::{AbilityPattern, ConditionPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// An entering object isn't on the battlefield yet, so "permanent" in the subject means
/// any object that would become a permanent.
fn entering_filter(f: Filter) -> Filter {
    match f {
        Filter::Permanent => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(entering_filter).collect()),
        Filter::Or(v) => Filter::Or(v.into_iter().map(entering_filter).collect()),
        other => other,
    }
}

/// A plural subject that may list several kinds of objects sharing a controller suffix:
/// "creatures and nonbasic lands your opponents control" means creatures your opponents
/// control and nonbasic lands your opponents control.
fn subject_list(subj: &str) -> Option<Filter> {
    let (heads, suffix) = [
        " you control",
        " your opponents control",
        " an opponent controls",
    ]
    .iter()
    .find_map(|sfx| subj.strip_suffix(sfx).map(|h| (h, *sfx)))
    .unwrap_or((subj, ""));
    let mut parts = Vec::new();
    for p in heads
        .split(", and ")
        .flat_map(|p| p.split(", or "))
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" and "))
        .flat_map(|p| p.split(" or "))
    {
        let (f, plural, tail) = parse_object_phrase(p.trim())?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        parts.push(f);
    }
    let head = match parts.len() {
        0 => return None,
        1 => parts.pop().unwrap(),
        _ => Filter::Or(parts),
    };
    if suffix.is_empty() {
        return Some(head);
    }
    // Parse the controller suffix on a neutral head noun ("card" = any object).
    let probe = format!("card{suffix}");
    let (sf, _, tail) = parse_object_phrase(&probe)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Filter::and(vec![head, sf]))
}

/// "[objects] enter tapped", e.g. "Artifacts, creatures, and lands your opponents
/// control enter tapped." (a list in the subject is a union).
fn others_enter_tapped(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let subj = l.strip_suffix(" enter tapped")?;
    if subj.starts_with('~') || subj.contains(" this turn") {
        return None;
    }
    let f = subject_list(subj)?;
    let def = ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(entering_filter(f)),
        action: ReplacementAction::EnterTapped,
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        block,
    )])
}

/// "Each other creature you control of the chosen type enters with an additional +1/+1
/// counter on it.", "Nontoken creatures you control enter with an additional +1/+1
/// counter on them for each ...": ETB replacement effects on other permanents
/// (CR 614.1d, 122.6).
fn others_enter_with_counters(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let l = end(l);
    // "As long as ~ is in your graveyard, each Human creature you control enters with an
    // additional +1/+1 counter on it." (a static ability functioning from the graveyard)
    let (l, zone) = match l.strip_prefix("as long as ~ is in your graveyard, ") {
        Some(r) => (r, FunctionZone::Graveyard),
        None => (l, FunctionZone::Battlefield),
    };
    let (subj, rest) = if let Some(r) = l.strip_prefix("each ") {
        let (s, rest) = r.split_once(" enters with ")?;
        // "each creature you control that's a Wolf or a Werewolf"
        let (s, types) = match s.split_once(" that's a ") {
            Some((a, b)) => {
                let mut v = Vec::new();
                for t in b.split(" or a ").flat_map(|x| x.split(" or an ")) {
                    v.push(Filter::Subtype(subtype_word(t.trim())?));
                }
                (a, Some(Filter::Or(v)))
            }
            None => (s, None),
        };
        let (f, plural, tail) = parse_object_phrase(s)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        let f = match types {
            Some(t) => Filter::and(vec![f, t]),
            None => f,
        };
        (f, rest)
    } else {
        let (s, rest) = l.split_once(" enter with ")?;
        let f = subject_list(s)?;
        (f, rest)
    };
    if matches!(subj, Filter::Source) {
        return None;
    }
    // "a number of additional +1/+1 counters on it equal to ~'s toughness"
    if let Some(r) = rest.strip_prefix("a number of additional ") {
        let (kind, r) = crate::oracle::costs::counter_kind(r)?;
        let r = strip(r, "counters")?;
        let r = r
            .strip_prefix("on it")
            .or_else(|| r.strip_prefix("on them"))?;
        let n = match end(r) {
            "equal to ~'s power" => Value::PowerOf(Box::new(Sel::This)),
            "equal to ~'s toughness" => Value::ToughnessOf(Box::new(Sel::This)),
            _ => return None,
        };
        return others_counters_ability(subj, kind, n, zone, text);
    }
    // "an additional +1/+1 counter", "two additional +1/+1 counters"
    let rest = match rest.strip_prefix("an additional ") {
        Some(r) => r,
        None if rest.contains(" additional ") => rest,
        None => return None,
    };
    // "+1/+1 counter on it", "two +1/+1 counters on them for each ..."
    let (n, r) = match parse_number(rest) {
        Some((n, r)) if !rest.starts_with('+') && !rest.starts_with('-') => (n, r),
        _ => (Value::c(1), rest),
    };
    let r = r
        .trim_start()
        .strip_prefix("additional ")
        .unwrap_or(r.trim_start());
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    let r = r
        .strip_prefix("on it")
        .or_else(|| r.strip_prefix("on them"))?
        .trim();
    let n = if r.is_empty() {
        n
    } else if let Some(x) = r.strip_prefix(", where x is ") {
        // "where X is the number of +1/+1 counters on ~" (~ is this effect's source)
        if !matches!(n, Value::X) {
            return None;
        }
        let x = end(x);
        let (k, on) = x
            .strip_prefix("the number of ")?
            .split_once(" counters on ")?;
        if on != "~" || k.contains(' ') {
            return None;
        }
        Value::CountersOn(Box::new(Sel::This), Some(k.into()))
    } else {
        let each = r.strip_prefix("for each ")?;
        let v = match each {
            "creature that died under your control this turn" => {
                Value::Custom("creatures_you_controlled_died_this_turn".into())
            }
            _ => {
                let (f, _, tail) = parse_object_phrase(each)?;
                if !end(tail).is_empty() {
                    return None;
                }
                Value::Count(f)
            }
        };
        match n {
            Value::Const(1) => v,
            other => Value::Mul(Box::new(other), Box::new(v)),
        }
    };
    others_counters_ability(subj, kind, n, zone, text)
}

fn others_counters_ability(
    subj: Filter,
    kind: crate::types::CounterKind,
    n: Value,
    zone: FunctionZone,
    text: &str,
) -> Option<Vec<Ability>> {
    let def = ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(entering_filter(subj)),
        action: ReplacementAction::EnterWithCounters(kind, n),
        self_replacement: false,
        optional: false,
    };
    let mut st = StaticAbility::new(StaticEffect::Replacement(def));
    st.zone = zone;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

/// "As long as ~ is in your graveyard, each Human creature you control enters with an
/// additional +1/+1 counter on it." (The core would read "as long as" as a condition.)
fn graveyard_others_enter_with_counters(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    if !lower.starts_with("as long as ~ is in your graveyard, ") {
        return None;
    }
    others_enter_with_counters(&lower, block, ctx)
}

/// "You may have ~ enter as a copy of any creature on the battlefield." (CR 707.9,
/// 614.1c); "You may have ~ enter tapped as a copy of any land on the battlefield."
fn enter_as_copy(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("you may have ~ enter ")?;
    let (tapped, r) = match r.strip_prefix("tapped ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let r = r.strip_prefix("as a copy of ")?;
    let (r, exceptions) = match r.split_once(", except ") {
        Some((a, _)) => {
            // Take the exception text from the original block (same offsets: the block
            // is ASCII here) to keep quoted abilities' capitalization.
            if !block.is_ascii() {
                return None;
            }
            let start = l.find(", except ")? + ", except ".len();
            (a, copy_exceptions(end(&block[start..]), ctx)?)
        }
        None => (r, vec![]),
    };
    let r = r
        .strip_prefix("any ")
        .or_else(|| r.strip_prefix("a "))
        .or_else(|| r.strip_prefix("an "))?;
    let r = r.strip_suffix(" on the battlefield").unwrap_or(r);
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    // "Enter tapped as a copy" is one effect; its "tapped" part is applied first (as a
    // self-replacement, CR 616.1a) so it isn't lost when the permanent becomes a copy
    // and loses this ability.
    let rep = |action, self_replacement| {
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
                ReplacementDef {
                    event: ReplacementEvent::EntersBattlefield(Filter::Source),
                    action,
                    self_replacement,
                    optional: false,
                },
            ))),
            block,
        )
    };
    let mut out = vec![rep(
        ReplacementAction::EnterAsCopy {
            filter: f,
            optional: true,
        },
        false,
    )];
    if tapped {
        out.push(rep(ReplacementAction::EnterTapped, true));
    }
    if !exceptions.is_empty() {
        out.push(rep(
            ReplacementAction::AsEnters(Box::new(Effect::EnterCopyExceptions(exceptions))),
            true,
        ));
    }
    Some(out)
}

/// Copy exceptions (CR 707.9b): "it's a Shapeshifter Rogue in addition to its other
/// types", "it's an artifact in addition to its other types", "it has \"[ability]\"",
/// "it isn't legendary", joined by "and".
fn copy_exceptions(s: &str, ctx: &CompileContext) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    let mut rest = s.trim();
    while !rest.is_empty() {
        let lower = rest.to_lowercase();
        if let Some(r) = lower.strip_prefix("it has \"") {
            let close = r.find('"')?;
            let inner = &rest["it has \"".len().."it has \"".len() + close];
            for a in crate::oracle::parse_ability(inner, ctx)? {
                if matches!(a.kind, AbilityKind::Unsupported(_)) {
                    return None;
                }
                out.push(Modification::AddAbility(a));
            }
            rest = &rest["it has \"".len() + close + 1..];
        } else if let Some(r) = lower.strip_prefix("it isn't legendary") {
            out.push(Modification::RemoveSupertypes(vec![
                crate::types::Supertype::Legendary,
            ]));
            rest = &rest[rest.len() - r.len()..];
        } else if let Some(r) = lower
            .strip_prefix("it's an ")
            .or_else(|| lower.strip_prefix("it's a "))
        {
            let (types, after) = r.split_once(" in addition to its other types")?;
            let mut card_types = Vec::new();
            let mut subtypes = Vec::new();
            for w in types.split_whitespace() {
                if let Some(t) = crate::types::CardType::from_word(w) {
                    card_types.push(t);
                } else {
                    subtypes.push(subtype_word(w)?);
                }
            }
            if !card_types.is_empty() {
                out.push(Modification::AddTypes(card_types));
            }
            if !subtypes.is_empty() {
                out.push(Modification::AddSubtypes(subtypes));
            }
            rest = &rest[rest.len() - after.len()..];
        } else {
            return None;
        }
        let t = rest.trim_start();
        rest = t
            .strip_prefix(", and ")
            .or_else(|| t.strip_prefix("and "))
            .or_else(|| t.strip_prefix(", "))
            .unwrap_or(t)
            .trim();
        if rest == "." {
            break;
        }
    }
    (!out.is_empty()).then_some(out)
}

// ---------------------------------------------------------------------------
// Day and night (CR 731)
// ---------------------------------------------------------------------------

/// "If it's neither day nor night, it becomes day as ~ enters." — a replacement effect
/// applied as the permanent enters (CR 614.1c, 731.1).
fn day_as_enters(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("if ")?;
    let (c, e) = r.split_once(", ")?;
    let e = e.strip_suffix(" as ~ enters")?;
    let cond = crate::oracle::statics::parse_condition(c, ctx)?;
    let eff = day_night_effect_inner(e)?;
    let def = ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::If {
            cond,
            then: Box::new(eff),
            otherwise: Box::new(Effect::Noop),
        })),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        block,
    )])
}

fn day_night_effect_inner(l: &str) -> Option<Effect> {
    match end(l) {
        "it becomes day" => Some(Effect::SetDayNight { day: true }),
        "it becomes night" => Some(Effect::SetDayNight { day: false }),
        _ => None,
    }
}

fn day_night_effect(l: &str, _b: &mut Builder) -> Option<Effect> {
    day_night_effect_inner(l)
}

fn day_night_condition(c: &str) -> Option<Condition> {
    match end(c) {
        "it's neither day nor night" => Some(Condition::And(vec![
            Condition::Not(Box::new(Condition::IsDay)),
            Condition::Not(Box::new(Condition::IsNight)),
        ])),
        _ => None,
    }
}

/// "Whenever day becomes night or night becomes day" (CR 731.1a).
fn day_night_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "day becomes night or night becomes day").then_some((
        TriggerCond::DayNightChanges,
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! {
    AbilityPattern { name: "others enter tapped", priority: 50, parse: others_enter_tapped }
}
inventory::submit! {
    AbilityPattern { name: "enter as a copy", priority: 50, parse: enter_as_copy }
}
inventory::submit! {
    StaticPattern { name: "others enter with additional counters", priority: 100, parse: others_enter_with_counters }
}
inventory::submit! {
    AbilityPattern { name: "others enter with counters (from the graveyard)", priority: 50, parse: graveyard_others_enter_with_counters }
}
inventory::submit! {
    AbilityPattern { name: "it becomes day as ~ enters", priority: 50, parse: day_as_enters }
}
inventory::submit! {
    EffectPattern { name: "it becomes day/night", priority: 100, parse: day_night_effect }
}
inventory::submit! {
    ConditionPattern { name: "neither day nor night", priority: 100, parse: day_night_condition }
}
inventory::submit! {
    TriggerPattern { name: "day becomes night or night becomes day", priority: 100, parse: day_night_trigger }
}
