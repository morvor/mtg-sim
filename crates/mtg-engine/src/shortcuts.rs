//! Taking shortcuts and loops (CR 732).
//!
//! * A player with priority may propose a shortcut: a sequence of priority choices for
//!   all players, possibly a loop repeated a number of times ([`propose`], CR 732.1b,
//!   732.2a). Each other player in turn order accepts it or shortens it by naming a place
//!   where they'll make a different choice (CR 732.2b); then the game advances through the
//!   choices, and if it was shortened, the player who then has priority must make a
//!   different choice than proposed (CR 732.2c).
//! * A fragmented loop — players each taking independent actions that bring back the same
//!   game state — is broken by the active player (or the first player in turn order
//!   involved), who must then make a different choice (CR 732.3).
//! * No player can be forced to take an action that would end a loop: players who only
//!   pass priority, or decline to pay the [B] of an "[A] unless [B]" effect, keep a loop of
//!   otherwise mandatory actions going, and the game is a draw (CR 732.4–732.6,
//!   104.4b).

use crate::casting::Illegal;
use crate::decision::{Action, Agent, Answer, Decision, PassiveAgent};
use crate::game::Game;
use crate::turn::{Stage, Step};
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Game states repeated this many times in a stretch of optional actions make a
/// fragmented loop (CR 732.3).
const FRAGMENTED_REPEATS: u32 = 3;

/// Shortcut and loop bookkeeping, in `Game::shortcuts`.
#[derive(Clone, Debug, Default)]
pub struct ShortcutState {
    /// The choices of a shortcut being taken, in order (CR 732.2c).
    pub pending: VecDeque<(PlayerId, Action)>,
    /// The shortcut being taken couldn't continue as proposed.
    pub aborted: bool,
    /// After a shortened shortcut: the choice the player who has priority must not make
    /// (CR 732.2c).
    pub must_differ: Option<(PlayerId, Action)>,
    /// The turn and step the loop watch applies to.
    watch_key: Option<(u32, Step)>,
    /// Game states seen at priority this step, with how often.
    seen: BTreeMap<u64, u32>,
    /// Players who took actions other than passing this step.
    involved: BTreeSet<PlayerId>,
    /// The choice each player made last time in each game state.
    last_choice: BTreeMap<(u64, PlayerId), Action>,
    /// The game state of the priority decision being made.
    current: Option<u64>,
}

/// Proposes a shortcut (CR 732.2a): `proposer`, who has priority, describes a sequence of
/// priority choices for all players, `choices` repeated `times` times. The sequence must
/// be legal and predictable: each choice is made by the player who has priority then,
/// and is an action they could take (it's tried out first). Each other player in turn
/// order, starting after the proposer, accepts it or shortens it (CR 732.2b); then the
/// shortcut is taken (CR 732.2c). Returns the number of choices taken.
pub fn propose(
    g: &mut Game,
    proposer: PlayerId,
    choices: &[(PlayerId, Action)],
    times: u32,
) -> Result<usize, Illegal> {
    if g.turn.stage != Stage::Priority || g.turn.priority != Some(proposer) {
        return Err(Illegal(
            "only the player with priority may propose a shortcut".into(),
        ));
    }
    let seq: Vec<(PlayerId, Action)> = (0..times.max(1))
        .flat_map(|_| choices.iter().cloned())
        .collect();
    if seq.is_empty() {
        return Err(Illegal("an empty shortcut".into()));
    }
    let mut trial = g.clone();
    trial.set_agents(
        (0..g.players.len())
            .map(|_| Box::new(PassiveAgent) as Box<dyn Agent>)
            .collect(),
    );
    if !take(&mut trial, &seq) {
        return Err(Illegal("the shortcut's choices can't be taken".into()));
    }
    // CR 732.2b: each other player accepts (0) or names an earlier ending point.
    let mut end = seq.len();
    let n = g.players.len();
    for i in 1..n {
        let p = PlayerId(((proposer.idx() + i) % n) as u8);
        if !g.player(p).in_game() {
            continue;
        }
        let k = g.ask_number(
            p,
            None,
            "Accept the shortcut (0), or shorten it to this many choices",
            0,
            end as i64,
        ) as usize;
        if k > 0 {
            end = end.min(k);
        }
    }
    // CR 732.2c: the game advances to the last proposed ending point; the player who now
    // has priority must make a different choice than proposed.
    take(g, &seq[..end]);
    if let Some((p, a)) = seq.get(end).cloned() {
        if g.turn.priority == Some(p) {
            g.shortcuts.must_differ = Some((p, a));
        }
    }
    Ok(end)
}

/// Advances the game through the shortcut's choices. Returns false if a choice couldn't
/// be taken as proposed.
fn take(g: &mut Game, seq: &[(PlayerId, Action)]) -> bool {
    g.shortcuts.pending = seq.iter().cloned().collect();
    g.shortcuts.aborted = false;
    for _ in 0..100_000 {
        if g.shortcuts.aborted || g.result.is_some() {
            g.shortcuts.pending.clear();
            return false;
        }
        if g.shortcuts.pending.is_empty() {
            return true;
        }
        g.advance();
    }
    g.shortcuts.pending.clear();
    false
}

