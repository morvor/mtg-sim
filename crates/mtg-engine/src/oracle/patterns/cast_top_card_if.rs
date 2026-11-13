//! "At the beginning of your upkeep, you may look at the top card of your library. You may
//! cast it without paying its mana cost if it's an instant or sorcery spell." (Galvanoth):
//! the sentence continues the "look at the top card of your library" instruction; "it" is
//! the card looked at, still on top of the library. It's cast while the ability resolves
//! (CR 608.2g), without paying its mana cost (CR 118.9).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// Whether the effect is "[you may] look at the top card of your library".
fn looks_at_top_card(e: &Effect) -> bool {
    match e {
        Effect::May { effect, .. } => looks_at_top_card(effect),
        Effect::Dig {
            who: PlayerRef::You,
            n: Value::Const(1),
            reveal: false,
            take: Value::Const(0),
            ..
        } => true,
        _ => false,
    }
}

fn cast_it_if(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l)
        .strip_prefix("you may cast it without paying its mana cost if it's ")
        .and_then(|r| r.strip_prefix("an ").or_else(|| r.strip_prefix("a ")))
        .and_then(|r| r.strip_suffix(" spell"))
    else {
        return false;
    };
    if !looks_at_top_card(prev) {
        return false;
    }
    let desc = format!("{r} card");
    let Some((quality, _, tail)) = parse_object_phrase(&desc) else {
        return false;
    };
    if !end(tail).trim().is_empty() {
        return false;
    }
    let top = Sel::TopOfLibrary(PlayerRef::You, Value::c(1));
    *prev = Effect::seq(vec![
        prev.clone(),
        Effect::If {
            cond: Condition::SelMatches(top.clone(), quality),
            then: Box::new(Effect::CastCard {
                who: PlayerRef::You,
                what: top,
                free: true,
                optional: true,
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "look at the top card; you may cast it without paying its mana cost if it's a [quality] spell", priority: 60, apply: cast_it_if } }
