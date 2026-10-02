//! Runtime support for the zone-move grammar (`oracle/patterns/zone_move_grammar.rs`):
//! "return a creature card at random from your graveyard to your hand" — the cards are
//! picked at random among those that match as the instruction is performed (CR 608.2c).

use super::{KeywordRegistration, KeywordRules};
use crate::types::Entity;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::zone_move_grammar::{RANDOM_COUNT, RANDOM_PICK, RANDOM_POOL};

/// `Effect::Custom`: picks [`RANDOM_COUNT`] of the objects in [`RANDOM_POOL`] at random
/// (all of them if there are fewer) and stores them in [`RANDOM_PICK`].
pub const PICK_AT_RANDOM: &str = "zone move: pick at random";

pub struct ZoneMoves;

impl KeywordRules for ZoneMoves {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PICK_AT_RANDOM {
            return false;
        }
        use rand::seq::SliceRandom;
        let mut pool: Vec<Entity> = ctx.vars.get(&RANDOM_POOL).cloned().unwrap_or_default();
        let n = ctx.nums.get(&RANDOM_COUNT).copied().unwrap_or(0).max(0) as usize;
        pool.shuffle(&mut g.rng);
        pool.truncate(n);
        ctx.vars.insert(RANDOM_PICK, pool);
        true
    }
}

inventory::submit! { KeywordRegistration(&ZoneMoves) }
