//! Who puts counters (CR 122.6, 122.6a) and whether an effect does (CR 609.1):
//!
//! * triggers: "Whenever you put one or more [kind] counters on [object]" (once for each
//!   put action on each permanent), "Whenever you put a [kind] counter on [object]" (once
//!   for each counter), "Whenever you put one or more counters on a permanent or player",
//!   "Whenever an opponent puts …" ([`TriggerCond::CountersPutBy`]). Counters count however
//!   the player put them: by an effect, as a cost, as the result of damage (wither,
//!   infect, toxic), or as a permanent entered under their control. "It"/"that creature"
//!   is the permanent; "that player" is the player who put them; "that much"/"that many"
//!   is how many.
//! * replacements: "If an effect would put one or more counters on a permanent you
//!   control, it puts twice that many of those counters on that permanent instead"
//!   (Doubling Season; not counters put as a cost, as the result of damage, or by a
//!   turn-based action), "If you would put one or more counters on a permanent you
//!   control, put that many plus one of each of those kinds of counters on that permanent
//!   instead" ([`ReplacementEvent::PutCountersMatching`]).

use super::{StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::patterns::triggers::parse_subject;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::types::CounterKind;

/// "you put ", "an opponent puts ", "a player puts ".
fn putter(r: &str) -> Option<(PlayerRel, &str)> {
    for (p, rel) in [
        ("you put ", PlayerRel::You),
        ("an opponent puts ", PlayerRel::Opponent),
        ("a player puts ", PlayerRel::Any),
        ("another player puts ", PlayerRel::NotYou),
    ] {
        if let Some(r) = r.strip_prefix(p) {
            return Some((rel, r));
        }
    }
    None
}

/// "one or more [kind] counters on " / "a [kind] counter on " (`each`) / "one or more
/// counters on " / "a counter on ": (kind, each, rest).
fn counters_on(r: &str) -> Option<(Option<CounterKind>, bool, &str)> {
    let (x, each) = if let Some(x) = r.strip_prefix("one or more ") {
        (x, false)
    } else {
        (
            r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?,
            true,
        )
    };
    let noun = if each { "counter on " } else { "counters on " };
    if let Some(rest) = x.strip_prefix(noun) {
        return Some((None, each, rest));
    }
    let (kind, rest) = crate::oracle::costs::counter_kind(x)?;
    let rest = rest.trim_start().strip_prefix(noun)?;
    Some((Some(kind), each, rest))
}

fn put_by_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let (who, r) = putter(end(r))?;
    let (kind, each, rest) = counters_on(r)?;
    let (on_objects, on_players, it) = if rest == "a permanent or player" {
        (Some(Filter::Permanent), Some(PlayerFilter::Any), Sel::TriggerObject)
    } else {
        let subj = parse_subject(rest)?;
        // "one or more other Heroes you control": several permanents in one event.
        if subj.one_or_more {
            return None;
        }
        let it = if subj.self_only {
            Sel::This
        } else {
            Sel::TriggerObject
        };
        (Some(subj.filter), None, it)
    };
    Some((
        TriggerCond::CountersPutBy {
            who,
            on_objects,
            on_players,
            kind,
            each,
        },
        it,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "r122 [player] puts counters on [object]", priority: 100, parse: put_by_trigger } }

/// "if you put a counter on ~ this turn", "as long as you've put one or more +1/+1
/// counters on ~ this turn", "if a counter was put on ~ this turn": such an event happened
/// this turn to this object (CR 400.7: not to an earlier object it was).
fn put_on_this_this_turn(c: &str) -> Option<Condition> {
    let c = end(c).strip_suffix(" on ~ this turn")?;
    let trigger = if let Some(r) = c
        .strip_prefix("you've put ")
        .or_else(|| c.strip_prefix("you put "))
    {
        let (kind, each, _) = counters_on(&format!("{r} on "))?;
        TriggerCond::CountersPutBy {
            who: PlayerRel::You,
            on_objects: Some(Filter::Source),
            on_players: None,
            kind,
            each,
        }
    } else {
        let r = match c.strip_suffix(" counters were put") {
            Some(r) => r.strip_prefix("one or more")?,
            None => {
                let r = c.strip_suffix(" counter was put")?;
                r.strip_prefix("an").or_else(|| r.strip_prefix("a"))?
            }
        };
        let kind = match r.trim() {
            "" => None,
            k => Some(
                crate::oracle::costs::counter_kind(k)
                    .filter(|(_, t)| t.trim().is_empty())?
                    .0,
            ),
        };
        TriggerCond::CountersPut {
            filter: Filter::Source,
            kind,
            each: false,
        }
    };
    Some(Condition::AllTriggerConditionsThisTurn(vec![trigger]))
}

inventory::submit! { super::ConditionPattern { name: "r122 (you put) counters (were put) on ~ this turn", priority: 100, parse: put_on_this_this_turn } }

/// "a permanent you control", "a creature or planeswalker you control or on yourself",
/// "a permanent or player": (objects, players).
fn recipients(s: &str) -> Option<(Option<Filter>, Option<PlayerFilter>)> {
    if s == "a permanent or player" {
        return Some((Some(Filter::Permanent), Some(PlayerFilter::Any)));
    }
    let (s, yourself) = match s.strip_suffix(" or on yourself") {
        Some(s) => (s, true),
        None => (s, false),
    };
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(s)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some((Some(f), yourself.then_some(PlayerFilter::You)))
}

/// "twice that many", "that many plus one": the change to the number of counters.
fn amount(s: &str) -> Option<(ReplacementAction, &str)> {
    if let Some(r) = s.strip_prefix("twice that many") {
        return Some((ReplacementAction::Multiply(2), r));
    }
    if let Some(r) = s.strip_prefix("that many plus one") {
        return Some((ReplacementAction::Add(Value::c(1)), r));
    }
    None
}

/// "If an effect would put one or more counters on a permanent you control, it puts twice
/// that many of those counters on that permanent instead." / "If you would put one or more
/// counters on a permanent you control, put that many plus one of each of those kinds of
/// counters on that permanent instead."
fn put_counters_replacement(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let def = put_counters_def(l)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

fn put_counters_def(l: &str) -> Option<ReplacementDef> {
    let r = end(l).strip_prefix("if ")?;
    let (by, effect_only, r) = if let Some(r) = r.strip_prefix("an effect would put ") {
        (None, true, r)
    } else if let Some(r) = r.strip_prefix("you would put ") {
        (Some(PlayerRel::You), false, r)
    } else {
        return None;
    };
    let r = r.strip_prefix("one or more ")?;
    let (kind, r) = match r.strip_prefix("counters on ") {
        Some(r) => (None, r),
        None => {
            let (k, r) = crate::oracle::costs::counter_kind(r)?;
            (Some(k), r.trim_start().strip_prefix("counters on ")?)
        }
    };
    let (who, r) = r.split_once(", ")?;
    let (on_objects, on_players) = recipients(who)?;
    let r = if effect_only {
        r.strip_prefix("it puts ")?
    } else {
        r.strip_prefix("put ")?
    };
    let (action, r) = amount(r)?;
    let what = match &kind {
        None if effect_only => " of those counters",
        None => " of each of those kinds of counters",
        Some(_) => "",
    };
    let r = r.strip_prefix(what)?;
    let r = match &kind {
        Some(k) => r.strip_prefix(' ')?.strip_prefix(k.as_str())?.strip_prefix(" counters")?,
        None => r,
    };
    let ok_tail = match (on_players.is_some(), kind.is_some()) {
        (true, _) => " on that permanent or player instead",
        (false, false) => " on that permanent instead",
        (false, true) => " on it instead",
    };
    if r != ok_tail {
        return None;
    }
    Some(ReplacementDef {
        event: ReplacementEvent::PutCountersMatching {
            on_objects,
            on_players,
            kind,
            by,
            effect_only,
        },
        action,
        self_replacement: false,
        optional: false,
    })
}

inventory::submit! { StaticPattern { name: "r122 if an effect/you would put counters on [recipients], more instead", priority: 100, parse: put_counters_replacement } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triggers() {
        for (t, each) in [
            ("you put one or more counters on a permanent or player", false),
            ("you put one or more +1/+1 counters on a creature you control", false),
            ("you put a +1/+1 counter on another creature", true),
            ("you put one or more -1/-1 counters on ~", false),
            ("you put a counter on a creature you control", true),
            ("you put one or more loyalty counters on a planeswalker", false),
        ] {
            let Some((TriggerCond::CountersPutBy { each: e, .. }, ..)) = put_by_trigger(t) else {
                panic!("{t}");
            };
            assert_eq!(e, each, "{t}");
        }
        assert!(put_by_trigger("you put one or more +1/+1 counters on one or more other heroes you control").is_none());
    }

    #[test]
    fn this_turn() {
        for c in [
            "you put a counter on ~ this turn",
            "you've put one or more +1/+1 counters on ~ this turn",
            "a counter was put on ~ this turn",
            "a +1/+1 counter was put on ~ this turn",
        ] {
            assert!(put_on_this_this_turn(c).is_some(), "{c}");
        }
    }

    #[test]
    fn replacements() {
        for l in [
            "if an effect would put one or more counters on a permanent you control, it puts twice that many of those counters on that permanent instead",
            "if an effect would put one or more counters on a permanent, it puts twice that many of those counters on that permanent instead",
            "if you would put one or more counters on a permanent you control, put that many plus one of each of those kinds of counters on that permanent instead",
            "if you would put one or more counters on a creature or planeswalker you control or on yourself, put that many plus one of each of those kinds of counters on that permanent or player instead",
            "if you would put one or more +1/+1 counters on a creature you control, put that many plus one +1/+1 counters on it instead",
        ] {
            assert!(put_counters_def(l).is_some(), "{l}");
        }
    }
}
