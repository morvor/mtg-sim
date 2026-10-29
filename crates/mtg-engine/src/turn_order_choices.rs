//! "Starting with you, each player chooses a creature.", "starting with the next opponent
//! in turn order, each opponent chooses a creature card in your graveyard that hasn't been
//! chosen" ([`Effect::InTurnOrder`]): the players make their choices one at a time, in turn
//! order beginning with the named player (CR 101.4b: a player making a choice in turn
//! order knows the choices made before), each performing the whole instruction before the
//! next one starts. "Starting with you and proceeding in the chosen direction" goes around
//! the table that way instead (see `seats.rs`); turn order is to the left (CR 101.4).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::PlayerId;

/// The players matching `who`, in turn order starting with `first` (CR 101.4b), or around
/// the table in `direction`.
pub fn order(
    g: &Game,
    first: TurnOrderStart,
    direction: SeatSide,
    who: &PlayerFilter,
    ctx: &Ctx,
) -> Vec<PlayerId> {
    let n = g.players.len();
    let you = ctx.controller;
    let start = match first {
        TurnOrderStart::You => you,
        // The next player after you in turn order who is your opponent.
        TurnOrderStart::NextOpponent => (1..n)
            .map(|i| g.seat_in_turn_order(you, i))
            .find(|p| g.player(*p).in_game() && g.opponents(you).contains(p))
            .unwrap_or(you),
    };
    // The default direction (`SeatSide::Left`) is turn order, which goes to the right once
    // the game's turn order has been reversed; a chosen direction (or right) goes around
    // the table that way. An undefined direction (none was chosen) is taken as turn order.
    let seat_side = match direction {
        SeatSide::Left => None,
        side => crate::seats::resolve_side(g, side, ctx.source),
    };
    (0..n)
        .map(|i| match seat_side {
            Some(true) => PlayerId(((start.idx() + i) % n) as u8),
            Some(false) => PlayerId(((start.idx() + n - i) % n) as u8),
            None => g.seat_in_turn_order(start, i),
        })
        .filter(|p| g.player(*p).in_game() && g.player_filter_matches(who, *p, ctx))
        .collect()
}

/// Performs `effect` for each of the players, one at a time.
pub fn run(
    g: &mut Game,
    first: TurnOrderStart,
    direction: SeatSide,
    who: &PlayerFilter,
    effect: &Effect,
    ctx: &mut Ctx,
) {
    let saved = ctx.iter_player;
    for p in order(g, first, direction, who, ctx) {
        if g.result.is_some() {
            break;
        }
        ctx.iter_player = Some(p);
        g.exec(effect, ctx);
    }
    ctx.iter_player = saved;
}
