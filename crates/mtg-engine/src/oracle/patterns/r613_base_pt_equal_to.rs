//! Base power and toughness that become equal to another creature's (layer 7b, CR 613.4b):
//! "Its base power and toughness become equal to ~'s power and toughness until end of
//! turn." (Galion, Elvenking's Butler) and "you may have the base power and toughness of
//! other creatures you control become equal to ~'s power and toughness until end of
//! turn" (Tanazir Quandrix). The values are determined once, as the effect begins
//! (CR 608.2h), and the affected set is locked in then (CR 611.2c).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "equal to ~'s power and toughness until end of turn" → the values.
fn equal_to_source(r: &str) -> Option<(Value, Value)> {
    (end(r) == "equal to ~'s power and toughness until end of turn").then(|| {
        (
            Value::PowerOf(Box::new(Sel::This)),
            Value::ToughnessOf(Box::new(Sel::This)),
        )
    })
}

fn modify(what: Sel, (p, t): (Value, Value)) -> Effect {
    Effect::Modify {
        what,
        mods: vec![Modification::SetPT(Some(p), Some(t))],
        duration: Duration::EndOfTurn,
    }
}

/// "its base power and toughness become equal to ~'s power and toughness until end of
/// turn", where "its" is a target or the trigger's object.
fn its_base_pt_become(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("its base power and toughness become ")?;
    let pt = equal_to_source(r)?;
    if !matches!(b.it, Sel::Target(_) | Sel::TriggerObject) {
        return None;
    }
    Some(modify(b.it.clone(), pt))
}

/// "have the base power and toughness of [objects] become equal to ~'s power and
/// toughness until end of turn" (after "you may").
fn have_base_pt_of_become(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("have the base power and toughness of ")?;
    let (objects, r) = r.split_once(" become ")?;
    let pt = equal_to_source(r)?;
    let (f, true, tail) = parse_object_phrase(objects)? else {
        return None;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    Some(modify(Sel::All(f), pt))
}

inventory::submit! { EffectPattern { name: "its base power and toughness become equal to ~'s", priority: 100, parse: its_base_pt_become } }
inventory::submit! { EffectPattern { name: "have the base power and toughness of [objects] become equal to ~'s", priority: 100, parse: have_base_pt_of_become } }
