//! The source's base power (and toughness) becoming a value (layer 7b, CR 613.4b):
//! "you may have ~'s base power and toughness become 4/2 until end of turn" (Mirkwood
//! Meditator), "... become 4/1 or 1/4 until end of turn" (Master of Winds: one of the two,
//! chosen as the effect is created), "you may have ~'s base power become equal to that
//! creature's power until end of turn" (Belligerent Yearling), "... base power and
//! toughness become equal to that creature's power and toughness ..." (Eldrazi Mimic),
//! and "~'s base power becomes equal to target creature's power." (Riptide Mangler, which
//! lasts indefinitely), "until end of turn, ~'s base power becomes equal to the number of
//! Towns you control" (PuPu UFO) and "... become 1 plus the greatest power among other
//! creatures you control ..." (Arni Brokenbrow). Values are determined once, as the
//! effect begins (CR 608.2h).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;

fn modify(mods: Vec<Modification>, duration: Duration) -> Effect {
    Effect::Modify {
        what: Sel::This,
        mods,
        duration,
    }
}

/// "N/M" as constant values (CR 107.1b allows negatives).
fn const_pt(s: &str) -> Option<(Value, Value)> {
    let (p, t) = s.trim().split_once('/')?;
    Some((Value::c(p.parse().ok()?), Value::c(t.parse().ok()?)))
}

/// "[that creature|target creature]'s [what]" → the object whose characteristics are
/// used. (The target is added only once the whole phrase matched.)
fn object_possessive(s: &str, what: &str, b: &mut Builder) -> Option<Sel> {
    if let Some(r) = s.strip_prefix("that creature's ") {
        if r != what || !b.in_trigger || !matches!(b.it, Sel::TriggerObject) {
            return None;
        }
        return Some(Sel::TriggerObject);
    }
    if s.strip_prefix("target creature's ")? != what {
        return None;
    }
    let slot = b.add_target(
        TargetSpec::object(Filter::creature(), "target creature"),
        "target creature",
    );
    Some(Sel::Target(slot))
}

/// A power value: "equal to [value]" or "N plus [value]" (Arni Brokenbrow).
fn power_value(s: &str, b: &mut Builder) -> Option<Value> {
    let (n, r) = match s.split_once(" plus ") {
        Some((n, r)) => (Some(n.parse::<i32>().ok()?), r),
        None => (None, s.strip_prefix("equal to ")?),
    };
    let (v, rest) = parse_value_phrase(r, b)?;
    if !end(&rest).trim().is_empty() {
        return None;
    }
    Some(match n {
        Some(n) => Value::Sum(vec![Value::c(n), v]),
        None => v,
    })
}

fn base_pt_becomes(l: &str, b: &mut Builder) -> Option<Effect> {
    let s = end(l);
    let (s, have) = match s.strip_prefix("have ") {
        Some(r) => (r, true),
        None => (s, false),
    };
    let s = s.strip_prefix("~'s base ")?;
    let (both, s) = if let Some(r) = s.strip_prefix("power and toughness ") {
        (true, r)
    } else {
        (false, s.strip_prefix("power ")?)
    };
    let verb = match (have, both) {
        (true, _) => "become ",
        (false, true) => "become ",
        (false, false) => "becomes ",
    };
    let s = s.strip_prefix(verb)?;
    let (s, duration) = match s.strip_suffix(" until end of turn") {
        Some(r) => (r, Duration::EndOfTurn),
        None => (s, Duration::Permanent),
    };
    let what = if both { "power and toughness" } else { "power" };
    if let Some(sel) = s
        .strip_prefix("equal to ")
        .and_then(|r| object_possessive(r, what, b))
    {
        let p = Value::PowerOf(Box::new(sel.clone()));
        let t = both.then(|| Value::ToughnessOf(Box::new(sel)));
        return Some(modify(vec![Modification::SetPT(Some(p), t)], duration));
    }
    if !both {
        let v = power_value(s, b)?;
        return Some(modify(vec![Modification::SetPT(Some(v), None)], duration));
    }
    if matches!(duration, Duration::Permanent) {
        return None;
    }
    if let Some((a, c)) = s.split_once(" or ") {
        // One of the two, chosen by the controller as the effect is created.
        let options = [a, c]
            .into_iter()
            .map(|x| {
                let (p, t) = const_pt(x)?;
                Some((
                    x.trim().to_string(),
                    modify(
                        vec![Modification::SetPT(Some(p), Some(t))],
                        duration.clone(),
                    ),
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        return Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        });
    }
    let (p, t) = const_pt(s)?;
    Some(modify(
        vec![Modification::SetPT(Some(p), Some(t))],
        duration,
    ))
}

inventory::submit! { EffectPattern { name: "~'s base power [and toughness] become(s) ...", priority: 100, parse: base_pt_becomes } }
