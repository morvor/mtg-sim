//! Parley (an ability word, CR 207.2c): "Each player reveals the top card of their
//! library. For each nonland card revealed this way, [effect]. Then each player draws a
//! card."
//!
//! * "Each player reveals the top card of their library" ([`REVEAL_TOPS`]): in APNAP order
//!   each player in the game with a card in their library reveals its top card
//!   (CR 701.20a); revealing doesn't move the cards (CR 701.20b), so unless something
//!   changes the libraries in between, the cards drawn afterward are the ones revealed.
//! * "for each nonland (or land) card revealed this way" counts those cards
//!   ([`NONLAND_REVEALED`], [`LAND_REVEALED`]). The sentence patterns are in
//!   `oracle/patterns/parley.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Effect::Custom` name: each player reveals the top card of their library.
pub const REVEAL_TOPS: &str = "parley:each player reveals the top card of their library";
/// `Value::Custom` name: the number of nonland cards revealed by [`REVEAL_TOPS`].
pub const NONLAND_REVEALED: &str = "parley:nonland cards revealed this way";
/// `Value::Custom` name: the number of land cards revealed by [`REVEAL_TOPS`].
pub const LAND_REVEALED: &str = "parley:land cards revealed this way";

/// The variable holding the cards revealed by [`REVEAL_TOPS`].
const REVEALED: Var = vars::USER + 207;

fn reveal_tops(g: &mut Game, ctx: &mut Ctx) {
    let mut revealed = Vec::new();
    for p in g.apnap() {
        let Some(top) = g.library_top(p) else {
            continue;
        };
        crate::reveal::reveal_in(g, p, &[top], Some(ctx));
        revealed.push(Entity::Object(top));
    }
    // "You may put the revealed cards into their owners' graveyards." (`dig_grammar`).
    crate::dig_steps::set_dug(ctx, revealed.clone());
    ctx.set_var(REVEALED, revealed);
}

fn count_revealed(g: &Game, ctx: &Ctx, land: bool) -> i64 {
    ctx.vars
        .get(&REVEALED)
        .map(|v| {
            v.iter()
                .filter_map(|e| e.object())
                .filter(|id| g.obj(*id).chars.is_land() == land)
                .count() as i64
        })
        .unwrap_or(0)
}

pub struct Parley;

impl KeywordRules for Parley {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REVEAL_TOPS {
            return false;
        }
        reveal_tops(g, ctx);
        true
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        match name {
            NONLAND_REVEALED => Some(count_revealed(g, ctx, false)),
            LAND_REVEALED => Some(count_revealed(g, ctx, true)),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&Parley) }
