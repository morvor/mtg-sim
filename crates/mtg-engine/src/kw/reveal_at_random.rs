//! "Target opponent reveals a card at random from their hand." (Planeswalker's Favor,
//! Planeswalker's Scorn): a card chosen at random from that player's hand is revealed
//! (CR 701.20a). It's "the revealed card" for the rest of the effect (`vars::IT`); with no
//! card in that hand, nothing is revealed and its mana value is 0 ("where X is the
//! revealed card's mana value").
//!
//! An `Effect::Custom` named by [`effect_name`] (compiled in
//! `oracle/patterns/reveal_at_random.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, PlayerRef, Sel};
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

/// "reveals two cards at random from their hand" (Rise // Fall), "Target creature's
/// controller reveals a card at random from their hand." (Friendly Fire): `n` cards, by
/// the player `who` (a target slot, the controller of a target, or the controller).
const PREFIX_N: &str = "reveal cards at random from hand:";

/// The `Effect::Custom` name for `n` cards revealed at random by `who`.
pub fn effect_name_n(who: &PlayerRef, n: u32) -> Option<String> {
    let who = match who {
        PlayerRef::You => "you".to_string(),
        PlayerRef::Target(slot) => format!("target {slot}"),
        PlayerRef::ControllerOf(s) => match &**s {
            Sel::Target(slot) => format!("controller of target {slot}"),
            _ => return None,
        },
        _ => return None,
    };
    Some(format!("{PREFIX_N}{n}:{who}"))
}

fn who_of(who: &str) -> Option<PlayerRef> {
    if let Some(slot) = who.strip_prefix("controller of target ") {
        return Some(PlayerRef::ControllerOf(Box::new(Sel::Target(
            slot.parse().ok()?,
        ))));
    }
    Some(match who.strip_prefix("target ") {
        Some(slot) => PlayerRef::Target(slot.parse().ok()?),
        None if who == "you" => PlayerRef::You,
        None => return None,
    })
}

/// The player and the number of cards of an instruction.
fn player_of(name: &str) -> Option<(PlayerRef, usize)> {
    if let Some(r) = name.strip_prefix(PREFIX_N) {
        let (n, who) = r.split_once(':')?;
        return Some((who_of(who)?, n.parse().ok()?));
    }
    Some((who_of(name.strip_prefix(PREFIX)?)?, 1))
}

pub struct RevealAtRandom;

impl KeywordRules for RevealAtRandom {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((who, n)) = player_of(name) else {
            return false;
        };
        let mut revealed = Vec::new();
        for p in g.eval_players(&who, ctx) {
            let mut hand: Vec<ObjectId> = g.player(p).hand.clone();
            if hand.is_empty() {
                continue;
            }
            if n == 1 {
                use rand::Rng;
                let card = hand[g.rng_for(p).gen_range(0..hand.len())];
                hand = vec![card];
            } else {
                use rand::seq::SliceRandom;
                hand.shuffle(g.rng_for(p));
                hand.truncate(n);
            }
            crate::reveal::reveal_in(g, p, &hand, Some(ctx));
            revealed.extend(hand.into_iter().map(Entity::Object));
        }
        ctx.set_var(vars::IT, revealed);
        true
    }
}

inventory::submit! { KeywordRegistration(&RevealAtRandom) }
