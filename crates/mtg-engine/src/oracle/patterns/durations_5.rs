//! Durations, grammar V (cards4-triggers-durations): durations ending at a step of a later
//! turn and "for as long as" conditions about the objects the instruction names (CR 611.2a,
//! 611.2b).
//!
//! - "Target land becomes a Swamp until its controller's next untap step." (Orcish Farmer)
//! - "~'s base power and toughness become equal to the power and toughness of target
//!   creature other than ~ until the end of your next upkeep." (Halfdane)
//! - "~ becomes a copy of that creature for as long as that creature remains tapped."
//!   (Zygon Infiltrator)
//! - "Gain control of target creature with power less than or equal to ~'s power for as
//!   long as ~ remains tapped and that creature's power remains less than or equal to ~'s
//!   power." (Old Man of the Sea)
//!
//! Each instruction is read as the core grammar reads it with "until end of turn", and the
//! duration of the one continuous effect it creates is replaced.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Replaces the `EndOfTurn` duration of the one continuous effect `e` creates with `d`.
/// `allow` says which effects may take the new duration. Returns how many were replaced.
fn swap_duration(e: &mut Effect, d: &Duration, allow: fn(&Effect) -> bool) -> usize {
    match e {
        Effect::Seq(v) => v.iter_mut().map(|x| swap_duration(x, d, allow)).sum(),
        // "you may have that creature become a copy of ..." (Blade of Shared Souls).
        Effect::May { effect, .. } => swap_duration(effect, d, allow),
        _ if !allow(e) => 0,
        Effect::Modify { duration, .. }
        | Effect::BecomeCopy { duration, .. }
        | Effect::BecomeCopyExcept { duration, .. }
        | Effect::GainControl { duration, .. }
            if matches!(duration, Duration::EndOfTurn) =>
        {
            *duration = d.clone();
            1
        }
        _ => 0,
    }
}

/// Effects that apply to particular objects, fixed as they begin (CR 611.2c).
fn object_effect(e: &Effect) -> bool {
    let what = match e {
        Effect::Modify { what, .. }
        | Effect::BecomeCopy { what, .. }
        | Effect::BecomeCopyExcept { what, .. }
        | Effect::GainControl { what, .. } => what,
        _ => return false,
    };
    !matches!(what, Sel::None | Sel::All(_))
}

/// `head` read with "until end of turn", with the duration replaced by `d`.
fn retimed(head: &str, d: Duration, b: &mut Builder, allow: fn(&Effect) -> bool) -> Option<Effect> {
    let saved = (b.targets.len(), b.it.clone());
    let mut e = parse_sentence(&format!("{head} until end of turn"), b)?;
    if swap_duration(&mut e, &d, allow) != 1 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(e)
}

