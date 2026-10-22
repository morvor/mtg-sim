//! Oracle patterns about battles' protectors (CR 310.9e: the player who protects a battle
//! is its protector): "Choose target battle. If an opponent protects it, remove a defense
//! counter from it. Otherwise, put a defense counter on it." (Portent Tracker).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::battle::{PROTECTED_BY_OPPONENT, PROTECTED_BY_YOU};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number, parse_target};

/// "choose target battle": the target is chosen; the following sentences refer to it.
fn choose_target_battle(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    if !r.starts_with("target battle") {
        return None;
    }
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let slot = b.add_target(spec, r);
    b.it = Sel::Target(slot);
    Some(Effect::Noop)
}

/// "remove three defense counters from it" / "put a defense counter on it", where "it" is
/// the chosen target battle.
fn defense_counters_on_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let what = match b.it {
        Sel::Target(_) => b.it.clone(),
        _ => return None,
    };
    let kind: crate::types::CounterKind = crate::types::counters::DEFENSE.into();
    if let Some(r) = l.strip_prefix("remove ") {
        let (n, r) = parse_number(r)?;
        let r = r.trim_start();
        if r != "defense counter from it" && r != "defense counters from it" {
            return None;
        }
        return Some(Effect::RemoveCounters {
            what,
            kind: Some(kind),
            n,
        });
    }
    let r = l.strip_prefix("put ")?;
    let (n, r) = parse_number(r)?;
    let r = r.trim_start();
    if r != "defense counter on it" && r != "defense counters on it" {
        return None;
    }
    Some(Effect::AddCounters { what, kind, n })
}

/// The protector condition "an opponent protects it" / "you protect it".
fn protects(c: &str, b: &Builder) -> Option<Condition> {
    let filter = match c {
        "an opponent protects it" => PROTECTED_BY_OPPONENT,
        "you protect it" => PROTECTED_BY_YOU,
        _ => return None,
    };
    if matches!(b.it, Sel::None) {
        return None;
    }
    Some(Condition::SelMatches(
        b.it.clone(),
        Filter::Custom(filter.into()),
    ))
}

/// "if an opponent protects it, [effect]"
fn if_protects(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let cond = protects(c, b)?;
    let then = parse_clause(rest, b)?;
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

/// "Otherwise, [effect]." after "If an opponent protects it, [effect]."
fn otherwise_protects(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("otherwise, ") else {
        return false;
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::If {
        cond: Condition::SelMatches(_, Filter::Custom(name)),
        otherwise,
        ..
    }) = last
    else {
        return false;
    };
    if !matches!(name.as_str(), PROTECTED_BY_OPPONENT | PROTECTED_BY_YOU)
        || !matches!(**otherwise, Effect::Noop)
    {
        return false;
    }
    let Some(e) = parse_clause(r, b) else {
        return false;
    };
    **otherwise = e;
    true
}

inventory::submit! { EffectPattern { name: "r310 choose target battle", priority: 100, parse: choose_target_battle } }
inventory::submit! { EffectPattern { name: "r310 defense counters on it", priority: 100, parse: defense_counters_on_it } }
inventory::submit! { EffectPattern { name: "r310 if an opponent protects it", priority: 100, parse: if_protects } }
inventory::submit! { FollowupPattern { name: "r310 otherwise (protector)", priority: 100, apply: otherwise_protects } }
