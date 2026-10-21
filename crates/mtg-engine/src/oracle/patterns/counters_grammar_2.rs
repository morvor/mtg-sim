//! Counter grammar II (CR 122): counter kinds chosen from a list or at random, counters
//! removed or moved from among a group, "all but N", counters copied from the counters an
//! event put, and doubling a player's counters.
//!
//! * "put your choice of a counter from among first strike, vigilance, deathtouch, and
//!   lifelink on ~" (Aragorn, Company Leader); "put a +1/+1 counter and a counter from
//!   among flying, first strike, lifelink, or vigilance on it" (Elspeth Resplendent):
//!   a keyword counter (CR 122.1b) of a kind the player chooses as the instruction is
//!   performed.
//! * "choose a kind of counter at random that ~ doesn't have on it from among flying,
//!   first strike, ..., and +1/+1. Put a counter of that kind on ~." (Crystalline Giant).
//! * "remove all but one +1/+1 counter from it" (Lily Bowen).
//! * "remove [N / up to N / any number of] [kind] counters from among [permanents]"
//!   (Eventide's Shadow, Overseer of Vault 76, Galloping Lizrog, Sensational Spider-Man)
//!   and "move any number of counters from among creatures you control onto that
//!   creature" (Slippery Bogbonder): the player chooses how many of each kind come from
//!   each permanent ([`Effect::CountersFromAmong`]).
//! * "move a counter of each kind not on ~ from another target permanent you control onto
//!   ~", "move one or more counters from ~ onto another target permanent you control"
//!   (Goldberry, River-Daughter).
//! * "they put the same number and kind of counters on ~" (Bold Plagiarist), "you may put
//!   the same number and kind of counters on ~" (Captain Marvel), "put one of each of
//!   those kinds of counters on up to one other target creature" (Aragorn): the counters
//!   the triggering event put.
//! * "double the number of each kind of counter you have" (Aetheric Amplifier).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::counter_groups::RANDOM_MISSING_KIND;
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::patterns::counter_grammar::{counter_noun, holder, quantity, Qty};
use crate::oracle::phrases::*;
use crate::types::CounterKind;

/// A counter kind named on its own: "+1/+1", "first strike", "flying".
fn kind_name(s: &str) -> Option<CounterKind> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if crate::layers::keyword_counter(s).is_some() {
        return Some(s.into());
    }
    let (k, rest) = crate::oracle::costs::counter_kind(s)?;
    rest.trim().is_empty().then_some(k)
}

/// "flying, first strike, lifelink, or vigilance", "first strike, vigilance, deathtouch,
/// and lifelink": a list of at least two kinds.
fn kind_list(s: &str) -> Option<Vec<CounterKind>> {
    let s = s.replace(", and ", ", ").replace(", or ", ", ");
    let s = match s.rsplit_once(" and ").or_else(|| s.rsplit_once(" or ")) {
        Some((a, z)) if !a.contains(", ") => format!("{a}, {z}"),
        _ => s,
    };
    let kinds: Vec<CounterKind> = s.split(", ").map(kind_name).collect::<Option<_>>()?;
    (kinds.len() >= 2).then_some(kinds)
}

/// The choice among `kinds` of one counter put on `what`.
fn choice_of(kinds: Vec<CounterKind>, what: &Sel) -> Effect {
    Effect::ChooseOne {
        who: PlayerRef::You,
        options: kinds
            .into_iter()
            .map(|kind| {
                (
                    format!("{kind} counter"),
                    Effect::AddCounters {
                        what: what.clone(),
                        kind,
                        n: Value::c(1),
                    },
                )
            })
            .collect(),
    }
}

