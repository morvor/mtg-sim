//! Temporary combat restrictions and requirements created by resolving spells and
//! abilities (CR 508.1c–d, 509.1b–c, 611.2c): "Target creature attacks this turn if
//! able.", "Target creature blocks ~ this turn if able.", "Target creature can't block ~
//! this turn.", "All creatures able to block target creature this turn do so.", "~ can't
//! be blocked by creatures with power 2 or less this turn.", and the same after a
//! pump: "Target creature gets +2/+2 until end of turn and can't be blocked this turn."

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, is_class_filter, object_ref, parse_simple, Builder};
use crate::oracle::patterns::statics::restriction_predicate;
use crate::oracle::phrases::end;

/// The filter a rule-modifying effect uses for its subject: a class of objects stays a
/// class (it can affect objects that join it later, CR 611.2c); specific objects are
/// locked in as the effect begins. Other groups ("creatures target player controls")
/// aren't handled.
/// Whether `what`, parsed from the subject text `subject`, may be the source itself: a
/// pronoun with nothing else to refer to ("that token" after creating a token, "those
/// creatures" after a group) falls back to the source, which would put the restriction on
/// the wrong object. Only "~" and "it" (as in "When you do, it ...") name the source.
fn names_source_faithfully(subject: &str, what: &Sel) -> bool {
    let subject = subject.trim();
    !matches!(what, Sel::This)
        || subject.starts_with('~')
        || subject == "it"
        || subject.starts_with("it ")
}

fn subject_filter(what: &Sel) -> Option<Filter> {
    match what {
        Sel::All(f) if is_class_filter(f) => Some(f.clone()),
        Sel::All(_) | Sel::None | Sel::Players(_) => None,
        _ => Some(Filter::In(Box::new(what.clone()))),
    }
}

/// Restrictions whose object filters a resolving effect can lock onto the objects it
/// names (see `fix_restriction`/`lock_restriction_objects` in `resolve.rs`).
fn lockable(r: &Restriction) -> bool {
    matches!(
        r,
        Restriction::CantAttack(_)
            | Restriction::CantBlock(_)
            | Restriction::CantAttackOrBlock(_)
            | Restriction::CantBeBlocked(_)
            | Restriction::MustAttack(_)
            | Restriction::MustBlock(_)
            | Restriction::MustBeBlocked(_)
            | Restriction::MustBeBlockedByAll(_)
            | Restriction::MustBlockAttacker { .. }
            | Restriction::CantBeBlockedBy { .. }
            | Restriction::AttackDespiteDefender(_)
            | Restriction::DamageByToughness(_)
    )
}

/// The restrictions and duration of a predicate like "attacks this turn if able", "can't
/// block ~ this turn", "can't be blocked this turn except by creatures with haste".
fn predicate(rest: &str, f: &Filter) -> Option<(Vec<Restriction>, Duration)> {
    let rest = end(rest.trim());
    // Requirements name their duration before "if able".
    for (infix, dur) in [
        (" this turn if able", Duration::EndOfTurn),
        (" this combat if able", Duration::EndOfCombat),
    ] {
        let Some(verb) = rest.strip_suffix(infix) else {
            continue;
        };
        let r = match verb {
            "attacks" | "attack" => Restriction::MustAttack(f.clone()),
            "blocks" | "block" => Restriction::MustBlock(f.clone()),
            "must be blocked" => Restriction::MustBeBlocked(f.clone()),
            // CR 509.1c: a requirement that the creature block this one.
            "blocks ~" | "block ~" => Restriction::MustBlockAttacker {
                blocker: f.clone(),
                attacker: Filter::Source,
            },
            _ => return None,
        };
        return Some((vec![r], dur));
    }
    // "can't be blocked this turn except by creatures with haste".
    let moved;
    let rest = match rest.split_once(" this turn except by ") {
        Some((a, b)) => {
            moved = format!("{a} except by {b} this turn");
            moved.as_str()
        }
        None => rest,
    };
    let (dur, p) = duration_suffix(rest);
    if !matches!(
        dur,
        Duration::EndOfTurn | Duration::EndOfCombat | Duration::UntilYourNextTurn
    ) {
        return None;
    }
    let rs = restriction_predicate(p, f)?;
    if rs.is_empty() || !rs.iter().all(lockable) {
        return None;
    }
    Some((rs, dur))
}

fn add(rs: Vec<Restriction>, dur: Duration) -> Vec<Effect> {
    rs.into_iter()
        .map(|restriction| Effect::AddRestriction {
            restriction,
            duration: dur.clone(),
        })
        .collect()
}

/// "[objects] [restriction/requirement] [duration]", and "all creatures able to block
/// [object] [this turn] do so".
fn temporary_restriction(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    if let Some(r) = l.strip_prefix("all creatures able to block ") {
        let (r, dur) = if let Some(r) = r.strip_suffix(" this turn do so") {
            (r, Duration::EndOfTurn)
        } else if let Some(r) = r.strip_suffix(" this combat do so") {
            (r, Duration::EndOfCombat)
        } else {
            return None;
        };
        let (what, tail) = object_ref(r, b)?;
        if !end(&tail).is_empty() || !names_source_faithfully(r, &what) {
            return None;
        }
        // CR 509.1c: a requirement on each creature able to block it.
        let f = subject_filter(&what)?;
        return Some(Effect::seq(add(
            vec![Restriction::MustBeBlockedByAll(f)],
            dur,
        )));
    }
    // "[pump] until end of turn and [restriction]".
    if let Some((first, second)) = l.split_once(" until end of turn and ") {
        let modify = parse_simple(&format!("{first} until end of turn"), b)?;
        let Effect::Modify { what, .. } = &modify else {
            return None;
        };
        if !names_source_faithfully(first, what) {
            return None;
        }
        let f = subject_filter(what)?;
        let (rs, dur) = predicate(second, &f)?;
        let mut v = vec![modify];
        v.extend(add(rs, dur));
        return Some(Effect::seq(v));
    }
    let (what, rest) = object_ref(l, b)?;
    let subject = l.trim().strip_suffix(rest.as_str()).unwrap_or(l);
    if !names_source_faithfully(subject, &what) {
        return None;
    }
    let f = subject_filter(&what)?;
    let (rs, dur) = predicate(&rest, &f)?;
    Some(Effect::seq(add(rs, dur)))
}

inventory::submit! { EffectPattern { name: "restrictions: temporary combat restrictions", priority: 100, parse: temporary_restriction } }

/// "Creatures your opponents control get -1/-1 until end of turn. Those creatures attack
/// this turn if able.": the pronoun names the objects the previous sentence changed, fixed
/// as the effect begins (CR 611.2c).
fn f_those_creatures_restriction(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    fn affected(e: &Effect) -> Option<Sel> {
        match e {
            Effect::Modify {
                what: w @ Sel::All(_),
                ..
            } => Some(w.clone()),
            Effect::Seq(v) => v.last().and_then(affected),
            _ => None,
        }
    }
    let Some(rest) = ["those creatures ", "they "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    let Some(what) = affected(prev) else {
        return false;
    };
    let Some((rs, dur)) = predicate(rest, &Filter::In(Box::new(what))) else {
        return false;
    };
    let old = std::mem::take(prev);
    let mut v = vec![old];
    v.extend(add(rs, dur));
    *prev = Effect::seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "restrictions: those creatures [restriction]", priority: 0, apply: f_those_creatures_restriction } }
