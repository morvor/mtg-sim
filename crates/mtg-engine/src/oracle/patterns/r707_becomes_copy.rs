//! Oracle patterns for permanents becoming copies of other objects (CR 707.2, 613.2a):
//! "[objects] become(s) a copy of [object][ until end of turn]" (True Polymorph:
//! "Target artifact or creature becomes a copy of another target artifact or creature.").
//! The affected permanents get the other object's copiable values in layer 1 for the
//! duration; everything else about them (designations, counters, status) is unchanged.
//! Exceptions ("..., except those creatures aren't legendary", CR 707.9) are parsed like
//! a token copy's (see `tokens_copies_copy::copy_exceptions`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, Builder};
use crate::oracle::phrases::end;

fn becomes_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" a copy of ") {
        return None;
    }
    // "..., except those creatures aren't legendary" (Echoing Equation).
    let (l, exceptions) = match l.split_once(", except ") {
        Some((head, except)) => {
            // Quoted abilities ("it has this ability and \"...\"", Aurora Shifter).
            let (masked, quotes) = super::statics::mask_quotes(except)?;
            (
                head,
                super::tokens_copies_copy::copy_exceptions(&masked, &quotes, b.ctx)?,
            )
        }
        None => (l, vec![]),
    };
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
    // "Each other creature you control becomes a copy of it": other than the object
    // copied (the chosen or targeted creature), not only other than the source.
    let what = match what {
        Sel::All(f) => Sel::All(other_than(f, &of)),
        w => w,
    };
    Some(if exceptions.is_empty() {
        Effect::BecomeCopy { what, of, duration }
    } else {
        Effect::BecomeCopyExcept {
            what,
            of,
            duration,
            exceptions,
        }
    })
}

/// Replaces "other" in the description of the objects becoming copies with "other than
/// the copied object".
fn other_than(f: Filter, of: &Sel) -> Filter {
    match f {
        Filter::Other => Filter::not(Filter::In(Box::new(of.clone()))),
        Filter::And(v) => Filter::And(v.into_iter().map(|x| other_than(x, of)).collect()),
        x => x,
    }
}

inventory::submit! { EffectPattern { name: "r707 becomes a copy of", priority: 80, parse: becomes_copy } }
