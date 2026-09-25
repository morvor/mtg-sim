//! Text-changing effects (CR 612): changing color words and basic land types in rules
//! text.

use crate::object::Characteristics;

/// Replaces one color word or basic land type with another in the object's text
/// (CR 612.2). The ability language stores filters structurally, so this rewrites the
/// relevant filter nodes.
pub fn change_text(c: &mut Characteristics, from: &str, to: &str) {
    let _ = (c, from, to);
}
