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
use crate::types::{Entity, PlayerId};

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

/// [`Effect::ChooseForEachPlayer`]: the choices for each player first (in turn order,
/// each knowing the earlier ones, CR 101.4b), then the action on all the chosen objects at
/// once, then the instruction for each of them about its player (Nihiloor's ruling).
#[allow(clippy::too_many_arguments)]
pub fn choose_for_each_player(
    g: &mut Game,
    first: TurnOrderStart,
    who: &PlayerFilter,
    chooser: &PlayerRef,
    filter: &Filter,
    act: &Effect,
    done: Var,
    each: &Effect,
    ctx: &mut Ctx,
) {
    let saved_iter = ctx.iter_player;
    let mut chosen: Vec<(PlayerId, Entity)> = Vec::new();
    for p in order(g, first, SeatSide::Left, who, ctx) {
        if g.result.is_some() {
            break;
        }
        ctx.iter_player = Some(p);
        let Some(c) = g.eval_player(chooser, ctx) else {
            continue;
        };
        let candidates: Vec<Entity> = g
            .eval_sel(&Sel::All(filter.clone()), ctx)
            .into_iter()
            .filter(|e| !chosen.iter().any(|(_, x)| x == e))
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let prompt = format!("Choose up to one for {p}");
        if let Some(e) = g
            .ask_entities(c, ctx.source, &prompt, candidates, 0, 1)
            .first()
        {
            chosen.push((p, *e));
        }
    }
    ctx.iter_player = saved_iter;
    if g.result.is_some() {
        return;
    }
    // The action, once, on all of them.
    ctx.set_var(vars::IT, chosen.iter().map(|(_, e)| *e).collect());
    ctx.set_var(done, vec![]);
    g.exec(act, ctx);
    let affected = ctx.vars.get(&done).cloned().unwrap_or_default();
    // Then each one's own instruction, about the player it was chosen for.
    for (p, e) in chosen {
        if g.result.is_some() {
            break;
        }
        ctx.iter_player = Some(p);
        ctx.set_var(vars::IT, vec![e]);
        let this_one = if affected.contains(&e) {
            vec![e]
        } else {
            vec![]
        };
        ctx.set_var(done, this_one);
        g.exec(each, ctx);
    }
    ctx.iter_player = saved_iter;
}
