//! "Look at a card at random in target player's hand." (Urza's Bauble): a card chosen at
//! random from that player's hand is looked at by the ability's controller only
//! (CR 701.16a: looking, unlike revealing, shows it to that player alone). The looking
//! player may go on seeing that card while it stays in that hand (`zones::allow_look`);
//! it's "it" for the rest of the effect (`vars::IT`).
//!
//! An `Effect::Custom` named by [`effect_name`] (compiled in
//! `oracle/patterns/paper_random.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, PlayerRef};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

const PREFIX: &str = "look at a card at random in hand:target ";

/// The `Effect::Custom` name for a target player's hand (target slot `slot`).
pub fn effect_name(slot: u8) -> String {
    format!("{PREFIX}{slot}")
}

/// The target slot of an instruction's player.
pub fn slot_of(name: &str) -> Option<u8> {
    name.strip_prefix(PREFIX)?.parse().ok()
}

pub struct LookAtRandom;

impl KeywordRules for LookAtRandom {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(slot) = slot_of(name) else {
            return false;
        };
        let viewer = ctx.controller;
        let mut seen = Vec::new();
        for p in g.eval_players(&PlayerRef::Target(slot), ctx) {
            let hand: Vec<ObjectId> = g.player(p).hand.clone();
            if hand.is_empty() {
                continue;
            }
            use rand::Rng;
            let card = hand[g.rng_for(p).gen_range(0..hand.len())];
            g.log(|g| format!("{viewer} looks at {} in {p}'s hand", g.describe(card)));
            crate::zones::allow_look(g, viewer, card);
            seen.push(Entity::Object(card));
        }
        ctx.set_var(vars::IT, seen);
        true
    }
}

inventory::submit! { KeywordRegistration(&LookAtRandom) }
