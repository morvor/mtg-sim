//! Results "this way" (round 5):
//!
//! * "If damage would be dealt to ~ while it has a +1/+1 counter on it, prevent that damage
//!   and remove that many +1/+1 counters from it. When one or more counters are removed
//!   from ~ this way, it deals that much damage to any target." (Magma Pummeler): a
//!   reflexive triggered ability (CR 603.12) of the instruction that removed the counters;
//!   it triggers if one or more were removed, and "that much" is the amount the
//!   instruction removed counters for (the prevented damage: with fewer counters than
//!   that, all of them are removed and "it will still deal that much damage", Magma
//!   Pummeler's ruling).
//! * "At the beginning of the next end step, return each of them to the battlefield under
//!   its owner's control. Each of them enters with an additional +1/+1 counter on it if
//!   it's a creature and an additional loyalty counter on it if it's a planeswalker."
//!   (Semester's End): replacement effects of the returning instruction on how those
//!   objects enter (CR 614.1c, 122.6), each kind of counter for each object that would
//!   enter as a permanent of that type (CR 614.12); the objects return together.
//! * "Each opponent mills two cards. When one or more cards are milled this way, exile
//!   target ..." (Saruman of Many Colors): a reflexive triggered ability (CR 603.12) that
//!   triggers if the mill instruction milled at least one such card (CR 701.17).
//!   Likewise "When you put one or more Equipment onto the battlefield this way, you may
//!   attach one of them to a Samurai you control." (Gilgamesh, Master-at-Arms) after an
//!   instruction putting cards onto the battlefield: "them" are those permanents.

use super::FollowupPattern;
use crate::ability::*;
use crate::types::CounterKind;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// Whether the last instruction of `e` removes counters from the source.
fn removes_own_counters(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(removes_own_counters),
        Effect::RemoveCounters { what: Sel::This, .. } => true,
        _ => false,
    }
}

/// "When one or more counters are removed from ~ this way, it deals that much damage to
/// any target." after an instruction removing counters from ~.
fn when_counters_removed_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("when one or more ") else {
        return false;
    };
    let Some((what, effect)) = r.split_once(" are removed from ~ this way, ") else {
        return false;
    };
    // Any counters, or the kind the instruction removes ("+1/+1 counters").
    let kind_ok = what == "counters"
        || what.strip_suffix(" counters").is_some_and(|k| {
            matches!(prev_kind(prev), Some(Some(pk)) if pk.as_str() == k)
        });
    if !kind_ok || !removes_own_counters(prev) {
        return false;
    }
    // In a replacement's instructions "that much" has already been read as X, the
    // amount of the event (`replacement_grammar::instructions`); the amount the counters
    // were removed for is that X too.
    let effect = effect
        .replace("that much damage", "x damage")
        .replace("that many", "x");
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::This;
    sub.it_player = b.it_player.clone();
    let body = super::value_grammar::with_x_defined(true, || parse_effect_text(&effect, &mut sub));
    let Some(body) = body else {
        return false;
    };
    let Some(amount) = prev_amount(prev) else {
        return false;
    };
    // Whether the instruction removed any (its result, CR 608.2c).
    let reflexive = Effect::If {
        cond: Condition::Compare(Value::Prev, Cmp::Ge, Value::c(1)),
        then: Box::new(Effect::seq(vec![
            Effect::SetX { value: amount },
            Effect::Reflexive {
                body: Box::new(Body {
                    targets: sub.targets,
                    effect: body,
                    modal: None,
                }),
            },
        ])),
        otherwise: Box::new(Effect::Noop),
    };
    append(prev, reflexive);
    true
}

/// The counter kind the last instruction of `e` removes.
fn prev_kind(e: &Effect) -> Option<Option<CounterKind>> {
    match e {
        Effect::Seq(v) => v.last().and_then(prev_kind),
        Effect::RemoveCounters { kind, .. } => Some(kind.clone()),
        _ => None,
    }
}

/// The amount the last instruction of `e` removes counters for ("remove that many").
fn prev_amount(e: &Effect) -> Option<Value> {
    match e {
        Effect::Seq(v) => v.last().and_then(prev_amount),
        Effect::RemoveCounters { n, .. } => Some(n.clone()),
        _ => None,
    }
}

/// Adds `e` after the instructions of `prev`.
fn append(prev: &mut Effect, e: Effect) {
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
}

inventory::submit! { FollowupPattern { name: "results 5: when one or more counters are removed from ~ this way", priority: 0, apply: when_counters_removed_this_way } }

/// One "an additional [kind] counter on it if it's a [type]" part.
fn conditional_counter(s: &str) -> Option<(CounterKind, Value, Filter)> {
    let r = s.trim().strip_prefix("an additional ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counter on it if it's ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some((kind, Value::c(1), f))
}

/// The last move onto the battlefield of `e`, through sequences and the body of a delayed
/// triggered ability that `e` ends with.
fn last_battlefield_move(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(e),
        Effect::Seq(v) => v.last_mut().and_then(last_battlefield_move),
        Effect::DelayedTrigger { body, .. } => last_battlefield_move(&mut body.effect),
        _ => None,
    }
}

