//! "[Player] reveals their library" (Mindblaze): every card in it is revealed (CR
//! 701.20a); its order is unchanged (CR 701.20b).

use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// `Effect::Custom`: the player performing the instruction (`ctx.controller`) reveals
/// their library.
pub const REVEAL_OWN_LIBRARY: &str = "reveal: their library";

pub struct RevealLibrary;

impl crate::kw::KeywordRules for RevealLibrary {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REVEAL_OWN_LIBRARY {
            return false;
        }
        let p = ctx.controller;
        let cards: Vec<ObjectId> = g.player(p).library.clone();
        crate::reveal::reveal_in(g, p, &cards, Some(ctx));
        ctx.set_var(
            crate::ability::vars::REVEALED,
            cards.into_iter().map(Entity::Object).collect(),
        );
        true
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&RevealLibrary) }
