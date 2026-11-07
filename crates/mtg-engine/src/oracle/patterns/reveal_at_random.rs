//! "Target opponent reveals a card at random from their hand." (Planeswalker's Favor,
//! Planeswalker's Scorn): see `kw/reveal_at_random.rs`. "The revealed card" is "it" for
//! the rest of the effect.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::reveal_at_random::effect_name;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn reveal_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let who = if l == "reveal a card at random from your hand" {
        PlayerRef::You
    } else {
        let subject = l.strip_suffix(" reveals a card at random from their hand")?;
        if !subject.starts_with("target ") {
            return None;
        }
        let (who, rest) = player_ref(subject, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        who
    };
    let name = effect_name(&who)?;
    b.it = Sel::Var(vars::IT);
    Some(Effect::Custom(SmolStr::new(name)))
}

inventory::submit! { EffectPattern { name: "reveal a card at random from [a] hand", priority: 100, parse: reveal_at_random } }
