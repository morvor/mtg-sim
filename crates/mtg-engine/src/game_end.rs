//! Ending the game (CR 104): winning, losing, and draws — including teams (CR 104.2c,
//! 104.3g, 104.4d, 104.4g), the Emperor variant (CR 104.2d, 104.3i, 104.4h), the limited
//! range of influence option (CR 104.3h, 104.4e, 104.4f) and loops of mandatory actions
//! (CR 104.4b).

use crate::ability::*;
use crate::events::Event;
use crate::game::{Game, GameResult, Variant};
use crate::replacement::ReplEvent;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

/// `Effect::Custom` name of "the game is a draw" (CR 104.4c).
pub const DRAW_EFFECT: &str = "game is a draw";

/// How many times the game state may repeat during a loop of mandatory actions before the
/// loop is recognized (CR 104.4b).
const LOOP_REPEATS: u32 = 3;

/// Forced priority passes in a row before repeated game states are looked for.
const LOOP_MIN_LEN: u32 = 8;

/// Game-ending bookkeeping.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EndState {
    /// Players for whom the game was a draw (CR 104.4e, 104.4f); they left the game.
    pub drew: BTreeSet<PlayerId>,
    /// Fingerprints of the game states seen at priority during the current stretch of
    /// mandatory actions (CR 104.4b), with how often each was seen.
    #[serde(skip)]
    pub loop_states: BTreeMap<u64, u32>,
    /// Objects (by controller) involved in the current stretch of mandatory actions.
    #[serde(skip)]
    pub loop_controllers: BTreeSet<PlayerId>,
    /// `actions_taken` at the last forced priority pass, to notice other decisions.
    #[serde(skip)]
    pub loop_mark: u64,
    /// Forced priority passes in the current stretch of mandatory actions.
    #[serde(skip)]
    pub loop_len: u32,
}

impl Game {
    /// CR 100.1a: a game that begins with only two players.
    pub fn is_two_player(&self) -> bool {
        self.players.len() == 2
    }

    /// CR 100.1b: a game that begins with more than two players.
    pub fn is_multiplayer(&self) -> bool {
        self.players.len() > 2
    }

    /// Whether this is a multiplayer game between teams (CR 102.3): some team has more
    /// than one player.
    pub fn has_teams(&self) -> bool {
        let mut seen = BTreeSet::new();
        self.players.iter().any(|p| !seen.insert(p.team))
    }

    /// Every player on `p`'s team, including `p` and players who have left the game, in
    /// seat order.
    pub fn team_members(&self, p: PlayerId) -> Vec<PlayerId> {
        let team = self.player(p).team;
        self.players
            .iter()
            .filter(|q| q.team == team)
            .map(|q| q.id)
            .collect()
    }

    /// The emperor of `p`'s team in the Emperor variant (CR 809.2): the player seated in
    /// the middle of the team.
    pub fn emperor_of(&self, p: PlayerId) -> Option<PlayerId> {
        if self.config.variant != Variant::Emperor {
            return None;
        }
        let members = self.team_members(p);
        let n = self.players.len();
        // Teams sit together; start from the member whose right neighbor isn't on the team.
        let team = self.player(p).team;
        let first = members
            .iter()
            .copied()
            .find(|q| self.players[(q.idx() + n - 1) % n].team != team)
            .unwrap_or(members[0]);
        let ordered: Vec<PlayerId> = (0..members.len())
            .map(|i| PlayerId(((first.idx() + i) % n) as u8))
            .collect();
        ordered.get((ordered.len().max(1) - 1) / 2).copied()
    }

    /// Whether `p` is an emperor (CR 809.2).
    pub fn is_emperor(&self, p: PlayerId) -> bool {
        self.emperor_of(p) == Some(p)
    }

    /// A player's range of influence (CR 801.2), `None` if unlimited. In the Emperor
    /// variant it's 2 for emperors and 1 for generals unless the game sets one (CR 809.3a).
    pub fn range_of_influence(&self, p: PlayerId) -> Option<u32> {
        if let Some(n) = self.config.range_of_influence {
            return Some(n);
        }
        if self.config.variant == Variant::Emperor {
            return Some(if self.is_emperor(p) { 2 } else { 1 });
        }
        None
    }

