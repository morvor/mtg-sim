//! Deck construction: constructed-deck limits (CR 100.2a) and abilities that modify the
//! deck construction rules (CR 113.6n). Such abilities function before the game begins.

use crate::ability::*;
use crate::card::CardDef;
use crate::types::*;
use std::collections::BTreeMap;
use std::sync::Arc;

/// `StaticEffect::Custom` name of "A deck can have any number of cards named ~".
pub const ANY_NUMBER: &str = "deck:any number";
/// `StaticEffect::Custom` name prefix of "A deck can have up to N cards named ~".
pub const UP_TO: &str = "deck:up to ";

/// A way a deck breaks the deck construction rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeckProblem {
    /// Fewer cards than the minimum deck size.
    TooFewCards { have: usize, min: usize },
    /// More copies of a card than allowed.
    TooManyCopies {
        name: String,
        have: usize,
        max: usize,
    },
}

/// The most copies of a card a constructed deck may contain (`None` = any number): any
/// number of basic lands, no more than four of other cards (CR 100.2a), unless the card's
/// own ability changes that (CR 113.6n).
pub fn max_copies(card: &CardDef) -> Option<usize> {
    let c = &card.front().chars;
    if c.supertypes.contains(Supertype::Basic) && c.card_types.contains(CardType::Land) {
        return None;
    }
    for a in &c.abilities {
        let AbilityKind::Static(s) = &a.kind else {
            continue;
        };
        let StaticEffect::Custom(n) = &s.effect else {
            continue;
        };
        if n.as_str() == ANY_NUMBER {
            return None;
        }
        if let Some(k) = n.strip_prefix(UP_TO).and_then(|k| k.parse().ok()) {
            return Some(k);
        }
    }
    Some(4)
}

/// Checks a constructed deck: at least 60 cards and no more copies of any card than
/// [`max_copies`] allows (CR 100.2a). Nontraditional cards aren't part of the deck
/// (CR 108.2a).
pub fn check_constructed(deck: &[Arc<CardDef>]) -> Vec<DeckProblem> {
    let cards: Vec<&Arc<CardDef>> = deck
        .iter()
        .filter(|c| !crate::variants::is_nontraditional(c))
        .collect();
    let mut problems = Vec::new();
    if cards.len() < 60 {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: 60,
        });
    }
    let mut counts: BTreeMap<&str, (usize, &Arc<CardDef>)> = BTreeMap::new();
    for c in &cards {
        counts.entry(c.name.as_str()).or_insert((0, c)).0 += 1;
    }
    for (name, (n, c)) in counts {
        if let Some(max) = max_copies(c) {
            if n > max {
                problems.push(DeckProblem::TooManyCopies {
                    name: name.to_string(),
                    have: n,
                    max,
                });
            }
        }
    }
    problems
}
