//! "Until end of turn, [object] becomes [color] and isn't a[n] [card type]" (Neurok
//! Transmuter: "Until end of turn, target artifact creature becomes blue and isn't an
//! artifact."), also with the duration last. The new color overwrites the old ones (CR
//! 105.3); losing a card type loses the subtypes of that type (CR 205.1b), not other
//! types, subtypes, or supertypes.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::types::{CardType, Color, ColorSet};

fn becomes_color_isnt_type(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let body = l
        .strip_prefix("until end of turn, ")
        .or_else(|| l.strip_suffix(" until end of turn"))?;
    let (subj, rest) = body.split_once(" becomes ")?;
    let (color, ty) = rest.split_once(" and isn't ")?;
    let color = Color::from_word(color)?;
    let ty = ty.strip_prefix("an ").or_else(|| ty.strip_prefix("a "))?;
    let ty = CardType::from_word(ty)?;
    let (what, r) = object_ref(subj, b)?;
    if !r.trim().is_empty() || matches!(what, Sel::None) {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![
            Modification::SetColors(ColorSet::single(color)),
            Modification::RemoveTypes(vec![ty]),
        ],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "until end of turn, [object] becomes [color] and isn't a [type]", priority: 60, parse: becomes_color_isnt_type } }
