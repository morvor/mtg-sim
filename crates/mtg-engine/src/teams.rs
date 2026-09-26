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

    /// Each team's primary player (CR 805.2), in the order the teams first appear in seat
    /// order: the team's representative, who takes the team's turns and makes the team's
    /// choices when its players can't agree.
    pub fn team_representatives(&self) -> Vec<PlayerId> {
        let mut seen = Vec::new();
        let mut out = Vec::new();
        for p in &self.players {
            if !seen.contains(&p.team) {
                seen.push(p.team);
                out.push(self.primary_player(p.id));
            }
        }
        out
    }

    /// The primary player of `p`'s team (CR 805.2): the player seated in the team's
    /// rightmost seat from its perspective.
    pub fn primary_player(&self, p: PlayerId) -> PlayerId {
        crate::multiplayer::setup::primary_player(self, p)
    }

    /// Groups of players who put their triggered abilities on the stack together, in
    /// order, each with the player who chooses the order: every player on their own in
    /// APNAP order (CR 603.3b) or, with shared team turns, the active team's players, then
    /// each nonactive team's in turn order, each team ordering all of its members'
    /// abilities as it likes — its primary player deciding (CR 805.7, 805.2).
    pub fn trigger_groups(&self) -> Vec<(PlayerId, Vec<PlayerId>)> {
        let order = self.apnap();
        if !self.uses_shared_team_turns() {
            return order.into_iter().map(|p| (p, vec![p])).collect();
        }
        let mut groups: Vec<(PlayerId, Vec<PlayerId>)> = Vec::new();
        for p in order {
            let team = self.player(p).team;
            match groups
                .iter_mut()
                .find(|(c, _)| self.player(*c).team == team)
            {
                Some((_, v)) => v.push(p),
                None => groups.push((p, vec![p])),
            }
        }
        for (chooser, members) in groups.iter_mut() {
            let primary = self.primary_player(members[0]);
            if members.contains(&primary) {
                *chooser = primary;
            }
        }
        groups
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
