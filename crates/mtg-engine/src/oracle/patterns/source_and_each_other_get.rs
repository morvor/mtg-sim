//! "~ and each other creature with the same name as it get +3/+3 until end of turn."
//! (Cylian Sunsinger): the source and the other objects get the same bonus. The set of
//! objects is determined as the effect begins (CR 611.2c), using the source's name at that
//! time (CR 201.2a, 608.2h).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

fn source_and_each_other_get(s: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(s);
    let r = l.strip_prefix("~ and each other ")?;
    let (objects, pump) = r.split_once(" get ")?;
    // "with the same name as it": "it" is the source, the sentence's first subject.
    let objects = objects.replace(" as it", " as ~");
    let e = parse_sentence(&format!("each other {objects} gets {pump}"), b)?;
    let Effect::Modify {
        what: Sel::All(f),
        mods,
        duration,
    } = e
    else {
        return None;
    };
    // Only "other" objects, so the source isn't counted twice.
    if !matches!(&f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::Other))) {
        return None;
    }
    Some(Effect::Modify {
        what: Sel::Union(vec![Sel::This, Sel::All(f)]),
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "~ and each other [objects] get +N/+N", priority: 100, parse: source_and_each_other_get } }
