//! Possibility Storm: "Whenever a player casts a spell from their hand, that player exiles
//! it, then exiles cards from the top of their library until they exile a card that shares
//! a card type with it. That player may cast that card without paying its mana cost. Then
//! they put all cards exiled with ~ on the bottom of their library in a random order."
//!
//! Everything but the last sentence compiles; the cards still exiled with ~ (the original
//! spell among them, and the found card if it wasn't cast — rulings) then go to the bottom
//! in a random order.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "Whenever a player casts a spell from their hand, that player exiles it, then exiles cards from the top of their library until they exile a card that shares a card type with it. That player may cast that card without paying its mana cost. Then they put all cards exiled with ~ on the bottom of their library in a random order.";

inventory::submit! { ManualAbility {
    card: "Possibility Storm",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever a player casts a spell from their hand, that player exiles it, then exiles cards from the top of their library until they exile a card that shares a card type with it. That player may cast that card without paying its mana cost.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| {
                    let mut bottom = Destination::zone(ZoneKind::Library);
                    bottom.position = LibraryPosition::BottomRandom;
                    Effect::Seq(vec![
                        e,
                        Effect::Move {
                            what: Sel::All(Filter::And(vec![
                                Filter::Card,
                                Filter::In(Box::new(Sel::Linked)),
                                Filter::InZone(ZoneKind::Exile),
                            ])),
                            to: bottom,
                        },
                    ])
                })
            })
            .collect()
    },
    reason: "the cards exiled with it go to the bottom afterwards: the rest compiles, only this card",
} }