    /// Players still in the game within `p`'s range of influence, including `p`
    /// (CR 801.2, 801.2b).
    pub fn players_in_range(&self, p: PlayerId) -> Vec<PlayerId> {
        self.players_in_game()
            .into_iter()
            .filter(|q| crate::combat::within_range(self, p, *q))
            .collect()
    }

    /// Whether an effect says `p` (or, in Two-Headed Giant, a teammate) can't win the game
    /// (CR 101.2, 810.8a).
    pub fn cant_win_game(&self, p: PlayerId) -> bool {
        self.team_scope(p)
            .into_iter()
            .any(|q| self.player_restricted(q, |r| matches!(r, Restriction::CantWinGame(_))))
    }

    /// Whether an effect says `p` (or, in Two-Headed Giant, a teammate) can't lose the
    /// game (CR 101.2, 810.8a).
    pub fn cant_lose_game(&self, p: PlayerId) -> bool {
        self.team_scope(p).into_iter().any(|q| {
            self.player_restricted(q, |r| matches!(r, Restriction::CantLoseGame(_)))
                || self
                    .player(q)
                    .has_mod(|m| matches!(m, PlayerModification::CantLoseGame))
        })
    }

    /// The players whose "can't win/lose" effects apply to `p`: in Two-Headed Giant, the
    /// whole team (CR 810.8a).
    fn team_scope(&self, p: PlayerId) -> Vec<PlayerId> {
        if self.config.variant == Variant::TwoHeadedGiant {
            self.team_members(p)
        } else {
            vec![p]
        }
    }

    /// A player would lose the game because of a state-based action or an effect
    /// (CR 104.3b–e): "can't lose" effects win (CR 101.2), then replacement effects apply
    /// (CR 614). Conceding doesn't go through here (CR 104.3a).
    pub fn lose_game(&mut self, p: PlayerId) {
        if !self.player(p).in_game() || self.result.is_some() || self.cant_lose_game(p) {
            return;
        }
        for e in self.replace(ReplEvent::LoseGame { player: p }) {
            self.execute_repl_event(e);
        }
    }

    /// Several players lose the game at the same time (CR 104.4a): the result is decided
    /// only once all of them have lost.
    pub fn lose_game_simultaneously(&mut self, ps: &[PlayerId]) {
        let was = self.losing_simultaneously;
        self.losing_simultaneously = true;
        for &p in ps {
            self.lose_game(p);
        }
        self.losing_simultaneously = was;
        if !was {
            self.check_game_over();
        }
    }

    /// Players win the game at the same time because of an effect (CR 104.2b). A player who
    /// can't win doesn't (CR 101.2). With the limited range of influence option, each
    /// winner's opponents within their range of influence lose instead (CR 104.3h,
    /// 801.14). Otherwise the winners' teams win and everyone else loses; a player who
    /// would both win and lose at once loses (CR 104.3f).
    pub fn players_win(&mut self, ps: &[PlayerId]) {
        if self.result.is_some() {
            return;
        }
        let winners: Vec<PlayerId> = ps
            .iter()
            .copied()
            .filter(|p| self.player(*p).in_game() && !self.cant_win_game(*p))
            .collect();
        if winners.is_empty() {
            return;
        }
        let limited = winners
            .iter()
            .any(|p| self.range_of_influence(*p).is_some());
        if limited {
            let mut losers: Vec<PlayerId> = Vec::new();
            for &w in &winners {
                for q in self.players_in_range(w) {
                    if self.are_opponents(w, q) && !losers.contains(&q) {
                        losers.push(q);
                    }
                }
            }
            self.lose_game_simultaneously(&losers);
            return;
        }
        let win_teams: BTreeSet<u8> = winners.iter().map(|p| self.player(*p).team).collect();
        // Each winner's opponents lose; a winner who is also an opponent of another winner
        // both wins and loses, so loses (CR 104.3f).
        let losers: Vec<PlayerId> = self
            .players_in_game()
            .into_iter()
            .filter(|q| win_teams.iter().any(|t| *t != self.player(*q).team))
            .collect();
        self.losing_simultaneously = true;
        for &q in &losers {
            self.players[q.idx()].has_lost = true;
            self.emit(Event::PlayerLost { player: q });
        }
        self.losing_simultaneously = false;
        self.check_game_over();
    }

