//! "For each of those creatures, create a token that's a copy of that creature." (Hate
//! Mirage), "For each creature target player controls, create a token that's a copy of
//! that creature." (Clone Legion): one token copy of each of the objects (CR 707.2, 111.1).
//! A token copy effect already copies each object it's given, so this is the same as
//! creating a token copy of each of them; later sentences ("Those tokens gain haste.")
//! name the tokens created.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::phrases::*;

fn for_each_create_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (group, clause) = r.split_once(", ")?;
    // The copies: "create a token that's a copy of that creature[, except ...]".
    let copy = clause
        .strip_prefix("create a token that's a copy of ")
        .or_else(|| clause.strip_prefix("create a token that is a copy of "))?;
    // "that creature" or "it": each of the objects in turn.
    let rest = match copy.strip_prefix("it") {
        Some(x) if x.is_empty() || x.starts_with(',') => x,
        _ => {
            let (noun, rest) = split_word(copy.strip_prefix("that ")?);
            if noun.is_empty() {
                return None;
            }
            rest
        }
    };
    // The objects: "of those creatures" (named earlier), or "creature target player
    // controls" (each of them).
    let group = match group.strip_prefix("of ") {
        Some(g) if g.starts_with("those ") => g.to_string(),
        Some(_) => return None,
        None => format!("each {group}"),
    };
    let saved_targets = b.targets.len();
    let saved_it = b.it.clone();
    let (sel, tail) = object_ref(&group, b)?;
    if !end(&tail).trim().is_empty() || matches!(sel, Sel::None) {
        b.targets.truncate(saved_targets);
        return None;
    }
    b.it = sel;
    let e = parse_clause(&format!("create a token that's a copy of it{rest}"), b);
    let ok = matches!(e, Some(Effect::CreateTokenCopy { .. }));
    if !ok {
        b.targets.truncate(saved_targets);
        b.it = saved_it;
        return None;
    }
    e
}

inventory::submit! { EffectPattern { name: "tokens_copies: for each of them, create a token copy", priority: 90, parse: for_each_create_copy } }
