//! What things were like as the turn began, and what happened earlier in the game:
//!
//! * "if you were the monarch as the turn began" (Knights of the Black Rose), "if you had
//!   no cards in hand at the beginning of this turn" (Mindstorm Crown), "the number of
//!   untapped lands they controlled at the beginning of this turn" (Power Surge), "that
//!   player's life total as the turn began" (Sengir, the Dark Baron): each player's state
//!   is recorded as each turn begins, before its untap step (CR 501.1, 502).
//!   [`Value::AsTurnBegan`].
//! * "this game" (`HistoryWindow::ThisGame`): the events of earlier turns that such a
//!   query can ask about (spells cast, cards cycled, damage dealt) are kept for the whole
//!   game ([`Game::game_events`]).

use crate::ability::{TurnStartStat, Value};
use crate::events::Event;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// One player's state as the turn began.
#[derive(Clone, Debug, Default)]
pub struct PlayerAtTurnStart {
    pub life: i32,
    pub hand: usize,
    pub monarch: bool,
    pub untapped_lands: usize,
}

/// Each player's state as the current turn began (indexed by player).
#[derive(Clone, Debug, Default)]
pub struct TurnStart {
    pub players: Vec<PlayerAtTurnStart>,
}

/// Records each player's state as a turn begins.
pub fn record(g: &mut Game) {
    let players = (0..g.players.len())
        .map(|i| {
            let p = PlayerId(i as u8);
            PlayerAtTurnStart {
                life: g.player(p).life,
                hand: g.player(p).hand.len(),
                monarch: g.monarch == Some(p),
                untapped_lands: g
                    .battlefield
                    .iter()
                    .filter(|o| {
                        let o = g.obj(**o);
                        o.zone == Zone::Battlefield
                            && o.controller == p
                            && !o.tapped
                            && o.is(CardType::Land)
                    })
                    .count(),
            }
        })
        .collect();
    g.turn_start = TurnStart { players };
}

/// A player's state as the turn began (the current state if none was recorded: the
/// game's first turn set up without beginning it).
pub fn stat(g: &Game, stat: TurnStartStat, p: PlayerId) -> i64 {
    let rec = g
        .turn_start
        .players
        .get(p.idx())
        .cloned()
        .unwrap_or_else(|| PlayerAtTurnStart {
            life: g.player(p).life,
            hand: g.player(p).hand.len(),
            monarch: g.monarch == Some(p),
            untapped_lands: 0,
        });
    match stat {
        TurnStartStat::Life => rec.life as i64,
        TurnStartStat::HandSize => rec.hand as i64,
        TurnStartStat::Monarch => rec.monarch as i64,
        TurnStartStat::UntappedLands => rec.untapped_lands as i64,
    }
}

/// [`Value::AsTurnBegan`]: the sum over the players the reference names.
pub fn value(g: &Game, v: &Value, ctx: &crate::eval::Ctx) -> i64 {
    let Value::AsTurnBegan(s, who) = v else {
        return 0;
    };
    g.eval_players(who, ctx)
        .into_iter()
        .map(|p| stat(g, *s, p))
        .sum()
}

/// Whether an event of an earlier turn is kept for "this game" queries.
pub fn kept_for_game(e: &Event) -> bool {
    match e {
        Event::SpellCast { .. } | Event::Cycled { .. } | Event::Damage { .. } => true,
        _ => false,
    }
}
