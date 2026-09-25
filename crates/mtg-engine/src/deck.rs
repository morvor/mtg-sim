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
    /// More cards than the maximum deck size (only Commander decks have one, CR 100.5).
    TooManyCards { have: usize, max: usize },
    /// More copies of a card than allowed.
    TooManyCopies {
        name: String,
        have: usize,
        max: usize,
    },
    /// A constructed sideboard with more than fifteen cards (CR 100.4a).
    SideboardTooLarge { have: usize, max: usize },
    /// Limited: more copies of a card than the card pool has (CR 100.2b).
    NotInPool {
        name: String,
        have: usize,
        available: usize,
    },
    /// Limited team play: the decks and sideboards don't divide up the team's card pool
    /// (CR 100.4c, 100.4d) — a card is missing or used twice.
    PoolMismatch { name: String },
    /// Commander: a card whose color identity isn't within the commander's (CR 903.5c).
    OutsideColorIdentity { name: String },
    /// Commander: a Commander game doesn't use sideboards (CR 903.5e).
    SideboardNotAllowed,
    /// Tournament rules bar the card from the format (CR 100.6): it's banned, or from a
    /// set the format doesn't use.
    NotLegalInFormat { name: String, format: String },
    /// A card that must be removed from decks and sideboards when not playing for ante
    /// (CR 407.3).
    AnteCard { name: String },
}

/// Pairs of card names treated as the same English name for deck construction
/// (interchangeable names, CR 201.3b). Names not listed stand for themselves.
#[derive(Clone, Debug, Default)]
pub struct NameEquivalence(pub Vec<(String, String)>);

impl NameEquivalence {
    /// The name a card counts as for deck construction.
    pub fn key<'a>(&'a self, name: &'a str) -> &'a str {
        for (a, b) in &self.0 {
            if b == name {
                return a;
            }
        }
        name
    }
}

/// A card's own deck-construction ability (CR 113.6n): `Some(None)` for "any number of
/// cards named ~", `Some(Some(k))` for "up to k cards named ~".
fn copies_ability(card: &CardDef) -> Option<Option<usize>> {
    for a in &card.front().chars.abilities {
        let AbilityKind::Static(s) = &a.kind else {
            continue;
        };
        let StaticEffect::Custom(n) = &s.effect else {
            continue;
        };
        if n.as_str() == ANY_NUMBER {
            return Some(None);
        }
        if let Some(k) = n.strip_prefix(UP_TO).and_then(|k| k.parse().ok()) {
            return Some(Some(k));
        }
    }
    None
}

/// The most copies of a card a constructed deck may contain (`None` = any number): any
/// number of basic lands, no more than four of other cards (CR 100.2a), unless the card's
/// own ability changes that (CR 113.6n).
pub fn max_copies(card: &CardDef) -> Option<usize> {
    if is_basic_land(card) {
        return None;
    }
    copies_ability(card).unwrap_or(Some(4))
}

/// The traditional cards of a deck: nontraditional cards (planes, schemes, ...) belong
/// to supplementary decks with their own construction rules, not to the deck
/// (CR 100.2d, 108.2a).
fn traditional(deck: &[Arc<CardDef>]) -> Vec<&Arc<CardDef>> {
    deck.iter()
        .filter(|c| !crate::variants::is_nontraditional(c))
        .collect()
}

fn is_basic_land(card: &CardDef) -> bool {
    let c = &card.front().chars;
    c.supertypes.contains(Supertype::Basic) && c.card_types.contains(CardType::Land)
}

/// Copies of each card by deck-construction name (CR 201.3b).
fn counts<'a>(
    cards: impl IntoIterator<Item = &'a Arc<CardDef>>,
    names: &'a NameEquivalence,
) -> BTreeMap<&'a str, (usize, &'a Arc<CardDef>)> {
    let mut out: BTreeMap<&str, (usize, &Arc<CardDef>)> = BTreeMap::new();
    for c in cards {
        out.entry(names.key(c.name.as_str())).or_insert((0, c)).0 += 1;
    }
    out
}

/// Checks a constructed deck: at least 60 cards and no more copies of any card than
/// [`max_copies`] allows (CR 100.2a). There is no maximum deck size (CR 100.5).
/// Nontraditional cards aren't part of the deck (CR 100.2d, 108.2a).
pub fn check_constructed(deck: &[Arc<CardDef>]) -> Vec<DeckProblem> {
    check_constructed_with(deck, &[], &NameEquivalence::default())
}

/// Checks a constructed deck and its sideboard: the sideboard has at most fifteen cards,
/// and the four-card limit applies to the deck and sideboard combined (CR 100.4a); cards
/// with interchangeable names count as the same card (CR 100.2a, 201.3b).
pub fn check_constructed_with(
    deck: &[Arc<CardDef>],
    sideboard: &[Arc<CardDef>],
    names: &NameEquivalence,
) -> Vec<DeckProblem> {
    let cards = traditional(deck);
    let mut problems = Vec::new();
    if cards.len() < 60 {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: 60,
        });
    }
    let side = traditional(sideboard);
    if side.len() > 15 {
        problems.push(DeckProblem::SideboardTooLarge {
            have: side.len(),
            max: 15,
        });
    }
    for (name, (n, c)) in counts(cards.into_iter().chain(side), names) {
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

/// Checks a limited deck built from a card pool (CR 100.2b): at least 40 cards, made only
/// of cards from the pool — as many duplicates as the pool has — plus any number of basic
/// lands.
pub fn check_limited(deck: &[Arc<CardDef>], pool: &[Arc<CardDef>]) -> Vec<DeckProblem> {
    let cards = traditional(deck);
    let mut problems = Vec::new();
    if cards.len() < 40 {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: 40,
        });
    }
    let names = NameEquivalence::default();
    let available = counts(pool.iter(), &names);
    for (name, (n, c)) in counts(cards, &names) {
        if is_basic_land(c) {
            continue;
        }
        let have = available.get(name).map_or(0, |x| x.0);
        if n > have {
            problems.push(DeckProblem::NotInPool {
                name: name.to_string(),
                have: n,
                available: have,
            });
        }
    }
    problems
}