/// The shortcut's choice for `p`, who has priority now, if a shortcut is being taken. A
/// choice that isn't `p`'s or isn't an action `p` could take ends the shortcut.
fn next_choice(g: &mut Game, p: PlayerId, actions: &[Action]) -> Option<Action> {
    let (q, a) = g.shortcuts.pending.front().cloned()?;
    if q != p || !actions.contains(&a) {
        g.shortcuts.pending.clear();
        g.shortcuts.aborted = true;
        return None;
    }
    g.shortcuts.pending.pop_front();
    Some(a)
}

/// The choices `p` may not make now: after a shortened shortcut, the proposed one
/// (CR 732.2c); in a fragmented loop, the one that would continue it (CR 732.3).
fn forbidden(g: &mut Game, p: PlayerId) -> Vec<Action> {
    let key = (g.turn.number, g.turn.step);
    if g.shortcuts.watch_key != Some(key) {
        g.shortcuts.watch_key = Some(key);
        g.shortcuts.seen.clear();
        g.shortcuts.involved.clear();
        g.shortcuts.last_choice.clear();
    }
    let fp = g.loop_fingerprint();
    g.shortcuts.current = Some(fp);
    let n = g.shortcuts.seen.entry(fp).or_insert(0);
    *n += 1;
    let repeats = *n;
    let mut forbid: Vec<Action> = Vec::new();
    if let Some((q, a)) = g.shortcuts.must_differ.take() {
        if q == p {
            forbid.push(a);
        }
    }
    let involved = &g.shortcuts.involved;
    if repeats >= FRAGMENTED_REPEATS && involved.len() >= 2 {
        // CR 732.3: the active player, or the first player in turn order involved.
        let breaker = if involved.contains(&g.turn.active) {
            Some(g.turn.active)
        } else {
            g.apnap().into_iter().find(|q| involved.contains(q))
        };
        if breaker == Some(p) {
            if let Some(a) = g.shortcuts.last_choice.get(&(fp, p)) {
                forbid.push(a.clone());
            }
        }
    }
    forbid
}

/// Records the choice `p` made at the current decision point.
fn record_choice(g: &mut Game, p: PlayerId, action: &Action) {
    if let Some(fp) = g.shortcuts.current.take() {
        g.shortcuts.last_choice.insert((fp, p), action.clone());
    }
    if !matches!(action, Action::Pass | Action::Concede) {
        g.shortcuts.involved.insert(p);
    }
}

/// CR 732.6: a player declined to take the [B] action of an "[A] unless [B]" effect. No
/// player can be forced to take it; if none does, the loop continues as though [A] were
/// mandatory, so the decision doesn't interrupt a stretch of mandatory actions.
pub fn declined_unless(g: &mut Game) {
    g.end.loop_mark += 1;
}

impl Game {
    /// The priority action of `p`, who receives priority (CR 117), with shortcuts and
    /// loops taken into account (CR 732). `None` if a loop ended the game.
    pub(crate) fn priority_decision(&mut self, p: PlayerId) -> Option<Action> {
        let mut actions = self.legal_actions(p);
        let forbid = forbidden(self, p);
        if !forbid.is_empty() {
            let kept: Vec<Action> = actions
                .iter()
                .filter(|a| !forbid.contains(a))
                .cloned()
                .collect();
            // A player who can do nothing else passes.
            if kept.iter().any(|a| !matches!(a, Action::Concede)) {
                actions = kept;
            }
        }
        // CR 104.4b, 732.4: a loop of mandatory actions is a draw.
        let forced = actions
            .iter()
            .all(|a| matches!(a, Action::Pass | Action::Concede));
        if forced && self.check_mandatory_loop(true) {
            return None;
        }
        let (action, asked) = match next_choice(self, p, &actions) {
            Some(a) => (a, false),
            None => {
                let answer = self.ask(
                    p,
                    Decision::Priority {
                        actions: actions.clone(),
                    },
                );
                let a = match answer {
                    Answer::Action(a) => a,
                    _ => Action::Pass,
                };
                // A choice the player may not make now is replaced by one they may.
                let a = if forbid.contains(&a) && !actions.contains(&a) {
                    actions.first().cloned().unwrap_or(Action::Pass)
                } else {
                    a
                };
                (a, true)
            }
        };
        if !forced {
            // CR 732.5: no player can be forced to take an action that would end a loop:
            // a player who passes keeps a loop of mandatory actions going; any other
            // action ends the stretch.
            let passing = matches!(action, Action::Pass);
            if asked {
                self.actions_taken -= 1;
            }
            let ended = self.check_mandatory_loop(passing);
            if asked {
                self.actions_taken += 1;
            }
            if ended {
                return None;
            }
        }
        record_choice(self, p, &action);
        Some(action)
    }
}
