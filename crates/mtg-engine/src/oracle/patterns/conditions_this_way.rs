//! Conditions about the outcome of an earlier instruction of the same effect (CR 608.2c):
//!
//! - "If you do, [effect]" after an instruction that a condition governs ("If that
//!   creature was a Cleric, you may draw a card. If you do, you lose 1 life."): the
//!   follow-up belongs to the conditional part (whether "you do" is about the optional
//!   instruction, which didn't happen if the condition didn't hold);
//! - "If you can't, [effect]", "If the player can't, ...", "If you don't or can't make an
//!   exchange, ...", "If you can't sacrifice a creature, ...": a mandatory instruction that
//!   couldn't be performed (nothing to sacrifice, discard, return; CR 101.3, 609.3);
//! - "If they do, [effect]", "If the player does, ...": whether the player the previous
//!   instruction was about did it ("Counter target spell unless its controller pays {2}.
//!   If they do, ...", "that player discards a card at random. If the player does, ...");
//! - "If a land card was milled this way, ...", "If a creature is dealt damage this way,
//!   it gets +2/+2 ...", "If you exiled a land card this way, ...", "If that artifact is
//!   put into a graveyard this way, ...": whether any of the objects the previous
//!   instruction affected matches (CR 608.2c). "It" in the effect refers to those
//!   objects.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase, split_word};


/// Whether an effect ends with an optional instruction ("you may ...", "you may pay").
fn ends_optional(e: &Effect) -> bool {
    match e {
        Effect::May { .. } | Effect::PayOptional { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_optional),
        _ => false,
    }
}

/// The object a condition is about ("it's a creature card", "its mana value is 3 or
/// less"): what "it" means in the alternative ("Otherwise, put it into your hand."), where
/// the instruction the condition governed (which may have moved it) didn't happen.
pub fn cond_subject(c: &Condition) -> Option<Sel> {
    fn of_value(v: &Value) -> Option<Sel> {
        match v {
            Value::PowerOf(s) | Value::ToughnessOf(s) | Value::ManaValueOf(s) => Some((**s).clone()),
            Value::CountersOn(s, _) => Some((**s).clone()),
            _ => None,
        }
    }
    let s = match c {
        Condition::SelMatches(s, _) => Some(s.clone()),
        Condition::Compare(a, _, b) => of_value(a).or_else(|| of_value(b)),
        Condition::Not(x) => cond_subject(x),
        Condition::And(v) | Condition::Or(v) => v.iter().find_map(cond_subject),
        _ => None,
    }?;
    matches!(s, Sel::Target(_) | Sel::Var(_) | Sel::TriggerObject | Sel::TriggerLki).then_some(s)
}

/// "You may put it onto the battlefield if it's a creature card" parses as an optional
/// conditional instruction; as the last instruction of `prev`, turns it into the
/// equivalent conditional optional one, so that what follows ("If you don't, ...",
/// "Otherwise, ...") can refer to the condition.
pub fn normalize_may_if(prev: &mut Effect) {
    let last = match prev {
        Effect::Seq(v) => match v.last_mut() {
            Some(l) => l,
            None => return,
        },
        other => other,
    };
    let Effect::May { who, effect } = last else {
        return;
    };
    let Effect::If {
        cond,
        then,
        otherwise,
    } = &**effect
    else {
        return;
    };
    if !matches!(**otherwise, Effect::Noop) {
        return;
    }
    *last = Effect::If {
        cond: cond.clone(),
        then: Box::new(Effect::May {
            who: who.clone(),
            effect: then.clone(),
        }),
        otherwise: Box::new(Effect::Noop),
    };
}

