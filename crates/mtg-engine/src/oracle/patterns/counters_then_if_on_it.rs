//! "Put a charge counter on ~. Then if there are four or more charge counters on it, you
//! may remove those counters and transform it." (Primal Amulet), "Remove an ice counter
//! from ~. Then if it has no ice counters on it, transform it." (Thing in the Ice), "...
//! Then if there are seven or more time counters on it, remove those counters and it deals
//! 20 damage to you." (Cursed Recording): after an instruction that changes the counters
//! on the source, "it" is the source (even in an ability that triggers on casting a
//! spell, where "it" would otherwise be the spell). The condition is checked as that part
//! of the effect happens (CR 608.2c); "those counters" are the counters of that kind on it.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::types::CounterKind;

/// The kind of counter the effect's last instruction put on or removed from the source.
fn last_counter_change(e: &Effect) -> Option<&CounterKind> {
    match e {
        Effect::Seq(v) => v.last().and_then(last_counter_change),
        Effect::AddCounters {
            what: Sel::This,
            kind,
            ..
        }
        | Effect::RemoveCounters {
            what: Sel::This,
            kind: Some(kind),
            ..
        } => Some(kind),
        _ => None,
    }
}

fn then_if_counters_on_it(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(changed) = last_counter_change(prev).cloned() else {
        return false;
    };
    let lower = s.to_lowercase();
    let l = end(&lower);
    let Some(r) = l.strip_prefix("then if ") else {
        return false;
    };
    let Some((cond_s, clause)) = r.split_once(", ") else {
        return false;
    };
    // "it has N or more [kind] counters on it", "there are N or more [kind] counters on
    // it", "it has no [kind] counters on it".
    let Some(c) = cond_s
        .strip_prefix("it has ")
        .or_else(|| cond_s.strip_prefix("there are "))
        .or_else(|| cond_s.strip_prefix("there is "))
    else {
        return false;
    };
    let Some((cmp, n, c)) = super::counters_resources_counters::amount_cmp(c) else {
        return false;
    };
    let Some((kind, c)) = super::counters_resources_counters::kind_then_on(c) else {
        return false;
    };
    if end(c) != "it" || kind.as_ref().is_some_and(|k| *k != changed) {
        return false;
    }
    let kind = kind.unwrap_or(changed);
    let cond = Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(kind.clone())),
        cmp,
        n,
    );
    // A later "if you do" would need to know whether a sacrifice here happened.
    if clause.starts_with("sacrifice ") {
        return false;
    }
    let (optional, clause) = match clause.strip_prefix("you may ") {
        Some(c) => (true, c),
        None => (false, clause),
    };
    // "remove those counters [and ...]": all the counters of that kind on it.
    let remove = Effect::RemoveCounters {
        what: Sel::This,
        kind: Some(kind.clone()),
        n: Value::CountersOn(Box::new(Sel::This), Some(kind)),
    };
    let (first, rest) = if let Some(r) = clause.strip_prefix("remove those counters and ") {
        (Some(remove), r)
    } else if clause == "remove those counters" {
        (Some(remove), "")
    } else {
        (None, clause)
    };
    // "it deals 20 damage to you", "transform it": the source.
    let rest = match rest.strip_prefix("it ") {
        Some(r) => format!("~ {r}"),
        None if rest == "transform it" => "transform ~".to_string(),
        None => rest.to_string(),
    };
    let saved_it = b.it.clone();
    let saved_targets = b.targets.len();
    b.it = Sel::This;
    let parsed = if rest.is_empty() {
        Some(Effect::Noop)
    } else {
        parse_clause(&rest, b)
    };
    let Some(then) = parsed else {
        b.it = saved_it;
        b.targets.truncate(saved_targets);
        return false;
    };
    let then = match first {
        Some(f) if matches!(then, Effect::Noop) => f,
        Some(f) => Effect::Seq(vec![f, then]),
        None => then,
    };
    let then = if optional {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(then),
        }
    } else {
        then
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "counters: then if there are N counters on it", priority: 60, apply: then_if_counters_on_it } }
