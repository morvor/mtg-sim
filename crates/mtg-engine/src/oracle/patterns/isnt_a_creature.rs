//! "[object] isn't a creature until end of turn" (the Alien Angel tokens of Aplan
//! Mortarium and Blink: "Whenever an opponent casts a creature spell, this token isn't a
//! creature until end of turn."): a one-shot effect that removes the creature card type
//! for a duration (CR 611.2). With it go the creature types (CR 205.3d), and a permanent
//! that stops being a creature is removed from combat (CR 506.4).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::types::CardType;

fn isnt_a_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l)
        .strip_suffix(" isn't a creature until end of turn")
        .or_else(|| end(l).strip_suffix(" aren't creatures until end of turn"))?;
    let (what, rest) = object_ref(subj, b)?;
    if !rest.trim().is_empty() || matches!(what, Sel::None) {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "isn't a creature until end of turn", priority: 60, parse: isnt_a_creature } }