    /// CR 104.2a, 104.2c: the game ends when only one team is left (a team with at least
    /// one player still in the game whose opponents have all left wins; each player on that
    /// team wins, even one who had lost), or when no player is left (a draw, CR 104.4a,
    /// 104.4d). In the Emperor variant a team wins with its emperor (CR 104.2d).
    pub fn decide_game_over(&mut self) {
        if self.result.is_some() {
            return;
        }
        let remaining = self.players_in_game();
        let teams: BTreeSet<u8> = remaining.iter().map(|p| self.player(*p).team).collect();
        if remaining.is_empty() {
            self.result = Some(GameResult::Draw);
        } else if teams.len() == 1 {
            let winners: Vec<PlayerId> = self
                .team_members(remaining[0])
                .into_iter()
                .filter(|p| !self.end.drew.contains(p))
                .collect();
            for p in &winners {
                self.players[p.idx()].has_won = true;
            }
            self.result = Some(GameResult::Win(winners));
        }
    }

    /// Extra losses when a player loses: in the Emperor variant, a team loses the game if
    /// its emperor loses (CR 104.3i, 809.5b).
    pub(crate) fn on_player_lost(&mut self, p: PlayerId) {
        if self.is_emperor(p) {
            let generals: Vec<PlayerId> = self
                .team_members(p)
                .into_iter()
                .filter(|q| *q != p && self.player(*q).in_game())
                .collect();
            let was = self.losing_simultaneously;
            self.losing_simultaneously = true;
            for q in generals {
                self.player_loses(q);
            }
            self.losing_simultaneously = was;
        }
    }

    /// The effect of a spell or ability controlled by `controller` states that the game is
    /// a draw (CR 104.4c). With the limited range of influence option the game is a draw
    /// only for the controller and the players within their range of influence, who leave
    /// the game (CR 104.4e, 801.15).
    pub fn game_is_a_draw(&mut self, controller: PlayerId) {
        if self.result.is_some() {
            return;
        }
        if self.range_of_influence(controller).is_some() && self.is_multiplayer() {
            let ps = self.players_in_range(controller);
            self.draw_for(&ps);
        } else {
            self.draw_game();
        }
    }

    /// The game is a draw for these players (CR 104.4e, 104.4f): they leave the game
    /// (CR 104.5). In the Emperor variant the game is a draw for a team if it's a draw for
    /// its emperor (CR 104.4h). A team for which the game is a draw for all remaining
    /// players has drawn (CR 104.4g).
    pub fn draw_for(&mut self, ps: &[PlayerId]) {
        let mut all: Vec<PlayerId> = Vec::new();
        for &p in ps {
            if !self.player(p).in_game() {
                continue;
            }
            all.push(p);
            if self.is_emperor(p) {
                for q in self.team_members(p) {
                    if self.player(q).in_game() && !all.contains(&q) {
                        all.push(q);
                    }
                }
            }
        }
        // Everyone left: the game is a draw.
        if self.players_in_game().iter().all(|p| all.contains(p)) {
            self.draw_game();
            return;
        }
        for &p in &all {
            self.end.drew.insert(p);
            self.log(|_g| format!("the game is a draw for {p}"));
            self.after_player_leaves(p);
        }
        self.check_game_over();
    }

    /// In a tournament, a judge's penalty makes a player lose the game (CR 104.3k). Like
    /// conceding, this isn't stopped by effects that say the player can't lose.
    pub fn game_loss_penalty(&mut self, p: PlayerId) {
        self.log(|_g| format!("{p} receives a game loss penalty"));
        self.player_loses(p);
    }

    /// In a tournament, all players in the game may agree to an intentional draw
    /// (CR 104.4i): each player still in the game is asked, and the game is a draw only if
    /// all of them agree. Returns whether it was.
    pub fn propose_intentional_draw(&mut self) -> bool {
        if self.result.is_some() {
            return false;
        }
        for p in self.apnap() {
            if !self.ask_yes_no(p, None, "Agree to an intentional draw?", false) {
                return false;
            }
        }
        self.draw_game();
        true
    }

    /// Whether the game was a draw for `p` (CR 104.4).
    pub fn drew_game(&self, p: PlayerId) -> bool {
        self.end.drew.contains(&p) || self.result == Some(GameResult::Draw)
    }

