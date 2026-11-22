//! A "loses the game" triggered ability whose source leaves the game with the losing
//! player doesn't trigger on that player's loss.
//!
//! Abilities that trigger when a player loses the game look back in time (CR 603.10f), and
//! when a player leaves a multiplayer game every object they own leaves with them (CR
//! 800.4a), so the loss is checked before those objects leave (`Game::player_loses`) and
//! objects that stay in the game but watch the loser's things keep triggering (Thieving
//! Amalgam, Grismold, the Dreadsower: a creature dying as its owner leaves). The exception
//! is the object that carries the trigger and itself leaves with the loser: "If you control
//! an opponent's Share the Spoils and that player loses the game, Share the Spoils leaves
//! the game with them and the first ability won't trigger." (Share the Spoils ruling,
//! 2021-07-23). Decided by the user, Nov 10 (interpretation
//! `loses-the-game-trigger-leaves-with-the-loser`).
//!
//! The case is found through the look-back check ([`crate::triggers::looks_back`]), never
//! by card: a look-back trigger on the loss, carried by an object the loser owns that the
//! leave-the-game procedure removes.

use crate::ability::TriggerCond;
use crate::events::Event;
use crate::game::Game;
use crate::object::Zone;
use crate::types::ObjectId;

/// Whether `source`'s triggered ability with condition `cond` doesn't trigger on `ev`
/// because the object leaves the game with the player the event says lost (CR 800.4a).
pub fn suppressed(g: &Game, source: ObjectId, cond: &TriggerCond, ev: &Event) -> bool {
    let Event::PlayerLost { player } = ev else {
        return false;
    };
    if !crate::triggers::looks_back(cond, ev) {
        return false;
    }
    let o = g.obj(source);
    // Objects in the ante zone stay (CR 800.4n), and nothing leaves when the game ends with
    // the loss (CR 104.2a): the procedure of 800.4a is for a game that goes on without the
    // player (the loser has already been counted out of `players_in_game`).
    o.owner == *player
        && !matches!(o.zone, Zone::Nowhere | Zone::Ante)
        && g.players_in_game().len() > 1
}
