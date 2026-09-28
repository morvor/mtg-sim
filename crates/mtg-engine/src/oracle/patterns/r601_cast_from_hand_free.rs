//! "You may cast a spell with mana value N or less from your hand without paying its mana
//! cost." (Sram's Expertise and the rest of the Expertise cycle): casting a spell as the
//! spell resolves; the mana value is that of the spell as it would be cast (CR 601.3e,
//! 709.3a). See `kw/cast_from_hand_free.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_from_hand_free::effect_name;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn cast_from_hand_free(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let n = l
        .strip_prefix("cast a spell with mana value ")?
        .strip_suffix(" or less from your hand without paying its mana cost")?;
    let max: u32 = n.parse().ok()?;
    Some(Effect::Custom(effect_name(max).into()))
}

inventory::submit! { EffectPattern { name: "you may cast a spell with mana value N or less from your hand without paying its mana cost", priority: 100, parse: cast_from_hand_free } }