/// "put your choice of a counter from among [kinds] on [object]", "put [counters] and a
/// counter from among [kinds] on [object]".
fn counter_from_among(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let (head, r) = r.split_once("a counter from among ")?;
    let (list, obj) = r.rsplit_once(" on ")?;
    let kinds = kind_list(list)?;
    if head == "your choice of " {
        let saved = b.targets.len();
        let (what, tail) = object_ref(obj, b)?;
        if !end(&tail).is_empty() {
            b.targets.truncate(saved);
            return None;
        }
        return Some(choice_of(kinds, &what));
    }
    // "put a +1/+1 counter and a counter from among ...": the first counters name the
    // object; the chosen one goes on it too.
    let first = head.strip_suffix(" and ")?;
    let saved = b.targets.len();
    let e = parse_clause(&format!("put {first} on {obj}"), b)?;
    let what = match &e {
        Effect::AddCounters { what, .. } => what.clone(),
        Effect::Seq(v) => match v.first() {
            Some(Effect::AddCounters { what, .. }) => what.clone(),
            _ => {
                b.targets.truncate(saved);
                return None;
            }
        },
        _ => {
            b.targets.truncate(saved);
            return None;
        }
    };
    Some(Effect::seq(vec![e, choice_of(kinds, &what)]))
}

inventory::submit! { EffectPattern { name: "counters grammar 2: a counter from among [kinds]", priority: 70, parse: counter_from_among } }

/// The variable holding the position of the kind chosen at random (from 1; 0: none).
const RANDOM_KIND: Var = vars::USER + 5101;

/// "choose a kind of counter at random that ~ doesn't have on it from among [kinds]".
fn random_missing_kind(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose a kind of counter at random that ")?;
    let (obj, list) = r.split_once(" doesn't have on it from among ")?;
    let kinds = kind_list(list)?;
    if obj != "~" {
        return None;
    }
    let _ = &b.targets;
    let json = serde_json::to_string(&(RANDOM_KIND, &kinds)).ok()?;
    let marker = format!(
        "\u{1}random counter kind:{}",
        kinds
            .iter()
            .map(|k| k.as_str().to_string())
            .collect::<Vec<_>>()
            .join("|")
    );
    b.named.push((marker, Sel::None));
    Some(Effect::Custom(
        format!("{RANDOM_MISSING_KIND}{json}").into(),
    ))
}

inventory::submit! { EffectPattern { name: "counters grammar 2: choose a kind of counter at random", priority: 70, parse: random_missing_kind } }

/// "Put a counter of that kind on ~." after a kind was chosen at random: the instruction
/// with the chosen kind (nothing if none could be chosen).
fn counter_of_that_kind(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(prev, Effect::Custom(n) if n.starts_with(RANDOM_MISSING_KIND)) {
        return false;
    }
    let Some((marker, _)) = b
        .named
        .iter()
        .rev()
        .find(|(p, _)| p.starts_with("\u{1}random counter kind:"))
        .cloned()
    else {
        return false;
    };
    let lower = l.to_lowercase();
    if !lower.contains("a counter of that kind") {
        return false;
    }
    let kinds: Vec<String> = marker
        .split_once(':')
        .map(|(_, w)| w.split('|').map(str::to_string).collect())
        .unwrap_or_default();
    let mut chain = Effect::Noop;
    for (i, k) in kinds.iter().enumerate().rev() {
        let text = lower.replacen("a counter of that kind", &format!("a {k} counter"), 1);
        let saved = b.targets.len();
        let Some(e) = parse_clause(end(&text), b) else {
            return false;
        };
        if b.targets.len() != saved || !matches!(e, Effect::AddCounters { .. }) {
            b.targets.truncate(saved);
            return false;
        }
        chain = Effect::If {
            cond: Condition::Compare(Value::Var(RANDOM_KIND), Cmp::Eq, Value::c(i as i32 + 1)),
            then: Box::new(e),
            otherwise: Box::new(chain),
        };
    }
    b.named.retain(|(p, _)| p != &marker);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, chain]);
    true
}

inventory::submit! { FollowupPattern { name: "counters grammar 2: a counter of that (random) kind", priority: 40, apply: counter_of_that_kind } }

