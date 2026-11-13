//! CR 701.7 Create. Creating tokens puts the specified number of tokens with the
//! specified characteristics onto the battlefield (CR 701.7a). A replacement effect that
//! applies to tokens being created considers the characteristics they're created with,
//! before any continuous effect that will modify them (CR 701.7b): e.g. a Treasure is
//! created as a noncreature artifact token even if an effect will make it a creature.

use crate::ability::*;
use crate::object::Characteristics;

/// Whether tokens created with `chars` match `f` (types, subtypes, supertypes, colors).
/// Other qualities can't be judged before the tokens exist, so they don't match.
pub fn token_chars_match(chars: &Characteristics, f: &Filter) -> bool {
    match f {
        Filter::Any | Filter::Token | Filter::Permanent => true,
        Filter::And(v) => v.iter().all(|x| token_chars_match(chars, x)),
        Filter::Or(v) => v.iter().any(|x| token_chars_match(chars, x)),
        Filter::Not(x) => !token_chars_match(chars, x),
        Filter::Type(t) => chars.card_types.contains(*t),
        Filter::Supertype(s) => chars.supertypes.contains(*s),
        Filter::Subtype(s) => chars.has_subtype(s),
        Filter::Color(c) => chars.colors.contains(*c),
        Filter::Colorless => chars.colors.is_colorless(),
        _ => false,
    }
}
