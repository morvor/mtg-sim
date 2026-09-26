//! "[Effect with an amount] for each [thing]": the amount is multiplied by the number of
//! those things, counted once as the effect happens (CR 608.2h).
//!
//! - "~ deals 1 damage to each opponent for each creature you control"
//! - "You gain 1 life for each creature you control", "Draw a card for each creature you
//!   control with power 4 or greater", "Create a 1/1 ... token for each Elf you control"
//! - After a destroy effect: "You gain 2 life for each creature destroyed this way",
//!   "Draw a card for each creature destroyed this way" (the permanents that effect
//!   actually destroyed).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;
use crate::types::CardType;

/// Multiplies the amount of an effect by `count`. Only effects with a single constant
/// amount qualify.
fn multiply(e: Effect, count: Value) -> Option<Effect> {
    let times = |n: &Value| -> Option<Value> {
        let k = n.as_const()?;
        Some(if k == 1 {
            count.clone()
        } else {
            Value::Mul(Box::new(Value::c(k)), Box::new(count.clone()))
        })
    };
    Some(match e {
        Effect::GainLife { who, n } => Effect::GainLife { who, n: times(&n)? },
        Effect::LoseLife { who, n } => Effect::LoseLife { who, n: times(&n)? },
        Effect::Draw { who, n } => Effect::Draw { who, n: times(&n)? },
        Effect::Mill { who, n } => Effect::Mill { who, n: times(&n)? },
        Effect::AddCounters { what, kind, n } => Effect::AddCounters {
            what,
            kind,
            n: times(&n)?,
        },
        Effect::AddPlayerCounters { who, kind, n } => Effect::AddPlayerCounters {
            who,
            kind,
            n: times(&n)?,
        },
        // "Add {C} for each charge counter on ~": that much mana of one type.
        Effect::AddMana {
            who,
            mana: ManaProduction::Fixed(syms),
            restriction,
        } if syms.len() == 1 => Effect::AddMana {
            who,
            mana: ManaProduction::Amount(syms[0], count),
            restriction,
        },
        Effect::DealDamage { source, amount, to } => Effect::DealDamage {
            source,
            amount: times(&amount)?,
            to,
        },
        Effect::CreateToken {
            spec,
            count: c,
            controller,
            tapped,
            attacking,
        } => Effect::CreateToken {
            spec,
            count: times(&c)?,
            controller,
            tapped,
            attacking,
        },
        _ => return None,
    })
}

/// "the number of [thing]" for "for each [thing]".
fn count_of(s: &str, b: &mut Builder) -> Option<Value> {
    let (v, rest) = parse_value_phrase(&format!("the number of {s}"), b)?;
    if !end(&rest).trim().is_empty() {
        return None;
    }
    Some(v)
}

fn p_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let (clause, thing) = l.rsplit_once(" for each ")?;
    if thing.ends_with("destroyed this way") || thing.contains(" this way") {
        return None;
    }
    let count = count_of(thing, b)?;
    let e = parse_clause(clause, b)?;
    multiply(e, count)
}

inventory::submit! { EffectPattern { name: "damage_removal: [amount] for each [thing]", priority: 70, parse: p_for_each } }

/// Whether every object matching `f` has card type `t` (a conjunction containing it).
fn implies_type(f: &Filter, t: CardType) -> bool {
    match f {
        Filter::Type(x) => *x == t,
        Filter::And(v) => v.iter().any(|x| implies_type(x, t)),
        Filter::Or(v) => !v.is_empty() && v.iter().all(|x| implies_type(x, t)),
        _ => false,
    }
}

/// The last destroy effect of the previous sentence and what it destroys.
fn last_destroy(e: &Effect) -> Option<&Sel> {
    match e {
        Effect::Seq(v) => v.last().and_then(last_destroy),
        Effect::Destroy { what, .. } => Some(what),
        _ => None,
    }
}

/// "[effect] for each [creature] destroyed this way" after a destroy effect.
fn f_destroyed_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some((clause, thing)) = l.rsplit_once(" for each ") else {
        return false;
    };
    let Some(noun) = thing.strip_suffix(" destroyed this way") else {
        return false;
    };
    // Every destroyed object must be of the counted kind.
    let ok = match last_destroy(prev) {
        None => false,
        Some(_) if noun == "permanent" => true,
        Some(Sel::All(f)) => CardType::from_word(noun).is_some_and(|t| implies_type(f, t)),
        Some(_) => false,
    };
    if !ok {
        return false;
    }
    let Some(e) = parse_clause(clause, b) else {
        return false;
    };
    // The destroy effect stores the permanents it destroyed in "it".
    let Some(e) = multiply(e, Value::CountSel(Box::new(Sel::Var(vars::IT)))) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "damage_removal: for each destroyed this way", priority: 55, apply: f_destroyed_this_way } }
