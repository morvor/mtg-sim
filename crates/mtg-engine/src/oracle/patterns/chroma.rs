//! Chroma (an ability word, CR 207.2c): counts of the mana symbols of a color in mana
//! costs. "The number of [color] mana symbols in the mana costs of permanents you control"
//! is that player's devotion to the color (CR 700.5): hybrid mana symbols of the color
//! count (CR 107.4e), mana symbols in text boxes don't, and the source's own mana cost
//! counts while it's a permanent the player controls.

use crate::ability::*;
use crate::oracle::phrases::split_word;
use crate::types::{Color, ColorSet};

/// "[color] mana symbols in the mana costs of permanents you control" (after "the number
/// of", or after "for each" in the singular), and the rest of the text.
pub(crate) fn mana_symbols_among_your_permanents(r: &str) -> Option<(Value, &str)> {
    let (w, rest) = split_word(r.trim_start());
    let color = Color::from_word(w)?;
    let rest = rest
        .strip_prefix("mana symbols in the mana costs of permanents you control")
        .or_else(|| rest.strip_prefix("mana symbol in the mana costs of permanents you control"))?;
    Some((Value::Devotion(ColorSet::single(color)), rest))
}