/// "remove all but one +1/+1 counter from it", "remove all but N [kind] counters from
/// [holder]": as many as it has beyond N.
fn remove_all_but(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove all but ")?;
    let (n, r) = parse_number(r)?;
    let Value::Const(keep) = n else {
        return None;
    };
    let (kind, r) = counter_noun(r)?;
    let r = r.trim_start().strip_prefix("from ")?;
    let what = holder(r, b)?;
    if !matches!(
        what,
        Sel::This | Sel::Target(_) | Sel::TriggerObject | Sel::Var(_)
    ) {
        return None;
    }
    Some(Effect::RemoveCounters {
        what: what.clone(),
        kind: kind.clone(),
        n: Value::Max(
            Box::new(Value::c(0)),
            Box::new(Value::Diff(
                Box::new(Value::CountersOn(Box::new(what), kind)),
                Box::new(Value::c(keep)),
            )),
        ),
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: remove all but N counters", priority: 70, parse: remove_all_but } }

/// "permanents on the battlefield", "all permanents", "creatures you control": a group of
/// permanents.
fn group(s: &str) -> Option<Filter> {
    let s = end(s);
    let s = s.strip_prefix("all ").unwrap_or(s);
    let s = s.strip_suffix(" on the battlefield").unwrap_or(s);
    let (f, plural, tail) = parse_object_phrase(s)?;
    (plural && end(tail).is_empty())
        .then(|| Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]))
}

/// (min, max) of a quantity of counters.
fn bounds(q: Qty) -> Option<(Value, Option<Value>)> {
    Some(match q {
        Qty::Exact(n) => (n.clone(), Some(n)),
        Qty::UpTo(n) => (Value::c(0), Some(n)),
        Qty::AnyNumber => (Value::c(0), None),
        Qty::All => return None,
    })
}

/// "remove [quantity] [kind] counters from among [permanents]".
fn remove_from_among(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove ")?;
    let (q, r) = quantity(r, b)?;
    let (kind, r) = counter_noun(&r)?;
    let r = r.trim_start().strip_prefix("from among ")?;
    let f = group(r)?;
    let (min, max) = bounds(q)?;
    Some(Effect::CountersFromAmong {
        from: Sel::All(f),
        to: None,
        kind,
        min,
        max,
        each_missing_kind: false,
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: remove counters from among [permanents]", priority: 70, parse: remove_from_among } }

/// The single object counters move onto or from: "~", "it", "that creature", "that
/// planeswalker", a target ("another target permanent you control").
fn single(s: &str, b: &mut Builder) -> Option<Sel> {
    let s = end(s);
    if s == "~" {
        return Some(Sel::This);
    }
    if s == "it" || s.strip_prefix("that ").is_some_and(|n| !n.contains(' ')) {
        if s != "it" && !matches!(b.it, Sel::Target(_) | Sel::TriggerObject | Sel::Var(_)) {
            return None;
        }
        return Some(b.it.clone());
    }
    let (spec, tail) = parse_target(s)?;
    if !end(tail).is_empty()
        || spec.fixed_min() != Some(1)
        || !matches!(spec.max, Value::Const(1))
        || !matches!(spec.what, TargetKind::Object(_))
    {
        return None;
    }
    let slot = b.add_target(spec, s);
    Some(Sel::Target(slot))
}

/// "move any number of counters from among creatures you control onto that creature",
/// "move one or more counters from ~ onto another target permanent you control", "move a
/// counter of each kind not on ~ from another target permanent you control onto ~".
fn move_counters_2(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("move ")?;
    let saved = (b.targets.len(), b.it.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
    };
    // "a counter of each kind not on ~ from [object] onto ~".
    if let Some(r) = r.strip_prefix("a counter of each kind not on ") {
        let (dest, r) = r.split_once(" from ")?;
        let (from, onto) = r.split_once(" onto ")?;
        if end(dest) != end(onto) {
            return None;
        }
        let Some((from, to)) = single(from, b).and_then(|f| Some((f, single(onto, b)?))) else {
            restore(b);
            return None;
        };
        return Some(Effect::CountersFromAmong {
            from,
            to: Some(to),
            kind: None,
            min: Value::c(0),
            max: None,
            each_missing_kind: true,
        });
    }
    let (min, max, r) = if let Some(r) = r.strip_prefix("one or more ") {
        (Value::c(1), None, r.to_string())
    } else {
        let (q, r) = quantity(r, b)?;
        let (min, max) = bounds(q)?;
        (min, max, r)
    };
    let (kind, r) = counter_noun(&r)?;
    let r = r.trim_start().strip_prefix("from ")?;
    let (from, onto) = r.split_once(" onto ")?;
    let from_group = from.strip_prefix("among ");
    // Plain moves between two objects are the core move pattern's, except "one or more".
    if from_group.is_none() && !matches!(min, Value::Const(1) if max.is_none()) {
        return None;
    }
    let from_sel = match from_group {
        Some(g) => Sel::All(group(g)?),
        None => match single(from, b) {
            Some(s) => s,
            None => {
                restore(b);
                return None;
            }
        },
    };
    let Some(to) = single(onto, b) else {
        restore(b);
        return None;
    };
    Some(Effect::CountersFromAmong {
        from: from_sel,
        to: Some(to),
        kind,
        min,
        max,
        each_missing_kind: false,
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: move counters from among / one or more / each kind", priority: 70, parse: move_counters_2 } }

/// "they put the same number and kind of counters on ~", "put the same number and kind of
/// counters on ~", "put one of each of those kinds of counters on up to one other target
/// creature": the kind (and number) of counters the triggering event put.
fn same_as_event(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = match l.strip_prefix("they put ") {
        Some(r) => (Some(PlayerRef::TriggerPlayer), r),
        None => (None, l.strip_prefix("put ")?),
    };
    let (one, obj) = if let Some(o) = r.strip_prefix("the same number and kind of counters on ") {
        (false, o)
    } else if let Some(o) = r.strip_prefix("one of each of those kinds of counters on ") {
        (true, o)
    } else {
        return None;
    };
    if !b.in_trigger {
        return None;
    }
    let saved = b.targets.len();
    let (to, tail) = object_ref(obj, b)?;
    if !end(&tail).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    let e = Effect::PutEventCounters { to, one_each: one };
    Some(match who {
        Some(who) => Effect::AsPlayer {
            who,
            effect: Box::new(e),
        },
        None => e,
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: the same number and kind of counters", priority: 70, parse: same_as_event } }

/// "double the number of each kind of counter you have": each kind of counter the
/// player has, as many again (CR 701.10e).
fn double_player_counters(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("double the number of each kind of counter ")?;
    let who = match r {
        "you have" => PlayerRef::You,
        _ => return None,
    };
    let k: CounterKind = CHOSEN_COUNTER_KIND.into();
    Some(Effect::ForEachCounterKind {
        from: Sel::Players(who.clone()),
        then: Box::new(Effect::AddCounters {
            what: Sel::Players(who.clone()),
            kind: k.clone(),
            n: Value::PlayerCounters(who, k),
        }),
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: double each kind of counter a player has", priority: 70, parse: double_player_counters } }

/// "an opponent puts one or more counters on a creature they control": counters put by a
/// player on a permanent that player controls ("they" is the player putting them).
fn put_on_their_own(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (head, subject) = r.split_once(" counters on ")?;
    let subject = subject.strip_suffix(" they control")?;
    if !(head.ends_with(" puts one or more") || head.ends_with(" put one or more"))
        || head.starts_with("you ")
    {
        return None;
    }
    let (t, it, p) =
        crate::oracle::triggers::parse_trigger_condition(&format!("{head} counters on {subject}"))?;
    if !matches!(t, TriggerCond::CountersPutBy { .. }) {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(t),
            cond: Condition::SelMatches(
                Sel::TriggerObject,
                Filter::ControlledBy(PlayerRel::TriggerPlayer),
            ),
        },
        it,
        p,
    ))
}

inventory::submit! { super::TriggerPattern { name: "counters grammar 2: a player puts counters on a permanent they control", priority: 90, parse: put_on_their_own } }

/// "remove a number of loyalty counters equal to that card's mana value from ~".
fn remove_a_number_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove a number of ")?;
    let (kind, r) = counter_noun(r)?;
    let r = r.trim_start().strip_prefix("equal to ")?;
    let saved = b.targets.len();
    let (n, rest) = super::value_grammar::parse_value(r, b)?;
    let Some(h) = rest.trim_start().strip_prefix("from ") else {
        b.targets.truncate(saved);
        return None;
    };
    let what = holder(h, b)?;
    Some(Effect::RemoveCounters { what, kind, n })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: remove a number of counters equal to", priority: 70, parse: remove_a_number_equal_to } }

/// How many counters the latest [`Effect::CountersFromAmong`] removed or moved.
const GROUP_COUNT: Var = vars::USER + 5102;

/// Whether `e` ends with an [`Effect::CountersFromAmong`].
fn ends_with_group_op(e: &Effect) -> bool {
    match e {
        Effect::CountersFromAmong { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_group_op),
        Effect::May { effect, .. } => ends_with_group_op(effect),
        _ => false,
    }
}

/// After counters were removed or moved from among a group: "You draw cards and lose
/// life equal to the number of counters removed this way.", "Draw cards equal to the
/// number of stun counters removed this way.", "If you do, put twice that many +1/+1
/// counters on ~.": the number of counters (CR 608.2c).
fn after_group_op(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_group_op(prev) {
        return false;
    }
    let lower = l.to_lowercase();
    let l = end(&lower);
    if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return false;
    }
    let mut text = l.to_string();
    // "the number of [kind] counters removed this way".
    if let Some(i) = text.find("the number of ") {
        let tail = &text[i + "the number of ".len()..];
        let Some(j) = ["removed this way", "moved this way"]
            .iter()
            .find_map(|p| tail.find(p).map(|j| j + p.len()))
        else {
            return false;
        };
        let noun = &tail[..j];
        let noun = noun
            .strip_suffix(" removed this way")
            .or_else(|| noun.strip_suffix(" moved this way"))
            .unwrap_or(noun);
        if !counter_noun(noun).is_some_and(|(_, r)| r.trim().is_empty()) {
            return false;
        }
        text = format!("{}x{}", &text[..i], &tail[j..]);
    } else if text.contains("that many") {
        text = text.replacen("that many", "x", 1);
    } else {
        return false;
    }
    // "you draw cards and lose life equal to x": each instruction of the list.
    let e = if let Some(r) = text
        .strip_prefix("you draw cards and lose life equal to ")
        .or_else(|| text.strip_prefix("draw cards and lose life equal to "))
    {
        if r != "x" {
            return false;
        }
        Effect::seq(vec![
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::Var(GROUP_COUNT),
            },
            Effect::LoseLife {
                who: PlayerRef::You,
                n: Value::Var(GROUP_COUNT),
            },
        ])
    } else {
        let Some(e) = crate::oracle::effects::parse_sentence(&text, b) else {
            return false;
        };
        let Some(e) = super::r107_numbers::substitute_x(&e, &Value::Var(GROUP_COUNT)) else {
            return false;
        };
        e
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::StoreValue {
            var: GROUP_COUNT,
            value: Value::c(0),
        },
        store_after_group_op(old),
        e,
    ]);
    true
}

/// `e` with the count of its final [`Effect::CountersFromAmong`] stored in [`GROUP_COUNT`].
fn store_after_group_op(e: Effect) -> Effect {
    match e {
        Effect::CountersFromAmong { .. } => Effect::seq(vec![
            e,
            Effect::StoreValue {
                var: GROUP_COUNT,
                value: Value::Prev,
            },
        ]),
        Effect::Seq(mut v) => {
            if let Some(last) = v.pop() {
                v.push(store_after_group_op(last));
            }
            Effect::Seq(v)
        }
        Effect::May { who, effect } => Effect::May {
            who,
            effect: Box::new(store_after_group_op(*effect)),
        },
        e => e,
    }
}

inventory::submit! { FollowupPattern { name: "counters grammar 2: counters removed or moved this way", priority: 45, apply: after_group_op } }

/// The permanent each "for each [permanent], choose a counter on it" is about.
const EACH_HOLDER: Var = vars::USER + 5103;

/// "For each non-Saga permanent, choose a counter on it." (The Caves of Androzani): a kind
/// of counter chosen for each permanent in turn; the next sentence says what's done with
/// it (see [`for_each_that_kind`]).
fn for_each_choose_a_counter(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (noun, rest) = r.split_once(", ")?;
    if rest != "choose a counter on it" {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some(Effect::ForEach {
        sel: Sel::All(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])),
        var: EACH_HOLDER,
        effect: Box::new(Effect::ChooseCounterKind {
            from: Sel::Var(EACH_HOLDER),
            then: Box::new(Effect::Noop),
        }),
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: for each [permanent], choose a counter on it", priority: 70, parse: for_each_choose_a_counter } }

/// "You may put an additional counter of that kind on that permanent." after "For each
/// [permanent], choose a counter on it.".
fn for_each_that_kind(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::ForEach { var, effect, .. } = prev else {
        return false;
    };
    if *var != EACH_HOLDER {
        return false;
    }
    let Effect::ChooseCounterKind { then, .. } = effect.as_mut() else {
        return false;
    };
    if !matches!(**then, Effect::Noop) {
        return false;
    }
    let lower = l.to_lowercase();
    let k = CHOSEN_COUNTER_KIND;
    let text = end(&lower)
        .replace(
            "an additional counter of that kind",
            &format!("an additional {k} counter"),
        )
        .replace("a counter of that kind", &format!("a {k} counter"))
        .replace(" on that permanent", " on it");
    if !text.contains(k) {
        return false;
    }
    let saved = (b.it.clone(), b.targets.len());
    b.it = Sel::Var(EACH_HOLDER);
    let e = crate::oracle::effects::parse_sentence(&text, b);
    let ok = b.targets.len() == saved.1;
    b.it = saved.0;
    let Some(e) = e.filter(|_| ok) else {
        b.targets.truncate(saved.1);
        return false;
    };
    **then = e;
    true
}

inventory::submit! { FollowupPattern { name: "counters grammar 2: for each permanent, [instruction] of that kind", priority: 45, apply: for_each_that_kind } }

/// "Then for each kind of counter among creatures you control, put a counter of that kind
/// on either of those tokens." (Exotic Pets): one counter of each kind among them, each
/// on one of the tokens just created, chosen for each kind.
fn each_kind_among_on_either(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    fn creates(e: &Effect) -> bool {
        match e {
            Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } => true,
            Effect::Seq(v) => v.last().is_some_and(creates),
            _ => false,
        }
    }
    if !creates(prev) {
        return false;
    }
    let lower = l.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l.strip_prefix("for each kind of counter among ") else {
        return false;
    };
    let Some((noun, rest)) = r.split_once(", ") else {
        return false;
    };
    if rest != "put a counter of that kind on either of those tokens"
        && rest != "put a counter of that kind on one of those tokens"
    {
        return false;
    }
    let Some(f) = group(noun) else {
        return false;
    };
    let k: CounterKind = CHOSEN_COUNTER_KIND.into();
    let e = Effect::ForEachCounterKind {
        from: Sel::All(f),
        then: Box::new(Effect::AddCounters {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::In(Box::new(Sel::Var(vars::CREATED))),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            kind: k,
            n: Value::c(1),
        }),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "counters grammar 2: for each kind of counter among, on either of those tokens", priority: 45, apply: each_kind_among_on_either } }

/// "Each other creature you control enters with a number of additional +1/+1 counters on
/// it equal to ~'s power and as a Mutant in addition to its other types." (Master
/// Biomancer): two modifications of how those permanents enter (CR 614.1c), each read on
/// its own: the counters, and "as [another such permanent] enters, it becomes a Mutant in
/// addition to its other types".
fn enters_with_counters_and_as(
    l: &str,
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l);
    let (subject, rest) = l.split_once(" enters with ")?;
    let (counters, typ) = rest.rsplit_once(" and as ")?;
    if !typ.ends_with(" in addition to its other types") {
        return None;
    }
    let one = subject.strip_prefix("each other ")?;
    let mut out =
        crate::oracle::statics::parse_static(&format!("{subject} enters with {counters}."), ctx)?;
    let types = crate::oracle::statics::parse_static(
        &format!("As another {one} enters, it becomes {typ}."),
        ctx,
    )?;
    out.extend(types);
    for a in &mut out {
        std::sync::Arc::make_mut(a).text = text.into();
    }
    Some(out)
}

inventory::submit! { super::StaticPattern { name: "counters grammar 2: others enter with counters and as a [type]", priority: 90, parse: enters_with_counters_and_as } }

/// In an ability that triggers on casting a spell: "that creature enters with an
/// additional +1/+1 counter on it for each Dog and/or Wolf you control" (Torgal), "~
/// enters with two +1/+1 counters on it for each creature sacrificed this way" (Gluttonous
/// Hellkite), "That planeswalker enters with an additional loyalty counter on it" (Dark
/// Intimations): a replacement effect for the permanent that spell becomes (CR 614.1c,
/// 400.7a). A number the resolving ability determines ("sacrificed this way") is locked
/// in as it resolves (CR 608.2h); a count of permanents is taken as it enters.
fn cast_spell_enters_with(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, rest) = l.split_once(" enters with ")?;
    // "When you cast ~, ... ~ enters with ...": the spell is the trigger's.
    let self_cast = subject == "~" && b.in_trigger && b.ctx.is_permanent() && b.sentences > 0;
    if !matches!(b.it, Sel::TriggerSpell) && !self_cast {
        return None;
    }
    let one_word_that = subject
        .strip_prefix("that ")
        .is_some_and(|n| !n.contains(' ') && n != "spell");
    if !(one_word_that || subject == "~" || subject == "it") {
        return None;
    }
    // Only counters: "an additional +1/+1 counter on it ...", "two +1/+1 counters on it".
    let rest = rest.replacen(" additional ", " ", 1);
    let rest = rest
        .strip_prefix("an ")
        .map_or(rest.clone(), |r| format!("a {r}"));
    let (head, tail) = rest
        .split_once(" counter on it")
        .or_else(|| rest.split_once(" counters on it"))?;
    let plural = rest.contains(" counters on it");
    let saved = (b.targets.len(), b.it.clone());
    let text = format!(
        "put {head} {} on ~{tail}",
        if plural { "counters" } else { "counter" }
    );
    let e = parse_clause(&text, b);
    let ok = b.targets.len() == saved.0;
    b.it = saved.1;
    let Some(Effect::AddCounters { kind, n, .. }) = e.filter(|_| ok) else {
        b.targets.truncate(saved.0);
        return None;
    };
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::In(Box::new(Sel::TriggerSpell))),
            action: ReplacementAction::EnterWithCounters(kind, n),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::Permanent,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: the cast spell enters with counters", priority: 85, parse: cast_spell_enters_with } }

/// "distribute three +1/+1 counters among one, two, or three creatures you control",
/// "distribute X +1/+1 counters among any number of creatures you control", "distribute a
/// number of +1/+1 counters equal to [value] among any number of creatures you control",
/// "distribute four +1/+1 counters among any number of creatures and/or Vehicles target
/// player controls": counters divided among objects chosen as it resolves (no targets,
/// CR 608.2d).
fn distribute_untargeted(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("distribute ")?;
    let saved = (b.targets.len(), b.it.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
    };
    let (n, up_to, kind, rest) = if let Some(r) = r.strip_prefix("a number of ") {
        let (kind, r) = counter_noun(r)?;
        let r = r.trim_start().strip_prefix("equal to ")?;
        let (v, rest) = super::value_grammar::parse_value(r, b)?;
        let Some(rest) = rest.trim_start().strip_prefix("among ").map(str::to_string) else {
            restore(b);
            return None;
        };
        (v, false, kind?, rest)
    } else {
        let (q, r) = quantity(r, b)?;
        let (n, up_to) = match q {
            Qty::Exact(n) => (n, false),
            Qty::UpTo(n) => (n, true),
            _ => return None,
        };
        let (kind, r) = counter_noun(&r)?;
        (
            n,
            up_to,
            kind?,
            r.trim_start().strip_prefix("among ")?.to_string(),
        )
    };
    let (min, max, group) = if let Some(g) = rest.strip_prefix("one, two, or three ") {
        (1, Some(3), g)
    } else if let Some(g) = rest.strip_prefix("one or two ") {
        (1, Some(2), g)
    } else if let Some(g) = rest.strip_prefix("any number of ") {
        (0, None, g)
    } else {
        return None;
    };
    // Targets are the core distribution's (CR 601.2d).
    if group.contains("target ") && !group.contains(" target player controls") {
        return None;
    }
    let Some((sel, tail)) = object_ref(&format!("all {group}"), b) else {
        restore(b);
        return None;
    };
    if !end(&tail).is_empty() || !matches!(sel, Sel::All(_)) {
        restore(b);
        return None;
    }
    Some(Effect::DistributeCounters {
        n,
        kind,
        among: sel,
        min_objects: min,
        max_objects: max,
        up_to,
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: distribute counters among objects chosen on resolution", priority: 90, parse: distribute_untargeted } }

/// "Put four time counters on each of those cards that has suspend." (Ecstatic Beauty):
/// on each of the cards the text just named that has that keyword.
fn on_each_of_those_that_has(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let (counters, rest) = r.split_once(" on each of those cards that has ")?;
    let kw = crate::keywords::KeywordKind::from_name(rest.trim())?;
    let Sel::Var(v) = b.it.clone() else {
        return None;
    };
    let saved = b.targets.len();
    let e = parse_clause(&format!("put {counters} on ~"), b)?;
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    let Effect::AddCounters { kind, n, .. } = e else {
        return None;
    };
    Some(Effect::AddCounters {
        what: Sel::Matching(Box::new(Sel::Var(v)), Filter::HasKeyword(kw)),
        kind,
        n,
    })
}

inventory::submit! { EffectPattern { name: "counters grammar 2: on each of those cards that has [keyword]", priority: 70, parse: on_each_of_those_that_has } }

/// "For each kind of counter on ~, each other Dinosaur you control enters with a counter
/// of that kind on it." (Blue, Loyal Raptor): a counter of each kind on ~ as each of
/// those permanents enters (CR 614.1c, 122.6).
fn each_kind_enters_with(
    l: &str,
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("for each kind of counter on ~, ")?;
    let subj = r.strip_suffix(" enters with a counter of that kind on it")?;
    let one = subj.strip_prefix("each other ")?;
    let (f, plural, tail) = parse_object_phrase(one)?;
    if plural || !end(tail).is_empty() || !ctx.is_permanent() {
        return None;
    }
    let f = match f {
        Filter::And(v) => Filter::and(
            v.into_iter()
                .filter(|x| !matches!(x, Filter::Permanent))
                .chain([Filter::Other])
                .collect(),
        ),
        f => Filter::and(vec![f, Filter::Other]),
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(f),
                action: ReplacementAction::EnterWithCounters(
                    EACH_KIND_ON_SOURCE.into(),
                    Value::c(1),
                ),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "counters grammar 2: for each kind of counter on ~, others enter with one", priority: 90, parse: each_kind_enters_with } }

/// "At the beginning of each end step, if you gained life this turn, distribute up to that
/// many +1/+1 counters among any number of other target creatures." (Lathiel, the
/// Bounteous Dawn): the amount is the life gained this turn as the ability is put on the
/// stack, divided among the targets then (CR 601.2d, 603.3d); with no targets nothing is
/// distributed (its rulings).
fn distribute_up_to_that_much_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("distribute up to that many ")?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("if you gained life this turn, distribute up to that many") {
        return None;
    }
    let (kind, r) = counter_noun(r)?;
    let kind = kind?;
    let r = r.trim_start().strip_prefix("among any number of ")?;
    let (mut spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    let n = Value::LifeGainedThisTurn(PlayerRef::You);
    spec.min = Value::c(0);
    spec.max = n.clone();
    spec.divide = Some(n);
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::Custom(
        crate::counter_rules::divided_counters_effect(slot, &kind),
    ))
}

inventory::submit! { EffectPattern { name: "counters grammar 2: distribute up to that many (life gained)", priority: 70, parse: distribute_up_to_that_much_life } }
