//! Results of an earlier instruction in older wordings ("this way"):
//!
//! - the card's own name as the subject: "If ~ deals damage to you this way, tap it."
//!   (Minion of Leshrac; the "it" form is Mishra's War Machine's, `unless_grammar`), "If ~
//!   is destroyed this way, it deals 7 damage to you." (Cosmic Horror: only if it was
//!   actually destroyed, not regenerated or indestructible: its ruling);
//! - an exchange: "If those permanents are exchanged this way, destroy all Auras attached
//!   to them." (Gauntlets of Chaos: CR 701.12b, an exchange that can't happen doesn't);
//! - a sacrifice that didn't happen: "If you don't sacrifice a Forest, ..." (Gargantuan
//!   Gorilla, after "you may sacrifice a Forest").

use super::{ConditionPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "If ~ deals damage to you this way, [effect]": the card's name for "it".
fn named_subject_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ~ deals damage to you this way, ") else {
        return false;
    };
    let text = format!("if it deals damage to you this way, {r}");
    crate::oracle_ext::apply_followup_ext(&text, prev, b)
}

inventory::submit! { FollowupPattern { name: "if ~ deals damage to you this way, [effect]", priority: 141, apply: named_subject_this_way } }

/// "~ is destroyed this way": the latest destroy instruction destroyed the source (a
/// regenerated or indestructible permanent wasn't destroyed, CR 701.8a-c).
fn source_destroyed_this_way(c: &str) -> Option<Condition> {
    let c = end(c);
    if c != "~ is destroyed this way" && c != "~ was destroyed this way" {
        return None;
    }
    Some(Condition::SelNonEmpty(Sel::Matching(
        Box::new(Sel::Var(crate::kw::value_results::DESTROYED)),
        Filter::Source,
    )))
}

inventory::submit! { ConditionPattern { name: "~ is destroyed this way", priority: 100, parse: source_destroyed_this_way } }

/// Whether an effect ends with an exchange of control.
fn ends_with_exchange(e: &Effect) -> bool {
    match e {
        Effect::ExchangeControl { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exchange),
        _ => false,
    }
}

/// "If those permanents are exchanged this way, [effect]" after an exchange of control:
/// only if the exchange happened (CR 701.12b).
fn exchanged_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if those ") else {
        return false;
    };
    let Some((noun, x)) = r.split_once(" are exchanged this way, ") else {
        return false;
    };
    if !matches!(noun, "permanents" | "creatures" | "artifacts" | "lands")
        || !ends_with_exchange(prev)
    {
        return false;
    }
    // "them": the exchanged permanents (the targets; a control change doesn't make
    // new objects).
    let saved = b.it.clone();
    b.it = Sel::AllTargets;
    let x = x.replace(" attached to them", " attached to it");
    let parsed = parse_clause(&x, b);
    b.it = saved;
    let Some(e) = parsed else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if those permanents are exchanged this way, [effect]", priority: 141, apply: exchanged_this_way } }

/// "you don't sacrifice a Forest" (after an instruction that let you sacrifice one): no
/// permanent of that kind was sacrificed this way.
fn didnt_sacrifice(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("you don't sacrifice ")
        .or_else(|| c.strip_prefix("you didn't sacrifice "))?;
    let r = r.strip_suffix(" this way").unwrap_or(r);
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::CountSel(Box::new(Sel::Matching(
            Box::new(Sel::Var(vars::SACRIFICED)),
            f,
        ))),
        Cmp::Eq,
        Value::c(0),
    ))
}

inventory::submit! { ConditionPattern { name: "you don't sacrifice a [permanent]", priority: 100, parse: didnt_sacrifice } }

/// Whether an effect lets you sacrifice permanents ("you may sacrifice a Forest").
fn sacrifices(e: &Effect) -> bool {
    match e {
        Effect::Sacrifice {
            who: PlayerRef::You,
            ..
        } => true,
        Effect::Seq(v) => v.iter().any(sacrifices),
        Effect::May { effect, .. } => sacrifices(effect),
        _ => false,
    }
}

/// Whether an effect is "If you sacrificed [a permanent] this way, ..." (so the sentence
/// before it sacrificed).
fn asks_sacrificed(e: &Effect) -> bool {
    match e {
        Effect::If {
            cond: Condition::Compare(Value::CountSel(sel), _, _),
            ..
        } => matches!(&**sel, Sel::Matching(v, _) if matches!(**v, Sel::Var(vars::SACRIFICED))),
        Effect::Seq(v) => v.last().is_some_and(asks_sacrificed),
        _ => false,
    }
}

/// "If you don't sacrifice a Forest, [effect]" after "you may sacrifice a Forest. If you
/// sacrifice a snow Forest this way, ...": about the sacrifice, not about the condition of
/// the sentence in between (which an "if you don't [verb]" after "[instruction] if
/// [condition]" would otherwise be read as the alternative of).
fn if_you_dont_sacrifice(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.starts_with("if you don't sacrifice ") || !(sacrifices(prev) || asks_sacrificed(prev)) {
        return false;
    }
    let Some((c, x)) = l["if ".len()..].split_once(", ") else {
        return false;
    };
    let Some(cond) = didnt_sacrifice(c) else {
        return false;
    };
    let Some(e) = parse_clause(x, b) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if you don't sacrifice a [permanent], [effect]", priority: 140, apply: if_you_dont_sacrifice } }