/// Removes one copy of each card of `used` from `pool` (basic lands not in the pool are
/// added freely). Returns the problems if something used isn't there.
fn take_from_pool(
    pool: &mut Vec<Arc<CardDef>>,
    used: &[Arc<CardDef>],
    problems: &mut Vec<DeckProblem>,
) {
    for c in used {
        match pool.iter().position(|x| x.name == c.name) {
            Some(i) => {
                pool.remove(i);
            }
            None if is_basic_land(c) => {}
            None => problems.push(DeckProblem::PoolMismatch {
                name: c.name.to_string(),
            }),
        }
    }
}

/// Limited play with individual players: every card in the pool that isn't in the deck
/// is in the sideboard (CR 100.4b).
pub fn limited_sideboard(pool: &[Arc<CardDef>], deck: &[Arc<CardDef>]) -> Vec<Arc<CardDef>> {
    let mut rest = pool.to_vec();
    take_from_pool(&mut rest, deck, &mut Vec::new());
    rest
}

/// Limited Two-Headed Giant: every card in the team's pool that isn't in either player's
/// deck is in the team's (shared) sideboard (CR 100.4c).
pub fn team_sideboard(pool: &[Arc<CardDef>], decks: &[Vec<Arc<CardDef>>]) -> Vec<Arc<CardDef>> {
    let mut rest = pool.to_vec();
    for d in decks {
        take_from_pool(&mut rest, d, &mut Vec::new());
    }
    rest
}

/// Limited play in other team variants: each card in the team's pool that isn't in any
/// player's deck is assigned to exactly one player's sideboard; each player has their own,
/// and cards can't be transferred between players (CR 100.4d). Checks that the decks and
/// sideboards divide the pool exactly.
pub fn check_team_pool(
    pool: &[Arc<CardDef>],
    decks: &[Vec<Arc<CardDef>>],
    sideboards: &[Vec<Arc<CardDef>>],
) -> Vec<DeckProblem> {
    let mut rest = pool.to_vec();
    let mut problems = Vec::new();
    for d in decks {
        take_from_pool(&mut rest, d, &mut problems);
    }
    for s in sideboards {
        take_from_pool(&mut rest, s, &mut problems);
    }
    for c in rest {
        problems.push(DeckProblem::PoolMismatch {
            name: c.name.to_string(),
        });
    }
    problems
}

/// Checks a Commander deck (CR 100.2c, 903.5): exactly 100 cards including the commander
/// (60 with the Brawl option, CR 903.12d), each nonbasic card only once, every card's color
/// identity within the commander's, and no sideboard.
pub fn check_commander(
    deck: &[Arc<CardDef>],
    commander: &CardDef,
    sideboard: &[Arc<CardDef>],
    brawl: bool,
) -> Vec<DeckProblem> {
    let cards = traditional(deck);
    let size = if brawl { 60 } else { 100 };
    let mut problems = Vec::new();
    if cards.len() < size {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: size,
        });
    }
    if cards.len() > size {
        problems.push(DeckProblem::TooManyCards {
            have: cards.len(),
            max: size,
        });
    }
    let names = NameEquivalence::default();
    for (name, (n, c)) in counts(cards.iter().copied(), &names) {
        // Singleton, other than basic lands (CR 903.5b) and cards whose own ability says
        // otherwise (CR 113.6n).
        let max = if is_basic_land(c) {
            None
        } else {
            copies_ability(c).unwrap_or(Some(1))
        };
        if let Some(max) = max {
            if n > max {
                problems.push(DeckProblem::TooManyCopies {
                    name: name.to_string(),
                    have: n,
                    max,
                });
            }
        }
        let ci = c.color_identity;
        if ci.union(commander.color_identity) != commander.color_identity {
            problems.push(DeckProblem::OutsideColorIdentity {
                name: name.to_string(),
            });
        }
    }
    if !sideboard.is_empty() {
        problems.push(DeckProblem::SideboardNotAllowed);
    }
    problems
}

/// Checks a deck and sideboard against a tournament format's card restrictions
/// (CR 100.6), which may bar some cards, including all cards from some older sets: every
/// card must be legal in the format, and a card restricted in it may appear only once in
/// the deck and sideboard combined. Nontraditional cards aren't part of the deck
/// (CR 100.2d).
pub fn check_format_legality(
    deck: &[Arc<CardDef>],
    sideboard: &[Arc<CardDef>],
    format: &str,
) -> Vec<DeckProblem> {
    let names = NameEquivalence::default();
    let mut problems = Vec::new();
    for (name, (n, c)) in counts(
        traditional(deck).into_iter().chain(traditional(sideboard)),
        &names,
    ) {
        if !c.is_legal_in(format) {
            problems.push(DeckProblem::NotLegalInFormat {
                name: name.to_string(),
                format: format.to_string(),
            });
        } else if c.is_restricted_in(format) && n > 1 {
            problems.push(DeckProblem::TooManyCopies {
                name: name.to_string(),
                have: n,
                max: 1,
            });
        }
    }
    problems
}