/// "If you do, [effect]." / "If you don't [cast it], [effect]." right after "If
/// [condition], you may [instruction]." (or "You may [instruction] if [condition]."): the
/// follow-up belongs to the conditional part; "if you don't" also happens when the
/// condition didn't hold. After "[instruction] if [condition]." (not optional), "If you
/// don't [draw a card] this way, [effect]." is what happens otherwise.
fn if_you_do_after_conditional_may(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (neg, r, bare) = if let Some(r) = l.strip_prefix("if you do, ") {
        (false, r, true)
    } else if let Some(r) = l.strip_prefix("if you don't, ") {
        (true, r, true)
    } else if let Some(r) = l.strip_prefix("if you don't ") {
        // "If you don't cast it, ...", "If you don't put the card onto the battlefield,
        // ...", "If you don't draw a card this way, ..."
        match r.split_once(", ") {
            Some((_, x)) => (true, x, false),
            None => return false,
        }
    } else {
        return false;
    };
    let mut trial = prev.clone();
    normalize_may_if(&mut trial);
    let last = match &mut trial {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::If {
        cond,
        then,
        otherwise,
    }) = last
    else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) || matches!(cond, Condition::PrevHappened) {
        return false;
    }
    let optional = ends_optional(then);
    if !optional && (bare || !neg) {
        return false;
    }
    let saved_it = b.it.clone();
    if let Some(sel) = cond_subject(cond) {
        b.it = sel;
    }
    let parsed = crate::oracle::effects::parse_clause(r, b);
    b.it = saved_it;
    let Some(e) = parsed else {
        return false;
    };
    if optional {
        let cond = if neg {
            Condition::Not(Box::new(Condition::PrevHappened))
        } else {
            Condition::PrevHappened
        };
        let inner = std::mem::replace(&mut **then, Effect::Noop);
        **then = Effect::seq(vec![
            inner,
            Effect::If {
                cond,
                then: Box::new(e.clone()),
                otherwise: Box::new(Effect::Noop),
            },
        ]);
    }
    if neg {
        **otherwise = e;
    }
    *prev = trial;
    true
}

inventory::submit! { FollowupPattern { name: "if you do, after if [condition], you may", priority: 150, apply: if_you_do_after_conditional_may } }

/// Who performs an instruction that records whether it was done, if it's one: (the
/// player, whether it can be impossible to perform).
fn performer(e: &Effect) -> Option<(PlayerRef, bool)> {
    Some(match e {
        Effect::Sacrifice { who, .. } | Effect::Discard { who, .. } => (who.clone(), true),
        Effect::SacrificeObjects { .. } | Effect::ExchangeControl { .. } => (PlayerRef::You, true),
        // "Return a land card from your graveyard to your hand": one the player chooses.
        Effect::Move {
            what: Sel::Choose { up_to: false, .. },
            ..
        } => (PlayerRef::You, true),
        Effect::PayOptional {
            who,
            then,
            otherwise,
            ..
        } if matches!(**then, Effect::Noop) && !matches!(**otherwise, Effect::Noop) => {
            (who.clone(), false)
        }
        _ => return None,
    })
}

/// Instructions that don't change whether the previous one "happened".
fn keeps_outcome(e: &Effect) -> bool {
    matches!(e, Effect::LoseLife { .. } | Effect::GainLife { .. })
}

/// "If you can't, ...", "If the player can't, ...", "If they do, ...", "If the player
/// does, ...", "If you don't or can't make an exchange, ...".
fn outcome_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    // (subject is you, did it).
    let (you, did, verb) = match c {
        "you can't" => (true, false, None),
        "you don't or can't make an exchange" => (true, false, Some("exchange")),
        "the player can't" | "that player can't" | "they can't" => (false, false, None),
        "they do" | "the player does" | "that player does" => (false, true, None),
        "they don't" | "the player doesn't" | "that player doesn't" => (false, false, None),
        _ => match c.strip_prefix("you can't ") {
            // "If you can't sacrifice a creature, ..."
            Some(v) if v.starts_with("sacrifice ") => (true, false, Some("sacrifice")),
            _ => return false,
        },
    };
    // The instruction it's about: the last one, or (for "if you can't sacrifice ...") the
    // last of that kind, followed only by instructions that keep its outcome.
    let parts: Vec<Effect> = match &*prev {
        Effect::Seq(v) => v.clone(),
        other => vec![other.clone()],
    };
    let Some(i) = parts.iter().rposition(|e| performer(e).is_some()) else {
        return false;
    };
    if !parts[i + 1..].iter().all(keeps_outcome) {
        return false;
    }
    if verb.is_none() && i + 1 != parts.len() {
        return false;
    }
    let target = &parts[i];
    let Some((who, can_fail)) = performer(target) else {
        return false;
    };
    match verb {
        Some("sacrifice") if !matches!(target, Effect::Sacrifice { .. }) => return false,
        Some("exchange") if !matches!(target, Effect::ExchangeControl { .. }) => return false,
        _ => {}
    }
    // "can't": only instructions that can be impossible; "you": performed by you.
    if (!did && c.contains("can't") && !can_fail) || (you != matches!(who, PlayerRef::You)) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(x, b) else {
        return false;
    };
    let cond = if did {
        Condition::PrevHappened
    } else {
        Condition::Not(Box::new(Condition::PrevHappened))
    };
    let cond_effect = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = match old {
        Effect::Seq(mut v) => {
            v.push(cond_effect);
            Effect::Seq(v)
        }
        other => Effect::seq(vec![other, cond_effect]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "if you can't / if they do, [effect]", priority: 150, apply: outcome_followup } }

