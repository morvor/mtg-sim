//! Kinship, an ability word (CR 207.2c: it has no rules meaning of its own): "At the
//! beginning of your upkeep, you may look at the top card of your library. If it shares a
//! creature type with ~, you may reveal it. If you do, [effect]."
//!
//! The middle instruction is [`REVEAL_TOP_IF_SHARES_TYPE`] (compiled by
//! `oracle/patterns/kinship.rs` after the "look at the top card" instruction): if the top
//! card of the controller's library shares a creature type with the source, the
//! controller may reveal it, even if it's already revealed (CR 701.20c). Whether it was
//! revealed is what "If you do" checks, and the revealed card is what "that card" refers
//! to. The card stays on top of the library.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Effect::Custom`: "If [the top card of your library] shares a creature type with ~,
/// you may reveal it."
pub const REVEAL_TOP_IF_SHARES_TYPE: &str = "reveal top card if it shares a creature type with ~";

pub struct Kinship;

impl KeywordRules for Kinship {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REVEAL_TOP_IF_SHARES_TYPE {
            return false;
        }
        let p = ctx.controller;
        let shares = Filter::SharesCreatureType(Box::new(Sel::This));
        let revealed = match g.library_top(p) {
            Some(card) if g.matches(card, &shares, ctx) => {
                let prompt = format!("Reveal {}?", g.describe(card));
                if g.ask_yes_no(p, ctx.source, &prompt, true) {
                    crate::reveal::reveal_in(g, p, &[card], Some(ctx));
                    ctx.set_var(vars::IT, vec![Entity::Object(card)]);
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        ctx.prev_happened = revealed;
        true
    }
}

inventory::submit! { KeywordRegistration(&Kinship) }
