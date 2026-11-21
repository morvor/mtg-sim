//! "You may cast that card by paying life equal to the spell's mana value rather than
//! paying its mana cost." (Bismuth Mindrender): after exiling cards until one was found
//! ("that card"), a spell cast as the ability resolves for an alternative cost of life.
//! See `kw/cast_it_paying_life.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_it_paying_life::CAST_IT_PAYING_LIFE;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn cast_it_paying_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    // "You may": the player chooses whether to cast it.
    let (may, l) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    if !matches!(
        l,
        "cast that card by paying life equal to the spell's mana value rather than paying its mana cost"
            | "cast that card by paying life equal to its mana value rather than paying its mana cost"
    ) {
        return None;
    }
    // "That card" is one an earlier instruction found.
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return None;
    }
    let e = Effect::Custom(CAST_IT_PAYING_LIFE.into());
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

inventory::submit! { EffectPattern { name: "you may cast that card by paying life equal to its mana value", priority: 100, parse: cast_it_paying_life } }