/// The objects an instruction that matched "if [objects] [verb] this way" affected.
pub const THIS_WAY: Var = vars::USER + 147;
/// What the instruction a "this way" condition asks about affected.
pub const SNAPSHOT: Var = vars::USER + 148;

/// Whether an effect is the snapshot [`if_this_way`] takes after an instruction.
fn is_snapshot(e: &Effect) -> bool {
    match e {
        Effect::Store { var, .. } => *var == SNAPSHOT,
        Effect::If { then, .. } => matches!(&**then, Effect::Store { var, .. } if *var == SNAPSHOT),
        _ => false,
    }
}

/// How a "this way" verb finds what the previous instruction did: which instruction, and
/// the variable holding the objects it affected. `back`: the instruction may be followed
/// by others (its variable is only set by instructions of its kind).
struct Verb {
    kind: fn(&Effect) -> bool,
    var: Var,
    back: bool,
}

fn verb(v: &str) -> Option<Verb> {
    fn dig_reveal(e: &Effect) -> bool {
        matches!(e, Effect::Dig { reveal: true, .. })
    }
    Some(match v {
        "milled" | "mill" => Verb {
            kind: |e| matches!(e, Effect::Mill { .. }),
            var: vars::IT,
            back: false,
        },
        "exiled" | "put into exile" | "exile" => Verb {
            kind: |e| matches!(e, Effect::Exile { .. }),
            var: vars::IT,
            back: false,
        },
        "discarded" | "discard" | "discards" => Verb {
            kind: |e| matches!(e, Effect::Discard { .. }),
            var: crate::discard_rules::DISCARDED,
            back: true,
        },
        "revealed" | "reveal" => Verb {
            kind: dig_reveal,
            var: vars::REVEALED,
            back: false,
        },
        "destroyed" => Verb {
            kind: |e| matches!(e, Effect::Destroy { .. }),
            var: vars::IT,
            back: false,
        },
        "put into a graveyard" => Verb {
            kind: |e| matches!(e, Effect::Destroy { .. } | Effect::Mill { .. }),
            var: vars::IT,
            back: false,
        },
        "sacrificed" => Verb {
            kind: |e| matches!(e, Effect::Sacrifice { .. }),
            var: vars::SACRIFICED,
            back: true,
        },
        "return" | "returned" => Verb {
            kind: |e| matches!(e, Effect::Move { .. }),
            var: vars::IT,
            back: false,
        },
        "dealt damage" => Verb {
            kind: |e| matches!(e, Effect::DealDamage { .. }),
            var: vars::DAMAGED,
            back: true,
        },
        _ => return None,
    })
}

/// The objects phrase after a quantifier: "a land card", "an instant or sorcery card",
/// "at least one Zombie card", "one or more creature cards".
fn quantified(s: &str) -> Option<(Filter, &str)> {
    let r = ["at least one ", "one or more ", "any ", "a ", "an "]
        .iter()
        .find_map(|q| s.strip_prefix(q))?;
    // "a card named ~"
    if let Some(rest) = r.strip_prefix("card named ~") {
        let f = Filter::and(vec![Filter::Card, Filter::SameNameAs(Box::new(Sel::This))]);
        return Some((f, rest.trim_start()));
    }
    let (f, _, rest) = parse_object_phrase(r)?;
    Some((f, rest.trim_start()))
}

/// "[objects] is/was/are/were [verb] this way", "you [verb] [objects] this way", "that
/// player discards [objects] this way", "that [noun] is put into a graveyard this way".
/// Returns (filter on the affected objects, verb, whether it's about "that [noun]", the
/// object the previous instruction acted on).
fn this_way_condition(c: &str) -> Option<(Filter, Verb, &str, bool)> {
    let c = c.strip_suffix(" this way")?;
    // "you exiled a land card", "that player discards an artifact card".
    for p in ["you ", "that player ", "the player "] {
        if let Some(r) = c.strip_prefix(p) {
            let (v, r) = split_word(r);
            let verb_ = verb(v)?;
            let (f, rest) = quantified(r)?;
            // "you return a nonland card to your hand this way"
            if !matches!(rest, "" | "to your hand" | "to the battlefield") {
                return None;
            }
            return Some((f, verb_, v, false));
        }
    }
    // "that artifact is put into a graveyard"
    if let Some(r) = c.strip_prefix("that ") {
        let (_noun, r) = split_word(r);
        let v = ["is ", "was "].iter().find_map(|x| r.strip_prefix(x))?;
        return Some((Filter::Any, verb(v)?, v, true));
    }
    let (f, rest) = quantified(c)?;
    let v = ["is ", "was ", "are ", "were "]
        .iter()
        .find_map(|x| rest.strip_prefix(x))?;
    Some((f, verb(v)?, v, false))
}