/// "[instruction] until its controller's next untap step", "[instruction] until the end of
/// your next upkeep".
fn until_later_step(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let steps = [
        ("untap step", TriggerStep::Untap),
        ("upkeep", TriggerStep::Upkeep),
        ("draw step", TriggerStep::Draw),
        ("end step", TriggerStep::End),
    ];
    for (name, step) in steps {
        if let Some(head) = l.strip_suffix(&format!(" until its controller's next {name}") as &str)
        {
            return retimed(
                head,
                Duration::UntilItsControllersNextStep(step),
                b,
                object_effect,
            );
        }
        if let Some(head) = l.strip_suffix(&format!(" until the end of your next {name}") as &str) {
            return retimed(head, Duration::UntilEndOfYourNextStep(step), b, object_effect);
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "durations 5: until its controller's next step, until the end of your next step", priority: 90, parse: until_later_step } }

/// One condition of "for as long as [condition] and [condition]", with `it` the object the
/// instruction named ("that creature").
fn remains(c: &str, it: &Sel) -> Option<Condition> {
    let named = |s: &str| -> Option<Sel> {
        match s {
            "~" => Some(Sel::This),
            "that creature" | "that permanent" | "it" if !matches!(it, Sel::None | Sel::This) => {
                Some(it.clone())
            }
            _ => None,
        }
    };
    // "~ remains attached to it" (Assimilation Aegis).
    if let Some((s, o)) = c.split_once(" remains attached to ") {
        return Some(Condition::SelMatches(
            named(s)?,
            Filter::AttachedTo(Box::new(named(o)?)),
        ));
    }
    if let Some(s) = c.strip_suffix(" remains tapped") {
        return Some(match named(s)? {
            Sel::This => match crate::untap_choice::remains_tapped(false) {
                Duration::WhileCondition(c) => c,
                _ => return None,
            },
            s => Condition::SelMatches(s, Filter::Tapped),
        });
    }
    // "that creature's power remains less than or equal to ~'s power".
    let (subject, r) = c.split_once("'s power remains ")?;
    let who = named(subject)?;
    let (cmp, other) = [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
    ]
    .into_iter()
    .find_map(|(p, cmp)| r.strip_prefix(p).map(|o| (cmp, o)))?;
    let other = other.strip_suffix("'s power")?;
    let other = named(other)?;
    Some(Condition::Compare(
        Value::PowerOf(Box::new(who)),
        cmp,
        Value::PowerOf(Box::new(other)),
    ))
}

/// "[instruction] for as long as that creature remains tapped", "... for as long as ~
/// remains tapped and that creature's power remains less than or equal to ~'s power": a
/// condition about the objects the instruction names, fixed as the effect begins
/// (`duration_refs`).
fn for_as_long_as_it_remains(l: &str, b: &mut Builder) -> Option<Effect> {
    // "For as long as ~ remains attached to it, that creature becomes a copy of ..."
    // (Assimilation Aegis): the duration written first.
    let moved;
    let l = match end(l).strip_prefix("for as long as ") {
        Some(r) => {
            let (cond, rest) = r.split_once(", ")?;
            moved = format!("{rest} for as long as {cond}");
            moved.as_str()
        }
        None => l,
    };
    let (head, cond) = end(l).rsplit_once(" for as long as ")?;
    if cond.contains(", ") {
        return None;
    }
    // Durations the core grammar reads alone go there.
    let parts: Vec<&str> = cond.split(" and ").collect();
    let names_it = parts.iter().any(|p| {
        p.starts_with("that ") || p.starts_with("it ") || p.ends_with(" attached to it")
    });
    if !names_it {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = parse_sentence(&format!("{head} until end of turn"), b)?;
    // "that creature": what an earlier sentence named ("Tap another target creature ...
    // ~ becomes a copy of that creature ..."), else what this one names.
    let it = match &saved.1 {
        Sel::None | Sel::This => b.it.clone(),
        s => s.clone(),
    };
    let conds: Option<Vec<Condition>> = parts.iter().map(|p| remains(p, &it)).collect();
    let d = match conds {
        Some(mut v) if v.len() == 1 => Duration::WhileCondition(v.remove(0)),
        Some(v) => Duration::WhileCondition(Condition::And(v)),
        None => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
    };
    if swap_duration(&mut e, &d, object_effect) != 1 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "durations 5: for as long as that creature remains", priority: 90, parse: for_as_long_as_it_remains } }

/// "~'s base power and toughness become equal to the power and toughness of target
/// creature other than ~ until end of turn" (Halfdane): the values of that creature as the
/// effect begins (CR 608.2h), in layer 7b (CR 613.4b).
fn base_pt_equal_to_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, r) = l.split_once("'s base power and toughness become equal to the power and toughness of ")?;
    let r = r.strip_suffix(" until end of turn")?;
    let what = match subject {
        "~" => Sel::This,
        _ => return None,
    };
    let (of, tail) = crate::oracle::effects::object_ref(r, b)?;
    if !end(&tail).is_empty() || matches!(of, Sel::None | Sel::This | Sel::All(_)) {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::SetPT(
            Some(Value::PowerOf(Box::new(of.clone()))),
            Some(Value::ToughnessOf(Box::new(of))),
        )],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "durations 5: ~'s base power and toughness become equal to another's", priority: 100, parse: base_pt_equal_to_other } }