/// "Each of them enters with an additional +1/+1 counter on it if it's a creature and an
/// additional loyalty counter on it if it's a planeswalker." after a move onto the
/// battlefield.
fn each_enters_with_if(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = ["each of them enters with ", "it enters with "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    let mut parts = Vec::new();
    for part in r.split(" and ") {
        match conditional_counter(part) {
            Some(x) => parts.push(x),
            None => return false,
        }
    }
    let Some(mv) = last_battlefield_move(prev) else {
        return false;
    };
    let Effect::Move { what, .. } = &*mv else {
        return false;
    };
    let moved = what.clone();
    let mut steps: Vec<Effect> = parts
        .into_iter()
        .map(|(kind, n, f)| Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::and(vec![
                    Filter::In(Box::new(moved.clone())),
                    f,
                ])),
                action: ReplacementAction::EnterWithCounters(kind, n),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        })
        .collect();
    let old = std::mem::take(mv);
    steps.push(old);
    *mv = Effect::seq(steps);
    true
}

inventory::submit! { FollowupPattern { name: "results 5: each of them enters with an additional counter if it's a [type]", priority: 0, apply: each_enters_with_if } }

/// Whether the last instruction of `e` mills cards.
fn mills(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(mills),
        Effect::Mill { .. } => true,
        _ => false,
    }
}

/// "When one or more [cards] are milled this way, [effect]" after a mill instruction.
fn when_milled_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("when one or more ") else {
        return false;
    };
    let Some((what, effect)) = r.split_once(" are milled this way, ") else {
        return false;
    };
    if !mills(prev) {
        return false;
    }
    let q = match what {
        "cards" => None,
        w => {
            let Some(q) = super::card_flow_search::card_filter(w, b) else {
                return false;
            };
            Some(q)
        }
    };
    // The reflexive ability as "When you do, [effect]" reads it, with its own targets.
    let Some(mut e) = crate::oracle::effects::parse_sentence(&format!("when you do, {effect}"), b)
    else {
        return false;
    };
    let Effect::If { cond, .. } = &mut e else {
        return false;
    };
    let milled = match q {
        None => Sel::Var(vars::IT),
        Some(f) => Sel::Matching(Box::new(Sel::Var(vars::IT)), f),
    };
    *cond = Condition::Compare(Value::CountSel(Box::new(milled)), Cmp::Ge, Value::c(1));
    append(prev, e);
    true
}

inventory::submit! { FollowupPattern { name: "results 5: when one or more cards are milled this way", priority: 0, apply: when_milled_this_way } }

/// The reflexive ability `e` ends with, if it's a [`when_milled_this_way`] one.
fn milled_reflexive(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(milled_reflexive),
        Effect::If {
            cond: Condition::Compare(Value::CountSel(x), Cmp::Ge, Value::Const(1)),
            then,
            otherwise,
        } if matches!(**otherwise, Effect::Noop)
            && matches!(x.as_ref(), Sel::Var(vars::IT) | Sel::Matching(..)) =>
        {
            match &mut **then {
                Effect::Reflexive { body } => Some(body),
                _ => None,
            }
        }
        _ => None,
    }
}

/// "Copy the exiled card. You may cast the copy without paying its mana cost." after
/// "When one or more cards are milled this way, exile target ...": the rest of the
/// reflexive ability (as after "When you do, ...").
fn continues_milled_reflexive(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(body) = milled_reflexive(prev) else {
        return false;
    };
    if body.modal.is_some() {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    sub.targets = body.targets.clone();
    if crate::oracle_ext::apply_followup_ext(l, &mut body.effect, &mut sub) {
        body.targets = sub.targets;
        return true;
    }
    let Some(e) = crate::oracle::effects::parse_sentence(l, &mut sub) else {
        return false;
    };
    body.targets = sub.targets;
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "results 5: the rest of a reflexive ability for cards milled this way", priority: 0, apply: continues_milled_reflexive } }

/// Whether the last instruction of `e` puts cards onto the battlefield (a dig's selection
/// or a move).
fn puts_onto_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(puts_onto_battlefield),
        Effect::Dig { take_to, .. } => take_to.zone == ZoneKind::Battlefield,
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield,
        Effect::May { effect, .. } => puts_onto_battlefield(effect),
        _ => false,
    }
}

/// "When you put one or more [permanents] onto the battlefield this way, [effect]".
fn when_put_onto_battlefield_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("when you put one or more ") else {
        return false;
    };
    let Some((what, effect)) = r.split_once(" onto the battlefield this way, ") else {
        return false;
    };
    if !puts_onto_battlefield(prev) {
        return false;
    }
    let Some((f, _, tail)) = crate::oracle::phrases::parse_object_phrase(what) else {
        return false;
    };
    if !tail.trim().is_empty() {
        return false;
    }
    let Some(mut e) = crate::oracle::effects::parse_sentence(&format!("when you do, {effect}"), b)
    else {
        return false;
    };
    let Effect::If { cond, .. } = &mut e else {
        return false;
    };
    let put = Sel::Matching(
        Box::new(Sel::Var(vars::IT)),
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
    );
    *cond = Condition::Compare(Value::CountSel(Box::new(put)), Cmp::Ge, Value::c(1));
    append(prev, e);
    true
}

inventory::submit! { FollowupPattern { name: "results 5: when you put one or more [permanents] onto the battlefield this way", priority: 0, apply: when_put_onto_battlefield_this_way } }