    // ------------------------------------------------------------------
    // Loops of mandatory actions (CR 104.4b, 104.4f)
    // ------------------------------------------------------------------

    /// Called when a player receives priority. If they can do nothing but pass while
    /// something is on the stack, and no other decision was made since the last such
    /// priority, the game is inside a stretch of mandatory actions. If the game state then
    /// repeats, the game has entered a loop of mandatory actions with no way to stop it:
    /// the game is a draw (CR 104.4b) — with the limited range of influence option, only
    /// for the players controlling objects involved in the loop and those within their
    /// ranges of influence (CR 104.4f, 801.16). Returns true if the loop ended the game
    /// for someone.
    pub fn check_mandatory_loop(&mut self, forced: bool) -> bool {
        // Any other decision since the last forced pass was an optional action, or the
        // stretch of mandatory actions ended.
        let interrupted = self.actions_taken != self.end.loop_mark;
        // The priority decision about to be asked counts as one action.
        self.end.loop_mark = self.actions_taken + 1;
        if interrupted || !forced || self.stack.is_empty() {
            self.end.loop_states.clear();
            self.end.loop_controllers.clear();
            self.end.loop_len = 0;
        }
        if !forced || self.stack.is_empty() {
            return false;
        }
        for s in self.stack.clone() {
            let c = self.obj(s).controller;
            self.end.loop_controllers.insert(c);
        }
        // Ordinary stretches of passing are short; only look for repeated states in long
        // ones.
        self.end.loop_len += 1;
        if self.end.loop_len < LOOP_MIN_LEN {
            return false;
        }
        let fp = self.loop_fingerprint();
        let n = self.end.loop_states.entry(fp).or_insert(0);
        *n += 1;
        if *n < LOOP_REPEATS {
            return false;
        }
        self.end.loop_states.clear();
        let involved: Vec<PlayerId> = std::mem::take(&mut self.end.loop_controllers)
            .into_iter()
            .collect();
        self.log(|_g| "a loop of mandatory actions".to_string());
        let limited = involved
            .iter()
            .any(|p| self.range_of_influence(*p).is_some())
            && self.is_multiplayer();
        if limited {
            let mut ps: Vec<PlayerId> = Vec::new();
            for p in involved {
                for q in self.players_in_range(p) {
                    if !ps.contains(&q) {
                        ps.push(q);
                    }
                }
            }
            self.draw_for(&ps);
        } else {
            self.draw_game();
        }
        true
    }

    /// A fingerprint of the game state that doesn't depend on object ids (which change with
    /// every zone change, CR 400.7), for recognizing repeated states.
    fn loop_fingerprint(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        let obj = |id: ObjectId| {
            let o = self.obj(id);
            format!(
                "{}|{}|{}|{}|{:?}|{}",
                o.chars.name, o.controller.0, o.tapped, o.damage, o.counters, o.face_down
            )
        };
        let sorted = |ids: &[ObjectId]| {
            let mut v: Vec<String> = ids.iter().map(|i| obj(*i)).collect();
            v.sort();
            v
        };
        self.turn.number.hash(&mut h);
        format!("{:?}", self.turn.step).hash(&mut h);
        self.turn.priority.map(|p| p.0).hash(&mut h);
        for p in &self.players {
            p.life.hash(&mut h);
            p.in_game().hash(&mut h);
            format!("{:?}", p.counters).hash(&mut h);
            p.library.len().hash(&mut h);
            sorted(&p.hand).hash(&mut h);
            sorted(&p.graveyard).hash(&mut h);
        }
        sorted(&self.battlefield).hash(&mut h);
        sorted(&self.exile).hash(&mut h);
        for s in &self.stack {
            obj(*s).hash(&mut h);
        }
        h.finish()
    }

    /// The settle loop (CR 117.5) couldn't finish: state-based actions and triggered
    /// abilities keep happening with no player ever receiving priority, a loop of
    /// mandatory actions (CR 104.4b).
    pub(crate) fn endless_settle_loop(&mut self) {
        self.log(|_g| "a loop of mandatory state-based actions".to_string());
        self.draw_game();
    }
}

/// Runs a named custom effect from this module, if it is one. Returns true if handled.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &crate::eval::Ctx) -> bool {
    if name == DRAW_EFFECT {
        g.game_is_a_draw(ctx.controller);
        return true;
    }
    false
}
