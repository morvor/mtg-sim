//! Oracle patterns for permanents becoming copies of other objects (CR 707.2, 613.2a):
//! "[objects] become(s) a copy of [object][ until end of turn]" (True Polymorph:
//! "Target artifact or creature becomes a copy of another target artifact or creature.").
//! The affected permanents get the other object's copiable values in layer 1 for the
//! duration; everything else about them (designations, counters, status) is unchanged.
//! Copy effects with exceptions ("except it's ...") aren't handled here.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, Builder};
use crate::oracle::phrases::end;

fn becomes_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (duration, l) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (Duration::EndOfTurn, r),
        None => duration_suffix(l),
    };
    if !l.contains(" a copy of ") {
        return None;
    }
    // "that creature" after the verb refers to what an earlier sentence mentioned
    // (Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature ..."), not to the subject of this one.
    let before = b.it.clone();
    let (what, rest) = object_ref(l, b)?;
    let subject = b.it.clone();
    let rest = rest.trim_start();
    let r = ["becomes a copy of ", "become a copy of ", "each become a copy of "]
        .iter()
        .find_map(|p| rest.strip_prefix(p))?;
    b.it = before;
    let parsed = object_ref(r, b);
    b.it = subject;
    let (of, tail) = parsed?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::BecomeCopy { what, of, duration })
}

inventory::submit! { EffectPattern { name: "r707 becomes a copy of", priority: 80, parse: becomes_copy } }
