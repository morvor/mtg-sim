//! Power/toughness changes with computed amounts, the "-X/-X" family of removal (and the
//! matching pumps):
//!
//! - "target creature gets -X/-X until end of turn, where X is the number of cards in
//!   your graveyard"
//! - "all creatures get -1/-1 until end of turn for each Swamp you control"
//! - "two target creatures each get -1/-1 until end of turn", "up to two target
//!   creatures each get +1/+1 and gain first strike until end of turn", "they each get"
//!
//! The amount is determined once, as the effect is created (CR 608.2h; the resolver
//! fixes the values), and the affected set is locked in then (CR 611.2c).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, keyword_mods, object_ref, parse_pt_mod, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;

/// Replaces X in a P/T component ("x", "-x") with `with`.
fn subst_x(v: Value, with: &Value) -> Value {
    match v {
        Value::X => with.clone(),
        Value::Diff(a, b) => Value::Diff(Box::new(subst_x(*a, with)), Box::new(subst_x(*b, with))),
        other => other,
    }
}

fn mentions_x(v: &Value) -> bool {
    match v {
        Value::X => true,
        Value::Diff(a, b) => mentions_x(a) || mentions_x(b),
        _ => false,
    }
}

/// "for each [thing]": the number of those things.
fn for_each_value(s: &str, b: &mut Builder) -> Option<Value> {
    let (v, rest) = parse_value_phrase(&format!("the number of {s}"), b)?;
    if !end(&rest).trim().is_empty() {
        return None;
    }
    Some(v)
}

fn p_pt_values(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // ", where x is [value]"
    let (main, where_x) = match l.split_once(", where x is ") {
        Some((m, v)) => {
            let (val, rest) = parse_value_phrase(v, b)?;
            if !end(&rest).trim().is_empty() {
                return None;
            }
            (m, Some(val))
        }
        None => (l, None),
    };
    // "... [until end of turn] for each [thing] [until end of turn]"
    let (mut dur, main) = duration_suffix(main);
    let (main, for_each) = match main.split_once(" for each ") {
        Some((m, f)) => (m, Some(f)),
        None => (main, None),
    };
    let main = if for_each.is_some() && matches!(dur, Duration::Permanent) {
        let (d, m) = duration_suffix(main);
        dur = d;
        m
    } else {
        main
    };
    let (what, rest) = object_ref(main, b)?;
    let rest = rest.trim();
    let (each, rest) = match rest.strip_prefix("each ") {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let r = rest
        .strip_prefix("gets ")
        .or_else(|| rest.strip_prefix("get "))?;
    let (p, t, tail) = parse_pt_mod(r)?;
    if where_x.is_none() && for_each.is_none() && !each {
        // The plain form is handled by the core pump pattern.
        return None;
    }
    let (p, t) = match &where_x {
        Some(x) => {
            if !mentions_x(&p) && !mentions_x(&t) {
                return None;
            }
            (subst_x(p, x), subst_x(t, x))
        }
        None => {
            if mentions_x(&p) || mentions_x(&t) {
                // X without a definition: the spell's X, handled by the core pattern
                // unless combined with "each"/"for each".
                if for_each.is_some() {
                    return None;
                }
            }
            (p, t)
        }
    };
    let (p, t) = match for_each {
        Some(f) => {
            let n = for_each_value(f, b)?;
            let (Some(pc), Some(tc)) = (p.as_const(), t.as_const()) else {
                return None;
            };
            (
                Value::Mul(Box::new(Value::c(pc)), Box::new(n.clone())),
                Value::Mul(Box::new(Value::c(tc)), Box::new(n)),
            )
        }
        None => (p, t),
    };
    let mut mods = vec![Modification::ModifyPT(p, t)];
    let tail = tail.trim();
    if !tail.is_empty() {
        let k = tail
            .strip_prefix("and gains ")
            .or_else(|| tail.strip_prefix("and gain "))?;
        mods.extend(keyword_mods(k)?);
    }
    Some(Effect::Modify {
        what,
        mods,
        duration: dur,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: P/T change with X / for each / each", priority: 60, parse: p_pt_values } }
