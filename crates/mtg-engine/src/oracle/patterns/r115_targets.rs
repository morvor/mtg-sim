//! Oracle patterns for changing targets (CR 115.7) and for spells and abilities described
//! by their targets (CR 115.9): "change the target of target spell with a single target",
//! "change a target of target spell or ability to this creature", "change any targets of
//! target Arcane spell", "choose new targets for target spell". Also target phrases with
//! a number of targets: "two target creatures each get ..." (CR 115.3) and "deals N damage
//! divided as you choose among one, two, or three targets" (CR 115.4, 601.2d).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase, parse_target};

/// "a player", "you", "~", "a single creature", "a creature you control" in "that targets
/// [only] ...".
fn targeted_thing(s: &str) -> Option<(Option<Filter>, Option<PlayerFilter>)> {
    let s = end(s);
    Some(match s {
        "a player" | "a single player" => (None, Some(PlayerFilter::Any)),
        "you" => (None, Some(PlayerFilter::You)),
        "an opponent" => (None, Some(PlayerFilter::Opponent)),
        "~" => (Some(Filter::Source), None),
        _ if s.starts_with("you or ") => {
            let (objects, _) = targeted_thing(&s["you or ".len()..])?;
            (objects, Some(PlayerFilter::You))
        }
        _ => {
            let s = s.strip_prefix("a single ").unwrap_or(s);
            let s = s
                .strip_prefix("a ")
                .or_else(|| s.strip_prefix("an "))
                .unwrap_or(s);
            let (f, _, rest) = parse_object_phrase(s)?;
            if !end(rest).is_empty() {
                return None;
            }
            (Some(f), None)
        }
    })
}

/// Splits a qualifier describing a stack object's targets off a target phrase: "with a
/// single target", "that targets only [...]", "that targets [...]" (CR 115.9).
fn split_targets_qualifier(s: &str) -> (&str, Option<TargetsFilter>) {
    if let Some((head, _)) = s.split_once(" with a single target") {
        return (head, Some(TargetsFilter::Count(1)));
    }
    if let Some((head, what)) = s.split_once(" that targets only ") {
        if let Some((objects, players)) = targeted_thing(what) {
            return (head, Some(TargetsFilter::Only { objects, players }));
        }
    }
    if let Some((head, what)) = s.split_once(" that targets ") {
        if let Some((objects, players)) = targeted_thing(what) {
            return (head, Some(TargetsFilter::Targets { objects, players }));
        }
    }
    (s, None)
}

/// "target spell [or ability] [with a single target | that targets only ...]".
fn stack_target(s: &str, b: &mut Builder) -> Option<Sel> {
    let (head, qualifier) = split_targets_qualifier(end(s));
    let (mut spec, rest) = parse_target(head)?;
    if !end(rest).is_empty() {
        return None;
    }
    let add = |f: &mut Filter| {
        if let Some(q) = &qualifier {
            *f = Filter::and(vec![f.clone(), Filter::StackTargets(Box::new(q.clone()))]);
        }
    };
    match &mut spec.what {
        TargetKind::Spell(f) | TargetKind::Ability(f) | TargetKind::SpellOrAbility(f) => add(f),
        _ => return None,
    }
    let slot = b.add_target(spec, head);
    Some(Sel::Target(slot))
}

/// "change the target of X", "change the targets of X", "change a target of X", "change
/// any targets of X", "choose new targets for X", each optionally ending "to ~".
fn change_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let heads: [(&str, TargetChange); 6] = [
        ("change the target of ", TargetChange::All),
        ("change the targets of ", TargetChange::All),
        ("change the target or targets of ", TargetChange::All),
        ("change a target of ", TargetChange::One),
        ("change any targets of ", TargetChange::Any),
        ("choose new targets for ", TargetChange::ChooseNew),
    ];
    let (how, rest) = heads
        .iter()
        .find_map(|(h, how)| l.strip_prefix(h).map(|r| (*how, r)))?;
    let (what, to) = match rest.strip_suffix(" to ~") {
        Some(r) => (r, Some(Sel::This)),
        None => (rest, None),
    };
    let what = stack_target(what, b)?;
    Some(Effect::ChangeTargets {
        what,
        who: PlayerRef::You,
        how,
        to,
    })
}

/// "two target creatures [you control] each get +2/+2 [and gain flying] until end of
/// turn": one instance of "target" with two targets, which must be different (CR 115.3).
fn two_targets_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let (subject, predicate) = l.split_once(" each ")?;
    let rest = subject.strip_prefix("two target ")?;
    let singular_subject = format!("target {}", rest.replacen("creatures", "creature", 1));
    let mut words = predicate.splitn(2, ' ');
    let verb = words.next()?;
    let tail = words.next().unwrap_or("");
    let verb = match verb {
        "get" => "gets",
        "gain" => "gains",
        "have" => "has",
        _ => return None,
    };
    let tail = tail.replace(" and gain ", " and gains ");
    let before = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&format!("{singular_subject} {verb} {tail}"), b)?;
    if b.targets.len() != before + 1 {
        return None;
    }
    let spec = b.targets.last_mut()?;
    spec.min = 2;
    spec.max = Value::Const(2);
    spec.text = subject.to_string();
    Some(e)
}

/// "[~ / it] deals N damage divided as you choose among one or two targets", "... one,
/// two, or three target creatures", "... up to N targets" (CR 601.2d).
fn divided_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let (source, rest) = if let Some(r) = l.strip_prefix("~ deals ") {
        (Sel::This, r)
    } else if let Some(r) = l
        .strip_prefix("it deals ")
        .or_else(|| l.strip_prefix("he deals "))
        .or_else(|| l.strip_prefix("she deals "))
    {
        (b.it.clone(), r)
    } else {
        return None;
    };
    let (amount, rest) = parse_number(rest)?;
    let rest = rest
        .trim_start()
        .strip_prefix("damage divided as you choose among ")?;
    let (max, rest) = if let Some(r) = rest.strip_prefix("one or two ") {
        (2, r)
    } else if let Some(r) = rest.strip_prefix("one, two, or three ") {
        (3, r)
    } else if let Some(r) = rest.strip_prefix("up to ") {
        match parse_number(r)? {
            (Value::Const(n), r) => (n, r.trim_start()),
            _ => return None,
        }
    } else {
        return None;
    };
    let rest = end(rest);
    let mut spec = if rest == "targets" {
        TargetSpec::any_target()
    } else {
        let r = rest.strip_prefix("target ")?;
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        TargetSpec::object(f, rest)
    };
    spec.min = 1;
    spec.max = Value::Const(max);
    spec.divide = Some(amount);
    let slot = b.add_target(spec, "targets (divided)");
    Some(Effect::DealDividedDamage { source, slot })
}

/// "counter target spell [or ability] that targets [...]" (CR 115.9b).
fn counter_spell_that_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let rest = l.strip_prefix("counter ")?;
    if !rest.contains(" that targets ") {
        return None;
    }
    let what = stack_target(rest, b)?;
    Some(Effect::CounterSpell { what })
}

inventory::submit! { EffectPattern { name: "change targets", priority: 0, parse: change_targets } }
inventory::submit! { EffectPattern { name: "counter spell that targets", priority: 0, parse: counter_spell_that_targets } }
inventory::submit! { EffectPattern { name: "two targets each", priority: 0, parse: two_targets_each } }
inventory::submit! { EffectPattern { name: "divided damage among targets", priority: 0, parse: divided_damage } }
