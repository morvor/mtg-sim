//! "Target opponent reveals a card at random from their hand." (Planeswalker's Favor,
//! Planeswalker's Scorn): a card chosen at random from that player's hand is revealed
//! (CR 701.20a). It's "the revealed card" for the rest of the effect (`vars::IT`); with no
//! card in that hand, nothing is revealed and its mana value is 0 ("where X is the
//! revealed card's mana value").
//!
//! An `Effect::Custom` named by [`effect_name`] (compiled in
//! `oracle/patterns/reveal_at_random.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, PlayerRef};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

const PREFIX: &str = "reveal a card at random from hand:";

/// The `Effect::Custom` name for the player `who` (a target slot, or the controller).
pub fn effect_name(who: &PlayerRef) -> Option<String> {
    Some(match who {
        PlayerRef::You => format!("{PREFIX}you"),
        PlayerRef::Target(slot) => format!("{PREFIX}target {slot}"),
        _ => return None,
    })
}

fn player_of(name: &str) -> Option<PlayerRef> {
    let who = name.strip_prefix(PREFIX)?;
    Some(match who.strip_prefix("target ") {
        Some(slot) => PlayerRef::Target(slot.parse().ok()?),
        None if who == "you" => PlayerRef::You,
        None => return None,
    })
}

pub struct RevealAtRandom;

impl KeywordRules for RevealAtRandom {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(who) = player_of(name) else {
            return false;
        };
        let mut revealed = Vec::new();
        for p in g.eval_players(&who, ctx) {
            let hand: Vec<ObjectId> = g.player(p).hand.clone();
            if hand.is_empty() {
                continue;
            }
            use rand::Rng;
            let card = hand[g.rng.gen_range(0..hand.len())];
            crate::reveal::reveal_in(g, p, &[card], Some(ctx));
            revealed.push(Entity::Object(card));
        }
        ctx.set_var(vars::IT, revealed);
        true
    }
}

inventory::submit! { KeywordRegistration(&RevealAtRandom) }
