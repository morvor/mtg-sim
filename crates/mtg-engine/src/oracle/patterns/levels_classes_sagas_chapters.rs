//! Saga chapter abilities with a flavor word (CR 714.2b, 207.2d): "{rN} — Aerial Blast —
//! ~ deals 4 damage to target tapped creature an opponent controls." The flavor word has
//! no rules meaning, so the chapter ability is the chapter without it (parsed by the core
//! chapter pattern in `r107_symbols.rs`).
//!
//! Only a title-case phrase of a few words counts as a flavor word ("Wings of Light",
//! "Stampede!"); a clause such as "Target opponent faces a villainous choice — ..." is
//! part of the effect and stays.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

const CHAPTER: &str = "{CHAPTER} ";

/// Whether `head` looks like a flavor word: one to five words, each capitalized except
/// short connecting words, with no rules punctuation.
pub(crate) fn is_flavor_word(head: &str) -> bool {
    let words: Vec<&str> = head.split_whitespace().collect();
    if words.is_empty() || words.len() > 5 || head.contains(['{', ':', '"', '.', ',', '~']) {
        return false;
    }
    let small = ["of", "the", "and", "a", "an", "to", "in", "on", "for", "from", "with"];
    words.iter().enumerate().all(|(i, w)| {
        w.chars().next().is_some_and(|c| c.is_uppercase()) || (i > 0 && small.contains(w))
    })
}

fn chapter_with_flavor_word(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rest = block.trim().strip_prefix(CHAPTER)?;
    let (numbers, after) = rest.split_once(" — ")?;
    let (flavor, effect) = after.split_once(" — ")?;
    if !is_flavor_word(flavor) || effect.trim().is_empty() {
        return None;
    }
    let without = format!("{CHAPTER}{numbers} — {}", effect.trim());
    let mut abilities = crate::oracle::parse_ability(&without, ctx)?;
    // Keep the printed text for display.
    for a in &mut abilities {
        std::sync::Arc::make_mut(a).text = block[CHAPTER.len()..].trim().to_string();
    }
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "levels_classes_sagas: chapter flavor word", priority: 60, parse: chapter_with_flavor_word } }
