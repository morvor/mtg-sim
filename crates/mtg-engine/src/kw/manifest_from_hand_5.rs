//! Manifesting cards from a hand (cards5-choices-players): "up to two target players each
//! manifest two cards from their hands. For each card manifested this way, you draw a
//! card." (Kozilek, the Broken Reality).
//!
//! To manifest a card is to turn it face down and put it onto the battlefield (CR
//! 701.40a): the player chooses that many cards from their hand (all of them with fewer,
//! its ruling) and manifests them one at a time (CR 701.40e); a card that can't enter face
//! down stays in the hand (CR 701.40f). The manifested permanents are kept for "each card
//! manifested this way" ([`MANIFESTED_THIS_WAY`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Effect, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Effect::Custom` name prefix: the player (`ctx.controller`) manifests that many cards
/// (the number follows) from their hand.
pub const MANIFEST_FROM_HAND: &str = "manifest5: manifest cards from hand:";
/// The permanents manifested this way, over all the players who did.
pub const MANIFESTED_THIS_WAY: Var = crate::ability::vars::USER + 9561;

/// The instruction to manifest `n` cards from the hand.
pub fn manifest_from_hand(n: u32) -> Effect {
    Effect::Custom(format!("{MANIFEST_FROM_HAND}{n}").into())
}

pub struct ManifestFromHand;

impl KeywordRules for ManifestFromHand {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(n) = name
            .strip_prefix(MANIFEST_FROM_HAND)
            .and_then(|n| n.parse::<usize>().ok())
        else {
            return false;
        };
        let p = ctx.controller;
        let hand: Vec<ObjectId> = g.zone_objects(Zone::Hand(p));
        let k = n.min(hand.len());
        let cards = if k == hand.len() {
            hand
        } else {
            g.ask_objects(p, ctx.source, "Choose cards to manifest", hand, k as u32, k as u32)
        };
        let mut done: Vec<Entity> = ctx.vars.get(&MANIFESTED_THIS_WAY).cloned().unwrap_or_default();
        for c in cards {
            g.end_event_batch();
            if let Some(new) =
                crate::kwa::manifest::put_face_down(g, c, p, crate::kwa::manifest::MANIFESTED, ctx.source)
            {
                done.push(Entity::Object(new));
            }
        }
        ctx.set_var(MANIFESTED_THIS_WAY, done);
        true
    }
}

inventory::submit! { KeywordRegistration(&ManifestFromHand) }
