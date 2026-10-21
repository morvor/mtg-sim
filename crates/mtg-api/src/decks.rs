//! Decks for games the API starts: parsing decklists into cards, and the card-tier bar
//! (`mtg_engine::card_tiers`) every deck must meet. The bar is
//! [`ProtocolOptions::min_card_tier`](crate::ProtocolOptions::min_card_tier), `Verified`
//! by default; [`crate::Session`] checks it before the game starts. Tests and the engine's
//! `Game` never check tiers.

use mtg_engine::card_tiers::{BelowBar, Tier, TierTable};
use mtg_engine::{CardDb, CardDef};
use std::fmt;
use std::sync::Arc;

/// A parsed decklist: the main deck and the sideboard.
#[derive(Clone, Debug, Default)]
pub struct Decklist {
    pub main: Vec<Arc<CardDef>>,
    pub sideboard: Vec<Arc<CardDef>>,
}

/// Why decks can't be used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeckError {
    /// A line names no card.
    UnknownCard { line: usize, name: String },
    /// Cards below the bar, per seat (0-based).
    BelowBar(Vec<(usize, BelowBar)>),
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckError::UnknownCard { line, name } => {
                write!(f, "line {line}: unknown card {name:?}")
            }
            DeckError::BelowBar(seats) => {
                for (seat, b) in seats {
                    write!(f, "seat {seat}'s deck: {b}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for DeckError {}

/// Parses "4 Lightning Bolt" lines (also "4x"); the cards after a "Sideboard" or
/// "Companion" line are the sideboard; `#` and `//` start comments.
pub fn parse_decklist(text: &str) -> Result<Decklist, DeckError> {
    let mut d = Decklist::default();
    let mut side = false;
    for (n, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') || l.starts_with("//") {
            continue;
        }
        if l.eq_ignore_ascii_case("sideboard") || l.eq_ignore_ascii_case("companion") {
            side = true;
            continue;
        }
        let (count, name) = match l.split_once(' ') {
            Some((k, rest)) if k.trim_end_matches('x').parse::<usize>().is_ok() => (
                k.trim_end_matches('x').parse::<usize>().unwrap_or(1),
                rest.trim(),
            ),
            _ => (1, l),
        };
        let Some(c) = CardDb::global().get(name) else {
            return Err(DeckError::UnknownCard {
                line: n + 1,
                name: name.to_string(),
            });
        };
        let out = if side { &mut d.sideboard } else { &mut d.main };
        out.extend(std::iter::repeat_n(c, count));
    }
    Ok(d)
}

/// Checks each seat's cards against `bar` with `table` (no check for
/// `Tier::Unsupported`).
pub fn check_decks(
    table: &TierTable,
    decks: &[Vec<Arc<CardDef>>],
    bar: Tier,
) -> Result<(), DeckError> {
    if bar == Tier::Unsupported {
        return Ok(());
    }
    let below: Vec<(usize, BelowBar)> = decks
        .iter()
        .enumerate()
        .filter_map(|(seat, d)| {
            mtg_engine::card_tiers::check_cards(table, d.iter().map(|c| c.as_ref()), bar)
                .err()
                .map(|e| (seat, e))
        })
        .collect();
    if below.is_empty() {
        Ok(())
    } else {
        Err(DeckError::BelowBar(below))
    }
}
