//! "[Subject] also [does something]" (e.g. Archon's Glory: "If this spell was bargained,
//! that creature also gains flying and lifelink until end of turn."): "also" only says the
//! effect is in addition to what the text did before; the clause means the same without
//! it. Tried after every other effect pattern.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn without_also(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // Only "also" right after the subject, before the verb: "that creature also gains",
    // "those creatures also gain", "it also deals", "you also gain".
    let i = l.find(" also ")?;
    let (subject, rest) = (&l[..i], &l[i + " also ".len()..]);
    if subject.is_empty() || subject.contains(',') || rest.contains(" also ") {
        return None;
    }
    let verb = rest.split_whitespace().next()?;
    if !matches!(
        verb,
        "gains" | "gain" | "gets" | "get" | "has" | "have" | "deals" | "deal" | "loses" | "lose"
    ) {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("{subject} {rest}"), b)
}

inventory::submit! { EffectPattern { name: "[subject] also [verb]", priority: 1000, parse: without_also } }
