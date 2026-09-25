//! Oracle patterns for copies of cards (CR 707.12, 707.13):
//!
//! - "Choose a card name that hasn't been chosen from among A, B, and C. Create a copy of
//!   the card with the chosen name." (Garth One-Eye, CR 707.13)
//! - "(You may) cast the copy [without paying its mana cost]." (CR 707.12)

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::copy_rules::NamedCopy;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use smol_str::SmolStr;

/// "a, b, and c" → [a, b, c].
fn name_list(s: &str) -> Vec<SmolStr> {
    s.replace(", and ", ", ")
        .replace(" and ", ", ")
        .split(", ")
        .map(|n| SmolStr::new(n.trim()))
        .filter(|n| !n.is_empty())
        .collect()
}

fn p_choose_name_from(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (unchosen, r) =
        if let Some(r) = l.strip_prefix("choose a card name that hasn't been chosen from among ") {
            (true, r)
        } else {
            (false, l.strip_prefix("choose a card name from among ")?)
        };
    let names = name_list(r);
    if names.len() < 2 {
        return None;
    }
    // The copy is made by the next sentence; the choice is made as part of it.
    Some(Effect::CopyCard {
        what: Sel::None,
        named: Some(NamedCopy::OneOf { names, unchosen }),
    })
}

inventory::submit! { EffectPattern { name: "r707 choose a card name from among", priority: 0, parse: p_choose_name_from } }

/// "Create a copy of the card with the chosen name." after choosing a name.
fn f_copy_chosen(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    l == "create a copy of the card with the chosen name"
        && matches!(
            prev,
            Effect::CopyCard {
                named: Some(NamedCopy::OneOf { .. }),
                ..
            }
        )
}

inventory::submit! { FollowupPattern { name: "r707 create a copy of the chosen card", priority: 0, apply: f_copy_chosen } }

/// "You may cast the copy [without paying its mana cost]."
fn p_cast_the_copy(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (optional, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let r = r
        .strip_prefix("cast the copy")
        .or_else(|| r.strip_prefix("cast the copies"))?;
    let free = match r.trim() {
        "" => false,
        "without paying its mana cost" | "without paying their mana costs" => true,
        _ => return None,
    };
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what: Sel::Var(vars::CREATED),
        free,
        optional,
    })
}

inventory::submit! { EffectPattern { name: "r707 cast the copy", priority: 0, parse: p_cast_the_copy } }
