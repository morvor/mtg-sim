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
pub(crate) fn multiply(e: Effect, count: Value) -> Option<Effect> {
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
        // "Target player discards a card for each Swamp you control".
        Effect::Discard {
            who,
            n,
            random,
            filter,
        } => Effect::Discard {
            who,
            n: times(&n)?,
            random,
            filter,
        },
        // "Target opponent sacrifices a creature of their choice for each ...": that many
        // at once.
        Effect::Sacrifice { who, filter, count: c } => Effect::Sacrifice {
            who,
            filter,
            count: times(&c)?,
        },
        Effect::RemoveCounters { what, kind, n } => Effect::RemoveCounters {
            what,
            kind,
            n: times(&n)?,
        },
        Effect::Scry { who, n } => Effect::Scry { who, n: times(&n)? },
        Effect::Surveil { who, n } => Effect::Surveil { who, n: times(&n)? },
        // "investigate for each goaded creature you control" (CR 701.16a: investigate
        // that many times).
        Effect::KeywordAction {
            action: KeywordAction::Investigate,
            who,
            what,
            n,
        } => Effect::KeywordAction {
            action: KeywordAction::Investigate,
            who,
            what,
            n: times(&n)?,
        },
        // "Target creature gets +1/+1 until end of turn for each of its colors": the
        // bonus multiplied (determined once, CR 608.2h).
        Effect::Modify {
            what,
            mods,
            duration,
        } if mods.len() == 1 && matches!(mods[0], Modification::ModifyPT(..)) => {
            let Modification::ModifyPT(p, t) = &mods[0] else {
                return None;
            };
            Effect::Modify {
                what,
                mods: vec![Modification::ModifyPT(times(p)?, times(t)?)],
                duration,
            }
        }
        Effect::CreateTokenWithPT {
            spec,
            power,
            toughness,
            count: c,
            controller,
            tapped,
            attacking,
        } => Effect::CreateTokenWithPT {
            spec,
            power,
            toughness,
            count: times(&c)?,
            controller,
            tapped,
            attacking,
        },
        // "Exile the top card of your library for each [thing]".
        Effect::Exile {
            what: Sel::TopOfLibrary(who, n),
            face_down,
            link,
        } => Effect::Exile {
            what: Sel::TopOfLibrary(who, times(&n)?),
            face_down,
            link,
        },
        _ => return None,
    })
}

/// "the number of [thing]" for "for each [thing]".
fn count_of(s: &str, b: &mut Builder) -> Option<Value> {
    // "for each of those creatures": the objects an earlier instruction named (Bounding
    // Felidar).
    if let Some(r) = s.strip_prefix("of those ") {
        let saved = b.targets.len();
        let (sel, rest) = crate::oracle::effects::object_ref(&format!("those {r}"), b)?;
        if !end(&rest).trim().is_empty() || b.targets.len() != saved {
            b.targets.truncate(saved);
            return None;
        }
        return Some(Value::CountSel(Box::new(sel)));
    }
    // "for each of its colors", "for each of that spell's colors": how many colors it
    // has (CR 105.2).
    if let Some(r) = s.strip_prefix("of ") {
        let r = end(r);
        let who = r
            .strip_suffix("'s colors")
            .or_else(|| (r == "its colors").then_some("it"))?;
        let saved = b.targets.len();
        let (sel, rest) = crate::oracle::effects::object_ref(who, b)?;
        if !rest.trim().is_empty() || b.targets.len() != saved {
            b.targets.truncate(saved);
            return None;
        }
        return Some(Value::DistinctAmong(Among::Colors, Box::new(sel)));
    }
    let (v, rest) = parse_value_phrase(&format!("the number of {s}"), b)?;
    if !end(&rest).trim().is_empty() {
        return None;
    }
    Some(v)
}

fn p_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let (clause, thing) = l.rsplit_once(" for each ")?;
    // "for each creature card milled this way" counts the cards the preceding mill
    // instruction milled (CR 701.17c); other "this way" counts are handled elsewhere.
    if thing.ends_with("destroyed this way")
        || (thing.contains(" this way") && !end(thing).ends_with(" milled this way"))
    {
        return None;
    }
    // The instruction first: what it names is what a pronoun in the counted phrase
    // refers to ("Target creature gets +1/+1 until end of turn for each of its colors",
    // "Each opponent ... for each creature they control"), and its targets come first.
    let (saved_targets, saved_it, saved_player) = (b.targets.len(), b.it.clone(), b.it_player.clone());
    if let Some(e) = parse_clause(clause, b) {
        // "Each creature your opponents control gets -1/-1 ... for each poison counter its
        // controller has": "its" is each affected object in turn (CR 608.2h).
        let it = b.it.clone();
        if matches!(&e, Effect::Modify { what: Sel::All(_), .. }) {
            b.it = Sel::Var(vars::AFFECTED);
        }
        let count = count_of(thing, b);
        b.it = it;
        if let Some(count) = count {
            if let Some(e) = multiply(e, count) {
                return Some(e);
            }
        }
    }
    b.targets.truncate(saved_targets);
    b.it = saved_it;
    b.it_player = saved_player;
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
