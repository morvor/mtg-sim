//! "When each opponent casts their first spell of the game, ..." (Chancellor of the
//! Annex): a trigger on a player's first spell of the whole game (spells cast are kept
//! for the game, `turn_start::kept_for_game`). Created at the game's start by an
//! opening-hand action (CR 103.6b, 603.7g), it triggers once for each opponent.

use super::{KeywordRegistration, KeywordRules};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::EventInfo;
use crate::types::*;

/// `TriggerCond::Custom`: "when each opponent casts their first spell of the game". The
/// event's spell is "that spell" and its caster "that player".
pub const OPPONENT_FIRST_SPELL_OF_GAME: &str = "cast: each opponent's first spell of the game";

/// How many spells `p` has cast this game (this turn's included).
fn spells_cast_this_game(g: &Game, p: PlayerId) -> usize {
    let earlier = g
        .game_events
        .iter()
        .filter(|e| matches!(e, Event::SpellCast { player, .. } if *player == p))
        .count();
    earlier + g.history.spells_cast.iter().filter(|(q, _)| *q == p).count()
}

struct FirstSpellOfGame;

impl KeywordRules for FirstSpellOfGame {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != OPPONENT_FIRST_SPELL_OF_GAME {
            return None;
        }
        Some(match ev {
            Event::SpellCast { spell, player, .. }
                if g.are_opponents(ctl, *player) && spells_cast_this_game(g, *player) == 1 =>
            {
                vec![EventInfo {
                    object: Some(*spell),
                    spell: Some(*spell),
                    player: Some(*player),
                    ..Default::default()
                }]
            }
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&FirstSpellOfGame) }