/// "If [objects] [verb] this way, [effect]." (also "[effect] if [objects] [verb] this
/// way.") after the instruction that verb names.
fn if_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let reordered;
    let l = match l.rsplit_once(" if ") {
        Some((x, c)) if !l.starts_with("if ") && c.ends_with(" this way") && !x.contains(',') => {
            reordered = format!("if {c}, {x}");
            reordered.as_str()
        }
        _ => l,
    };
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    // The condition may contain commas ("If a noncreature, nonland card is milled this
    // way, ...").
    let Some((c, x)) = r
        .split_once(" this way, ")
        .map(|(c, x)| (&r[..c.len() + " this way".len()], x))
    else {
        return false;
    };
    if x.ends_with(" instead") || x.starts_with("instead ") {
        return false;
    }
    let Some((f, verb, word, that)) = this_way_condition(c) else {
        return false;
    };
    // The instruction: the last one (or, for a variable only its kind sets, the last of
    // its kind), looking into an optional part ("you may reveal the top card").
    let parts: Vec<Effect> = match &*prev {
        Effect::Seq(v) => v.clone(),
        other => vec![other.clone()],
    };
    let is_kind = |e: &Effect| match e {
        Effect::May { effect, .. } => (verb.kind)(effect),
        other => (verb.kind)(other),
    };
    // A snapshot of what it affected, taken right after it, so that several sentences can
    // ask about it ("If a land card is milled this way, ... If a creature card is milled
    // this way, ...").
    let snap_after = |i: usize| parts.get(i + 1).is_some_and(is_snapshot);
    // Instructions that don't change what the most recent instruction affected.
    let keeps = |e: &Effect| {
        matches!(
            e,
            Effect::GrantPlayPermission { .. } | Effect::GainLife { .. } | Effect::LoseLife { .. }
        )
    };
    let i = match parts.iter().rposition(is_kind) {
        Some(i) if verb.back || parts[i + 1..].iter().all(keeps) || snap_after(i) => i,
        _ => return false,
    };
    let has_snapshot = snap_after(i);
    let (optional, instr) = match &parts[i] {
        Effect::May { effect, .. } => (true, (**effect).clone()),
        other => (false, other.clone()),
    };
    let in_var = Filter::In(Box::new(Sel::Var(SNAPSHOT)));
    let graveyard = word == "put into a graveyard";
    let what = Filter::and(vec![
        f.clone(),
        if graveyard {
            Filter::InZone(ZoneKind::Graveyard)
        } else {
            Filter::Any
        },
    ]);
    let affected = Filter::and(vec![what.clone(), in_var.clone()]);
    // Some affected object matches. (The sacrificed objects are their last known
    // information: they're checked as they were, not looked for where they are.)
    let snapshot = Sel::Var(SNAPSHOT);
    let any_matches = match &what {
        Filter::Any => Condition::SelNonEmpty(snapshot.clone()),
        w => Condition::And(vec![
            Condition::SelNonEmpty(snapshot.clone()),
            Condition::Not(Box::new(Condition::SelMatches(
                snapshot.clone(),
                Filter::not(w.clone()),
            ))),
        ]),
    };
    // What the condition checks, and what "it" in the effect refers to.
    let destroyed_target = match &instr {
        Effect::Destroy {
            what: what @ Sel::Target(_),
            ..
        } => Some(what.clone()),
        _ => None,
    };
    let (mut conds, it) = match (&destroyed_target, word, that) {
        // "that artifact is put into a graveyard this way": the object as it last existed
        // is still "that artifact".
        (_, _, true) => (vec![any_matches.clone()], None),
        // "If a creature is destroyed this way, you gain life equal to its toughness": the
        // destroyed creature as it last existed on the battlefield (CR 608.2h).
        (Some(t), "destroyed", false) => {
            let mut v = vec![Condition::SelNonEmpty(snapshot.clone())];
            if !matches!(f, Filter::Any) {
                v.push(Condition::SelMatches(t.clone(), f.clone()));
            }
            (v, Some(t.clone()))
        }
        _ => (vec![any_matches.clone()], Some(Sel::Var(THIS_WAY))),
    };
    let cond = if conds.len() == 1 {
        conds.pop().unwrap_or(Condition::Always)
    } else {
        Condition::And(conds)
    };
    let saved_it = b.it.clone();
    if let Some(it) = &it {
        b.it = it.clone();
    }
    let first_new = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(x, b) else {
        b.it = saved_it;
        b.targets.truncate(first_new);
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    if !has_snapshot {
        let store = |sel| Effect::Store { var: SNAPSHOT, sel };
        // An optional instruction that wasn't performed affected nothing.
        let snapshot = if optional {
            Effect::If {
                cond: Condition::PrevHappened,
                then: Box::new(store(Sel::Var(verb.var))),
                otherwise: Box::new(store(Sel::None)),
            }
        } else {
            store(Sel::Var(verb.var))
        };
        v.insert(i + 1, snapshot);
    }
    if matches!(it, Some(Sel::Var(THIS_WAY))) {
        v.push(Effect::Store {
            var: THIS_WAY,
            sel: Sel::All(affected),
        });
    }
    v.push(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if [objects] [verb] this way, [effect]", priority: 150, apply: if_this_way } }

/// "If it doesn't, [effect]." after "[effect] if it [has ...]": what happens if the
/// condition doesn't hold ("Draw a card if that creature has a +1/+1 counter on it. If it
/// doesn't, put a +1/+1 counter on it.").
fn if_it_doesnt(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = ["if it doesn't, ", "if it isn't, ", "if that creature doesn't, "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::If {
        cond, otherwise, ..
    }) = last
    else {
        return false;
    };
    // About an object (not "if you do").
    if !matches!(cond, Condition::SelMatches(..) | Condition::Compare(..)) {
        return false;
    }
    if !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(x, b) else {
        return false;
    };
    **otherwise = e;
    true
}

inventory::submit! { FollowupPattern { name: "if it doesn't, [effect]", priority: 150, apply: if_it_doesnt } }

/// "If you search your library this way, ...", "If you draw one or more cards this way,
/// ..." after "you may [search/draw]": whether you did.
fn if_you_did_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::May { who: PlayerRef::You, effect }) = last else {
        return false;
    };
    let cond = match (c, &**effect) {
        ("you search your library this way", Effect::Search { .. }) => Condition::PrevHappened,
        // The search found and put a card into your hand.
        ("you don't put a card into your hand this way", Effect::Search { to, .. })
            if to.zone == ZoneKind::Hand =>
        {
            Condition::Not(Box::new(Condition::And(vec![
                Condition::PrevHappened,
                Condition::SelNonEmpty(Sel::Var(vars::IT)),
            ])))
        }
        ("you draw one or more cards this way", Effect::Draw { n, .. }) => Condition::And(vec![
            Condition::PrevHappened,
            Condition::Compare(n.clone(), Cmp::Ge, Value::c(1)),
        ]),
        _ => return false,
    };
    let Some(e) = crate::oracle::effects::parse_clause(x, b) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if you search/draw this way, [effect]", priority: 150, apply: if_you_did_this_way } }

/// "Target player exiles a card from their graveyard. If it's a creature card, ...": when
/// "it" has no more specific antecedent (the source of a permanent's ability, or none in
/// a spell), it's the object the previous instruction acted on, if that instruction
/// chose it (a card a player chose, the top card of a library).
fn if_it_after_chosen_object(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    use crate::oracle::patterns::oracle_hardening_referents::is_no_referent;
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    if !(c.starts_with("it ") || c.starts_with("it's ") || c.starts_with("its "))
        || x.ends_with(" instead")
        || !(matches!(b.it, Sel::This) || is_no_referent(&b.it))
    {
        return false;
    }
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let chosen = |s: &Sel| matches!(s, Sel::Choose { .. } | Sel::TopOfLibrary(..));
    let found = match last {
        Some(Effect::Exile { what, .. } | Effect::Move { what, .. }) => chosen(what),
        _ => false,
    };
    if !found {
        return false;
    }
    let saved = b.it.clone();
    b.it = Sel::Var(vars::IT);
    let Some(cond) = super::conditions_referents::parse_condition_with(c, b) else {
        b.it = saved;
        return false;
    };
    let Some(e) = crate::oracle::effects::parse_clause(x, b) else {
        b.it = saved;
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if it [state] after an instruction about a chosen object", priority: 150, apply: if_it_after_chosen_object } }
