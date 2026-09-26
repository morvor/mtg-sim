//! The shared team turns option (CR 805; always used in Two-Headed Giant, CR 810.2):
//! teams take turns rather than players (CR 805.4). The engine represents a team's turn by
//! one of its players as `turn.active`; every player on that team is an active player for
//! the turn-based actions of the turn (untapping, drawing, playing lands, discarding).

use crate::game::Game;
use crate::types::*;

impl Game {
    /// Whether the game uses the shared team turns option (CR 805).
    pub fn uses_shared_team_turns(&self) -> bool {
        crate::combat::shared_team_turns(self)
    }

    /// The players whose turn it is: the active player, or with shared team turns every
    /// player still in the game on the active team (CR 805.4a), in seat order.
    pub fn active_players(&self) -> Vec<PlayerId> {
        let a = self.turn.active;
        if !self.uses_shared_team_turns() {
            return vec![a];
        }
        let team = self.player(a).team;
        let mut v: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|p| p.team == team && (p.in_game() || p.id == a))
            .map(|p| p.id)
            .collect();
        // The player representing the turn first.
        v.retain(|p| *p != a);
        v.insert(0, a);
        v
    }

    /// Whether it's `p`'s turn: `p` is the active player or, with shared team turns, on the
    /// active team (CR 805.4a).
    pub fn is_active_player(&self, p: PlayerId) -> bool {
        p == self.turn.active
            || (self.uses_shared_team_turns()
                && self.player(p).team == self.player(self.turn.active).team
                && self.player(p).in_game())
    }

    /// Who takes the turn after `after`'s: the next player in turn order still in the game
    /// or, with shared team turns, the first player of the next team in turn order
    /// (CR 805.4).
    pub fn next_turn_player(&self, after: PlayerId) -> PlayerId {
        if !self.uses_shared_team_turns() {
            return self.next_player(after);
        }
        let n = self.players.len();
        let team = self.player(after).team;
        for i in 1..=n {
            let q = PlayerId(((after.idx() + i) % n) as u8);
            if self.player(q).in_game() && self.player(q).team != team {
                return q;
            }
        }
        self.next_player(after)
    }

    /// The first player (in seat order) of each team, the team's representative.
    pub fn team_representatives(&self) -> Vec<PlayerId> {
        let mut seen = Vec::new();
        let mut out = Vec::new();
        for p in &self.players {
            if !seen.contains(&p.team) {
                seen.push(p.team);
                out.push(p.id);
            }
        }
        out
    }

    /// The order in which players act while starting the game: the starting player, who is
    /// the active player, then the others in turn order (CR 101.4e, 103.5, 103.6). With
    /// shared team turns, every player on the starting team first, then the players on
    /// each other team in turn order (CR 103.5d, 103.6c).
    pub fn pregame_order(&self) -> Vec<PlayerId> {
        let order = self.apnap();
        if !self.uses_shared_team_turns() {
            return order;
        }
        let mut teams: Vec<u8> = Vec::new();
        for p in &order {
            let t = self.player(*p).team;
            if !teams.contains(&t) {
                teams.push(t);
            }
        }
        teams
            .into_iter()
            .flat_map(|t| {
                order
                    .iter()
                    .copied()
                    .filter(move |p| self.player(*p).team == t)
            })
            .collect()
    }
}
