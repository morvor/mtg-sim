//! Behaviors for the grammar of `oracle/patterns/game_rules_4.rs`:
//!
//! - [`COLOR_OUTSIDE_COMMANDER_IDENTITY`]: what "protection from each color that's not in
//!   your commander's color identity" protects from (CR 903.4, 903.4f, 702.16);
//! - [`MORE_LOYALTY`]: "you may activate ~'s loyalty ability two more times this turn"
//!   (CR 606.3);
//! - [`DIFFERENT_DUNGEONS_COMPLETED`]: "the number of differently named dungeons you've
//!   completed" (CR 309.7).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom` name: an object with a color that isn't in the color identity of the
/// commanders of the player the filter is evaluated for (for a protection ability, the
/// protected permanent's controller). Matches nothing if that player has no commander:
/// the quality is undefined and the ability does nothing (CR 903.4f).
pub const COLOR_OUTSIDE_COMMANDER_IDENTITY: &str =
    "game rules:has a color outside your commander's color identity";

/// `Effect::Custom` prefix: "you may activate ~'s loyalty ability N more times this turn"
/// (`game rules:more loyalty activations:N`): N more activations of loyalty abilities of
/// the source this turn, on top of the once each turn of CR 606.3.
pub const MORE_LOYALTY: &str = "game rules:more loyalty activations:";

/// `Value::Custom` name: how many differently named dungeons the player has completed
/// (CR 309.7).
pub const DIFFERENT_DUNGEONS_COMPLETED: &str = "game rules:differently named dungeons completed";

/// Records of [`MORE_LOYALTY`]: (turn, permanent, controller, how many more).
const MORE_LOYALTY_ROWS: &str = "game rules:more loyalty activations";

/// The effect "you may activate ~'s loyalty ability `n` more times this turn".
pub fn more_loyalty_activations(n: u32) -> crate::ability::Effect {
    crate::ability::Effect::Custom(SmolStr::new(format!("{MORE_LOYALTY}{n}")))
}

/// How many more activations an effect named `name` grants, if it's [`MORE_LOYALTY`].
pub fn more_loyalty_count(name: &str) -> Option<u32> {
    name.strip_prefix(MORE_LOYALTY)?.parse().ok()
}

struct GameRules4;

impl KeywordRules for GameRules4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != COLOR_OUTSIDE_COMMANDER_IDENTITY {
            return None;
        }
        let Some(identity) = crate::commander_rules::commanders_identity(g, ctx.controller) else {
            return Some(false);
        };
        Some(g.obj(id).chars.colors.iter().any(|c| !identity.contains(c)))
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(n) = more_loyalty_count(name) else {
            return false;
        };
        let Some(src) = ctx.source.map(|s| g.ability_source_of(s)) else {
            return true;
        };
        let src = g.current(src);
        let turn = i64::from(g.turn.number);
        g.cards.retain(MORE_LOYALTY_ROWS, |r| r[0] == turn);
        g.cards.push(
            MORE_LOYALTY_ROWS,
            vec![
                turn,
                i64::from(src.0),
                i64::from(ctx.controller.0),
                i64::from(n),
            ],
        );
        true
    }

    fn extra_loyalty_activations(&self, g: &Game, p: PlayerId, src: ObjectId) -> u32 {
        let turn = i64::from(g.turn.number);
        g.cards
            .get(MORE_LOYALTY_ROWS)
            .iter()
            .filter(|r| r[0] == turn && r[1] == i64::from(src.0) && r[2] == i64::from(p.0))
            .map(|r| r[3] as u32)
            .sum()
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != DIFFERENT_DUNGEONS_COMPLETED {
            return None;
        }
        let mut names: Vec<&SmolStr> = g.player(ctx.controller).completed_dungeons.iter().collect();
        names.sort();
        names.dedup();
        Some(names.len() as i64)
    }
}

inventory::submit! { KeywordRegistration(&GameRules4) }
