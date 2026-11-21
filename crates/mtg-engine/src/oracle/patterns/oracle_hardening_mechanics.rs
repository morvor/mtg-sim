//! Mechanics the engine doesn't implement, which a card's abilities can't express.
//!
//! The Comprehensive Rules don't define augment and host (Unstable); the engine has no
//! notion of combining an augment card with a host. A host creature's "When this creature
//! enters" is its host ability, and an augment card's text is an incomplete ability that
//! only means something once combined, so such cards are reported as unsupported rather
//! than as ordinary creatures.
//!
//! Likewise a type-line word that is neither a card type (CR 205.2a) nor a supertype
//! (CR 205.4a) — "Host", "Hero", "Summon" — is dropped when the type line is parsed, so the
//! object would silently lack whatever the word means.

use crate::card::Layout;
use crate::types::{CardType, Supertype};

/// Type-line words to the left of the dash that aren't types or supertypes but carry no
/// rules the engine lacks: tokens and emblems are labeled as such (CR 111.4, 114.2), and
/// sticker sheets are handled as stickers (CR 123).
const HARMLESS_WORDS: &[&str] = &["Token", "Emblem", "Stickers"];

/// Unsupported-text entries for a card face whose layout or type line uses a mechanic the
/// engine doesn't implement.
pub fn unimplemented(layout: Layout, type_line: &str) -> Vec<String> {
    let mut out = Vec::new();
    match layout {
        Layout::Augment => out.push("Augment (combining with a host) isn't implemented".into()),
        Layout::Host => out.push("Host (combining with an augment) isn't implemented".into()),
        _ => {}
    }
    let left = type_line
        .split('—')
        .next()
        .unwrap_or_default()
        .split(" - ")
        .next()
        .unwrap_or_default();
    for w in left.split_whitespace() {
        if Supertype::from_word(w).is_none()
            && CardType::from_word(w).is_none()
            && !HARMLESS_WORDS.contains(&w)
        {
            out.push(format!("Type-line word \"{w}\" has no rules in the engine"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_type_words_and_layouts() {
        assert!(unimplemented(Layout::Normal, "Legendary Creature — Elf Druid").is_empty());
        assert!(unimplemented(Layout::Token, "Token Creature — Goblin").is_empty());
        assert!(unimplemented(Layout::Normal, "Tribal Instant — Elf").is_empty());
        assert_eq!(unimplemented(Layout::Host, "Host Creature — Dog").len(), 2);
        assert_eq!(unimplemented(Layout::Augment, "Creature — Bird").len(), 1);
        // Old type lines without a dash ("Summon Dragon"): both words are unknown.
        assert_eq!(unimplemented(Layout::Normal, "Summon Dragon").len(), 2);
    }
}
