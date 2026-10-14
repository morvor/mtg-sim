//! The shared team turns option (CR 805; always used in Two-Headed Giant, CR 810.2):
//! teams take turns rather than players (CR 805.4). The engine represents a team's turn by
//! one of its players as `turn.active`; every player on that team is an active player for
//! the turn-based actions of the turn (untapping, drawing, playing lands, discarding).

use crate::ability::{Body, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// The variable holding which active player "the active player" refers to while an
/// ability's effect applies with shared team turns (CR 805.9).
pub const ACTIVE_PLAYER_VAR: Var = u16::MAX - 805;

/// The shared team turns option can be used only if the members of each team sit in
/// adjacent seats (CR 805.1).
pub fn teams_seated_together(g: &Game) -> bool {
    let teams: Vec<u8> = g.players.iter().map(|p| p.team).collect();
    crate::multiplayer::setup::teams_together(&teams)
}

/// CR 805.4d: with shared team turns, an ability that triggers at the beginning of "each
/// player's" or "each opponent's" step or phase triggers once for each appropriate player
/// on the active team if its trigger condition, effect or intervening "if" clause refers
/// to "that player" or similar. Given the matches already found for the step (for one
/// player), the matches for the other players on the active team.
pub fn more_step_trigger_infos(
    g: &Game,
    t: &crate::ability::TriggeredAbility,
    base: &Ctx,
    ev: &crate::events::Event,
    found: &[crate::object::EventInfo],
) -> Vec<crate::object::EventInfo> {
    use crate::ability::TriggerCond;
    let TriggerCond::BeginningOf { whose, .. } = &t.trigger else {
        return vec![];
    };
    if found.is_empty()
        || !matches!(ev, crate::events::Event::StepBegan { .. })
        || !g.uses_shared_team_turns()
    {
        return vec![];
    }
    let refers = |v: serde_json::Result<String>| v.is_ok_and(|j| j.contains("TriggerPlayer"));
    if !refers(serde_json::to_string(&t.body)) && !refers(serde_json::to_string(&t.intervening_if))
    {
        return vec![];
    }
    g.active_players()
        .into_iter()
        .filter(|p| {
            g.player(*p).in_game()
                && g.player_rel_matches(*whose, *p, base)
                && !found.iter().any(|i| i.player == Some(*p))
        })
        .map(|p| crate::object::EventInfo {
            player: Some(p),
            ..Default::default()
        })
        .collect()
}

/// CR 805.9: an ability that refers to "the active player" refers to one specific active
/// player; its controller chooses which as its effect is applied.
pub fn choose_active_player(g: &mut Game, body: &Body, ctx: &mut Ctx) {
    if !g.uses_shared_team_turns() || ctx.vars.contains_key(&ACTIVE_PLAYER_VAR) {
        return;
    }
    let actives: Vec<PlayerId> = g
        .active_players()
        .into_iter()
        .filter(|p| g.player(*p).in_game())
        .collect();
    if actives.len() < 2 {
        return;
    }
    let mentions = serde_json::to_string(body).is_ok_and(|j| j.contains("\"ActivePlayer\""));
    if !mentions {
        return;
    }
    let chosen = g
        .ask_entities(
            ctx.controller,
            ctx.source,
            "Choose the active player this refers to",
            actives.iter().map(|p| Entity::Player(*p)).collect(),
            1,
            1,
        )
        .first()
        .and_then(|e| e.player())
        .unwrap_or(actives[0]);
    ctx.set_var(ACTIVE_PLAYER_VAR, vec![Entity::Player(chosen)]);
}

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
