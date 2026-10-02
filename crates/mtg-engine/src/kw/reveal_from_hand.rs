//! "Target player reveals three cards from their hand" (Blackmail, Noggin Whack, Thieving
//! Sprite, ...): the player chooses that many cards from their own hand (all of them if
//! they have fewer) and reveals them (CR 701.20a). The chosen cards are "them" for "You
//! choose one of them".
//!
//! The choice is a `Sel::Choose` stored in [`REVEALED`]; this `Effect::Custom`
//! ([`REVEAL_CHOSEN`]) reveals what was stored (compiled in
//! `oracle/patterns/reveal_from_hand.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// The cards a player chose to reveal from their hand.
pub const REVEALED: Var = vars::USER + 2743;

/// The `Effect::Custom` name: reveal the cards in [`REVEALED`] (each to all players, by
/// its owner).
pub const REVEAL_CHOSEN: &str = "reveal the chosen cards from hand";

pub struct RevealFromHand;

impl KeywordRules for RevealFromHand {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REVEAL_CHOSEN {
            return false;
        }
        let cards: Vec<ObjectId> = ctx
            .vars
            .get(&REVEALED)
            .map(|v| {
                v.iter()
                    .filter_map(|e| match e {
                        Entity::Object(o) => Some(*o),
                        Entity::Player(_) => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut by_owner: Vec<(PlayerId, Vec<ObjectId>)> = Vec::new();
        for o in cards {
            let owner = g.obj(o).owner;
            match by_owner.iter_mut().find(|(p, _)| *p == owner) {
                Some((_, v)) => v.push(o),
                None => by_owner.push((owner, vec![o])),
            }
        }
        for (p, v) in by_owner {
            crate::reveal::reveal_in(g, p, &v, Some(ctx));
        }
        ctx.prev_happened = ctx.vars.get(&REVEALED).is_some_and(|v| !v.is_empty());
        true
    }
}

inventory::submit! { KeywordRegistration(&RevealFromHand) }
