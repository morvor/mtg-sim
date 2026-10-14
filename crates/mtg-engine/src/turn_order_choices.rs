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
    let you = ctx.controller.idx();
    let start = match first {
        TurnOrderStart::You => you,
        // The next player after you in turn order who is your opponent.
        TurnOrderStart::NextOpponent => (1..n)
            .map(|i| (you + i) % n)
            .find(|&i| {
                let p = PlayerId(i as u8);
                g.player(p).in_game() && g.opponents(ctx.controller).contains(&p)
            })
            .unwrap_or(you),
    };
    // An undefined direction (none was chosen) is taken as turn order.
    let left = crate::seats::resolve_side(g, direction, ctx.source).unwrap_or(true);
    (0..n)
        .map(|i| {
            let k = if left { start + i } else { start + n - i };
            PlayerId((k % n) as u8)
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
