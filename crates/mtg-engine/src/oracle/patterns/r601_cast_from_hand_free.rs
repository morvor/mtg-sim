//! "You may cast a spell with mana value N or less from your hand without paying its mana
//! cost." (Sram's Expertise and the rest of the Expertise cycle): casting a spell as the
//! spell resolves; the mana value is that of the spell as it would be cast (CR 601.3e,
//! 709.3a). See `kw/cast_from_hand_free.rs`. This form comes before the general "cast a[n]
//! [quality] spell from your hand without paying its mana cost" (`cast_from_hand_free.rs`),
//! which judges the quality on the card in hand: a split card's combined mana value isn't
//! the mana value of the half being cast.

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

inventory::submit! { EffectPattern { name: "you may cast a spell with mana value N or less from your hand without paying its mana cost", priority: 99, parse: cast_from_hand_free } }

/// "Counter target spell. You may cast a spell with equal or lesser mana value from your
/// hand without paying its mana cost." (Reinterpret): at most the mana value of the spell
/// targeted earlier in the text, as it last existed.
fn cast_from_hand_free_equal_or_lesser(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let l = l.strip_prefix("you may ").unwrap_or(l);
    if l != "cast a spell with equal or lesser mana value from your hand without paying its mana cost" {
        return None;
    }
    let slot = b
        .targets
        .iter()
        .rposition(|t| matches!(t.what, TargetKind::Spell(_)))?;
    Some(Effect::Custom(
        crate::kw::cast_from_hand_free::effect_name_target(slot as u8).into(),
    ))
}

inventory::submit! { EffectPattern { name: "you may cast a spell with equal or lesser mana value from your hand without paying its mana cost", priority: 99, parse: cast_from_hand_free_equal_or_lesser } }
