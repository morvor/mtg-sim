//! Behaviors for the grammar of `oracle/patterns/dice_coins_4.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` name: the controller chooses a creature on the battlefield or a player
/// in the game (not a target) and it's stored in [`CHOSEN_CREATURE_OR_PLAYER`] ("~ deals 2
/// damage to a creature or player").
pub const CHOOSE_CREATURE_OR_PLAYER: &str = "dice coins 4:choose a creature or player";
/// The creature or player chosen by [`CHOOSE_CREATURE_OR_PLAYER`].
pub const CHOSEN_CREATURE_OR_PLAYER: Var = vars::USER + 43000;

pub fn choose_creature_or_player() -> Effect {
    Effect::Custom(SmolStr::new(CHOOSE_CREATURE_OR_PLAYER))
}

struct DiceCoins4;

impl KeywordRules for DiceCoins4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            CHOOSE_CREATURE_OR_PLAYER => {
                let mut cands: Vec<Entity> = g
                    .zone_objects(Zone::Battlefield)
                    .into_iter()
                    .filter(|o| g.obj(*o).is(CardType::Creature))
                    .map(Entity::Object)
                    .collect();
                cands.extend(
                    g.player_ids()
                        .into_iter()
                        .filter(|p| g.player(*p).in_game())
                        .map(Entity::Player),
                );
                let pick = g.ask_entities(
                    ctx.controller,
                    ctx.source,
                    "Choose a creature or player",
                    cands,
                    1,
                    1,
                );
                ctx.set_var(CHOSEN_CREATURE_OR_PLAYER, pick);
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&DiceCoins4) }
