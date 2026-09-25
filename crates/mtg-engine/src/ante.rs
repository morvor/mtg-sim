//! Playing for ante (CR 407), an optional variation enabled by `GameConfig::ante`.
//!
//! * Before any cards are drawn, each player antes a random card from their deck; the
//!   winner of the game becomes the owner of every card in the ante zone (CR 407.2).
//! * Cards with "Remove this card from your deck before playing if you're not playing for
//!   ante" are the only ones that add or remove cards from the ante zone or change a card's
//!   owner; without ante they can't be in decks or sideboards, nor be brought into the
//!   game from outside it (CR 407.3).
//! * Only an object's owner can ante it (CR 407.4).

use crate::ability::*;
use crate::card::CardDef;
use crate::deck::DeckProblem;
use crate::game::{Game, GameResult};
use crate::object::*;
use crate::replacement::MoveEv;
use crate::types::*;
use std::sync::Arc;

/// `StaticEffect::Custom` name of "Remove this card from your deck before playing if
/// you're not playing for ante."
pub const ANTE_CARD: &str = "ante:card";

/// Whether a card has the ante reminder "Remove this card from your deck before playing
/// if you're not playing for ante." (CR 407.3).
pub fn is_ante_card(card: &CardDef) -> bool {
    card.faces.iter().any(|f| {
        f.chars.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Static(s)
                if matches!(&s.effect, StaticEffect::Custom(n) if n.as_str() == ANTE_CARD))
        })
    })
}

fn object_is_ante_card(g: &Game, id: ObjectId) -> bool {
    g.obj(id).card.as_deref().is_some_and(is_ante_card)
}

/// CR 407.3: when not playing for ante, ante cards can't be in a deck or sideboard.
pub fn check_deck(
    deck: &[Arc<CardDef>],
    sideboard: &[Arc<CardDef>],
    playing_for_ante: bool,
) -> Vec<DeckProblem> {
    if playing_for_ante {
        return vec![];
    }
    let mut out: Vec<DeckProblem> = Vec::new();
    for c in deck.iter().chain(sideboard) {
        if is_ante_card(c) {
            let p = DeckProblem::AnteCard {
                name: c.name.to_string(),
            };
            if !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

/// Moves the rules of ante forbid; the object stays where it is.
pub fn move_forbidden(g: &Game, mv: &MoveEv) -> bool {
    let o = g.obj(mv.obj);
    // CR 407.3: without ante, these cards can't be brought into the game from outside.
    if matches!(o.zone, Zone::Outside(_))
        && !matches!(mv.to, Zone::Outside(_))
        && !g.config.ante
        && object_is_ante_card(g, mv.obj)
    {
        return true;
    }
    let into = mv.to == Zone::Ante && o.zone != Zone::Ante;
    let out_of = o.zone == Zone::Ante && !matches!(mv.to, Zone::Ante | Zone::Nowhere);
    if into || out_of {
        // CR 407.3: only ante cards add cards to or remove cards from the ante zone.
        if !mv.source.is_some_and(|s| object_is_ante_card(g, s)) {
            return true;
        }
        // CR 407.4: only an object's owner can ante it.
        if into && mv.by != Some(o.owner) {
            return true;
        }
    }
    false
}

/// CR 407.2: after the starting player is determined and before any cards are drawn,
/// each player puts a random card from their deck into the ante zone.
pub fn ante_at_start(g: &mut Game) {
    use rand::Rng;
    if !g.config.ante {
        return;
    }
    for p in g.apnap() {
        let n = g.player(p).library.len();
        if n == 0 {
            continue;
        }
        let i = g.rng.gen_range(0..n);
        let card = g.players[p.idx()].library.remove(i);
        let new = g.create_incarnation(card, Zone::Ante);
        g.ante.push(new);
        g.log(|g| format!("{p} antes {}", g.describe(new)));
    }
    g.dirty = true;
}

/// CR 407.2: at the end of the game, the winner becomes the owner of all the cards in
/// the ante zone.
pub fn game_ended(g: &mut Game) {
    if !g.config.ante {
        return;
    }
    let Some(GameResult::Win(winners)) = &g.result else {
        return;
    };
    let Some(&winner) = winners.first() else {
        return;
    };
    for id in g.ante.clone() {
        g.objects[id.0 as usize].owner = winner;
    }
    g.log(|_| format!("{winner} becomes the owner of the cards in the ante"));
}
