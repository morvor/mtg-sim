//! Attraction cards (CR 717). Opening an Attraction and rolling to visit Attractions are
//! keyword actions (CR 701.51, 701.52: `kwa/attractions.rs`, `variants.rs`).
//!
//! * Attraction is an artifact subtype of nontraditional cards with lit-up numbers
//!   (`CardDef::attraction_lights`, CR 717.1).
//! * Attraction cards aren't part of a player's deck: a player's Attraction deck is the
//!   face-down Attraction cards they own in the command zone, shuffled before the game
//!   begins (CR 717.2, 103.3a). Its construction rules: [`check_constructed_deck`],
//!   [`check_limited_deck`] (CR 717.2a, 717.2b).
//! * A card with an Astrotorium card back that would be put into a zone other than the
//!   battlefield, exile or the command zone is put into the command zone instead
//!   (CR 717.6), face up, apart from the Attraction deck: its owner's "junkyard"
//!   (CR 717.6a, [`junkyard`]).

use crate::card::CardDef;
use crate::deck::DeckProblem;
use crate::game::Game;
use crate::object::{ObjKind, Zone};
use crate::replacement::MoveEv;
use crate::types::*;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Whether a card is an Attraction card (it has an Astrotorium card back, CR 717.1).
pub fn is_attraction_card(card: &CardDef) -> bool {
    card.faces
        .iter()
        .any(|f| f.chars.has_subtype("Attraction") && f.chars.is(CardType::Artifact))
}

/// Whether the object is an Attraction card (not a token or copy).
fn is_attraction_object(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.kind == ObjKind::Card && o.card.as_deref().is_some_and(is_attraction_card)
}

/// CR 717.2, 103.3a: each player's Attraction deck is shuffled before the game begins.
pub fn shuffle_attraction_decks(g: &mut Game) {
    use rand::seq::SliceRandom;
    for p in g.player_ids() {
        let slots: Vec<usize> = g
            .command
            .iter()
            .enumerate()
            .filter(|(_, id)| {
                let o = g.obj(**id);
                o.face_down && o.owner == p && is_attraction_object(g, **id)
            })
            .map(|(i, _)| i)
            .collect();
        let mut ids: Vec<ObjectId> = slots.iter().map(|i| g.command[*i]).collect();
        ids.shuffle(&mut g.rng);
        for (i, id) in slots.into_iter().zip(ids) {
            g.command[i] = id;
        }
    }
}

/// CR 717.6: whether the move puts an Attraction card into a zone other than the
/// battlefield, exile, or the command zone (it goes to the command zone instead).
pub fn goes_to_junkyard(g: &Game, m: &MoveEv) -> bool {
    !matches!(
        m.to,
        Zone::Battlefield | Zone::Exile | Zone::Command | Zone::Nowhere
    ) && is_attraction_object(g, m.obj)
}

/// A player's junkyard: the face-up Attraction cards they own that were put into the
/// command zone instead of another zone, kept apart from their Attraction deck
/// (CR 717.6a).
pub fn junkyard(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            !o.face_down && o.owner == p && is_attraction_object(g, *id)
        })
        .collect()
}

/// Cards of an Attraction deck that aren't Attractions.
fn non_attractions(cards: &[Arc<CardDef>]) -> Vec<DeckProblem> {
    cards
        .iter()
        .filter(|c| !is_attraction_card(c))
        .map(|c| DeckProblem::NotAnAttraction {
            name: c.name.to_string(),
        })
        .collect()
}

/// CR 717.2a: in constructed play, an Attraction deck has at least ten Attraction cards,
/// each with a different English name.
pub fn check_constructed_deck(cards: &[Arc<CardDef>]) -> Vec<DeckProblem> {
    let mut problems = non_attractions(cards);
    if cards.len() < 10 {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: 10,
        });
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cards {
        *counts.entry(c.name.as_str()).or_insert(0) += 1;
    }
    for (name, n) in counts {
        if n > 1 {
            problems.push(DeckProblem::TooManyCopies {
                name: name.to_string(),
                have: n,
                max: 1,
            });
        }
    }
    problems
}

/// CR 717.2b: in limited play, an Attraction deck has at least three Attraction cards
/// from the player's card pool; it may contain several with the same English name (as
/// many as the pool has).
pub fn check_limited_deck(cards: &[Arc<CardDef>], pool: &[Arc<CardDef>]) -> Vec<DeckProblem> {
    let mut problems = non_attractions(cards);
    if cards.len() < 3 {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min: 3,
        });
    }
    let mut available: BTreeMap<&str, usize> = BTreeMap::new();
    for c in pool {
        *available.entry(c.name.as_str()).or_insert(0) += 1;
    }
    let mut used: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cards {
        *used.entry(c.name.as_str()).or_insert(0) += 1;
    }
    for (name, n) in used {
        let have = available.get(name).copied().unwrap_or(0);
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
