//! [`PropertyAgent`]: a random agent whose choices don't depend on seats or object ids,
//! which records what it was asked and answered in a shared [`Recorder`].

use super::canon::Canon;
use crate::decision::*;
use crate::game::Game;
use crate::object::CastMethod;
use crate::turn::Step;
use crate::types::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

/// What the agents of one game record and check.
#[derive(Default)]
pub struct Recorder {
    pub canon: Canon,
    /// One line per decision (who decided, the decision and the answer), preceded at each
    /// priority decision by the canonical state (if `states`) and at the first decision of
    /// each turn by a hash of the whole game (if `full_hashes`).
    pub lines: Vec<String>,
    pub states: bool,
    pub full_hashes: bool,
    /// Record the whole game's debug output instead of its hash at this turn (to show
    /// where two games differ).
    pub full_text_turn: Option<u32>,
    last_turn: u32,
    /// Check that an action the engine reversed left the game as it was (CR 733.1).
    pub check_rollbacks: bool,
    /// The game as a player started an action at priority: the player, the game, and its
    /// count of reversed attempts.
    pending: Option<(PlayerId, Box<Game>, u32)>,
    /// Actions started at priority so far; the game is kept for the first 30 and every
    /// fifth after that (each check copies and prints the whole game).
    actions_started: u64,
    /// The number of objects the game had at the latest decision (to notice a restart).
    objects_seen: usize,
    pub rollbacks_checked: u64,
    /// Of those, the reversals that kept library actions (CR 733.1), which are checked
    /// for what they must undo only (see [`undone_difference`]).
    pub rollbacks_keeping_library_actions: u64,
    pub violations: Vec<String>,
}

impl Recorder {
    pub fn new(canon: Canon) -> Recorder {
        Recorder {
            canon,
            ..Default::default()
        }
    }

    /// A recorder for a copy of the game this one records: the same settings and
    /// position, no lines.
    pub fn for_copy(&self) -> Recorder {
        Recorder {
            canon: self.canon.clone(),
            states: self.states,
            full_hashes: self.full_hashes,
            full_text_turn: self.full_text_turn,
            last_turn: self.last_turn,
            objects_seen: self.objects_seen,
            check_rollbacks: self.check_rollbacks,
            ..Default::default()
        }
    }

    fn before_decision(&mut self, g: &Game, p: PlayerId, d: &Decision) {
        if self.full_hashes && g.turn.number != self.last_turn {
            self.last_turn = g.turn.number;
            if self.full_text_turn == Some(g.turn.number) {
                self.lines
                    .push(format!("hash T{} {}", g.turn.number, full_text(g)));
            } else {
                self.lines
                    .push(format!("hash T{} {:016x}", g.turn.number, full_hash(g)));
            }
        }
        if let Decision::Priority { .. } = d {
            if self.states {
                self.lines.push(self.canon.state(g));
            }
            if let Some((q, before, attempts)) = self.pending.take() {
                if g.turn.illegal_attempts == attempts + 1 && q != p {
                    self.violations.push(format!(
                        "turn {}: an action of player {} was reversed but player {} got \
                         priority (CR 733.2)",
                        g.turn.number, q.0, p.0
                    ));
                } else if g.turn.illegal_attempts == attempts + 1 {
                    self.rollbacks_checked += 1;
                    if g.reversals.kept > before.reversals.kept {
                        // CR 733.1: the library actions stay, and what they cause (a
                        // triggered ability, a token ceasing to exist) follows.
                        self.rollbacks_keeping_library_actions += 1;
                        if let Some(d) = undone_difference(&before, g) {
                            self.violations.push(format!(
                                "turn {}: an action of player {} was reversed keeping its                                  library actions, but {d} (CR 733.1)",
                                g.turn.number, p.0
                            ));
                        }
                        return;
                    }
                    let (a, b) = (rollback_view(&before), rollback_view(g));
                    if a != b {
                        self.violations.push(format!(
                            "turn {}: an action of player {} was reversed but the game \
                             changed (CR 733.1): {}",
                            g.turn.number,
                            p.0,
                            first_difference_text(&a, &b)
                        ));
                    }
                }
            }
        }
    }

    fn after_decision(&mut self, g: &Game, p: PlayerId, d: &Decision, a: &Answer) {
        let line = format!(
            "P{} T{} {:?}: {} => {}",
            self.canon.player(p),
            g.turn.number,
            g.turn.step,
            self.canon.decision(g, d),
            self.canon.answer(g, d, a)
        );
        self.lines.push(line);
        if self.check_rollbacks {
            if let Answer::Action(x) = a {
                if !matches!(x, Action::Pass | Action::Concede) {
                    self.actions_started += 1;
                    let n = self.actions_started;
                    if n <= 30 || n % 5 == 0 {
                        self.pending = Some((p, Box::new(g.clone()), g.turn.illegal_attempts));
                    }
                }
            }
        }
    }
}

/// A hash of the whole game's debug output.
pub fn full_hash(g: &Game) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    full_text(g).hash(&mut h);
    h.finish()
}

/// The whole game's debug output, with ability ids renamed by first appearance (see
/// [`rename_uids`]).
pub fn full_text(g: &Game) -> String {
    rename_uids(&format!("{g:?}"))
}

/// Debug output with ability ids renamed in order of first appearance. Ability ids come
/// from a counter shared by the whole process, and many abilities are created as a game
/// goes (a delayed trigger as it triggers, a Clue's ability as it's investigated), so two
/// runs of the same game in one process number them differently; they're compared up to
/// renaming, like objects. Renamed: `uid: N` and `ability: N` fields, and map keys that
/// are ability ids (`activations_this_turn: {N: 1}`). Also renamed: `key: N` fields, which
/// identify a replacement effect by a mix of its object and ability ids, links of
/// abilities effects give (`link: N` from 0x8000 on) and mana riders' ids.
pub fn rename_uids(s: &str) -> String {
    use regex::Regex;
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static FIELD: OnceLock<Regex> = OnceLock::new();
    static KEY: OnceLock<Regex> = OnceLock::new();
    // `uid: N`, `ability: N`, `ability_uid: N`, and `(.., ObjectId(n), N)` (abilities of
    // objects: `activated`, `state_triggers_active`, `power_up_activations`).
    let field = FIELD.get_or_init(|| {
        Regex::new(r"\b(uid|ability|ability_uid|key): (\d+)|(ObjectId\(\d+\)), (\d+)\)").unwrap()
    });
    let key = KEY.get_or_init(|| {
        Regex::new(r"\b(activations_this_turn|activations|triggers_this_turn): \{([^}]*)\}")
            .unwrap()
    });
    let mut names: HashMap<String, usize> = HashMap::new();
    let num = |c: &regex::Captures| c.get(2).or(c.get(4)).unwrap().as_str().to_string();
    for c in field.captures_iter(s) {
        let n = names.len();
        names.entry(num(&c)).or_insert(n);
    }
    let s = field.replace_all(s, |c: &regex::Captures| {
        let k = names[&num(c)];
        match c.get(1) {
            Some(f) => format!("{}: u{k}", f.as_str()),
            None => format!("{}, u{k})", &c[3]),
        }
    });
    // The maps keyed by ability ids (`activations_this_turn`, `activations`,
    // `triggers_this_turn`), whose keys may carry flags in the high bits (see
    // `triggers::turn_keys`): the ability id is renamed, the flags kept.
    let mut names = names;
    let mut rename_key = |n: &str| -> String {
        let Ok(n) = n.parse::<u64>() else {
            return n.to_string();
        };
        let flags = n & (3 << 62);
        let base = (n & !(3 << 62)).to_string();
        let len = names.len();
        let k = *names.entry(base).or_insert(len);
        if flags != 0 {
            format!("u{k}+{flags:x}")
        } else {
            format!("u{k}")
        }
    };
    let s = key.replace_all(&s, |c: &regex::Captures| {
        let entries: Vec<String> = c[2]
            .split(", ")
            .filter(|e| !e.is_empty())
            .map(|e| match e.split_once(": ") {
                Some((k, v)) => format!("{}: {v}", rename_key(k)),
                None => e.to_string(),
            })
            .collect();
        format!("{}: {{{}}}", &c[1], entries.join(", "))
    });
    // Links of abilities effects give (CR 607.5; numbered per game now, see
    // `ability_grants::FreshLinks`, but still compared up to renaming), and mana riders'
    // ids.
    static OTHER: OnceLock<Regex> = OnceLock::new();
    let other = OTHER.get_or_init(|| Regex::new(r"(ManaRider \{ id|\blink): (\d+)").unwrap());
    let mut fresh: HashMap<String, usize> = HashMap::new();
    other
        .replace_all(&s, |c: &regex::Captures| {
            let dynamic = &c[1] != "link" || c[2].parse::<u64>().is_ok_and(|n| n >= 0x8000);
            if !dynamic {
                return c[0].to_string();
            }
            let n = fresh.len();
            let k = *fresh.entry(c[2].to_string()).or_insert(n);
            format!("{}: f{k}", &c[1])
        })
        .into_owned()
}

/// The game for comparing it before an action and after the action was reversed: its
/// debug output without the simulation's bookkeeping (decision and check counters, the
/// count of reversed attempts, the log, the events recorded for observers, whose positions
/// don't go back) and without the loop watch, which records the choices players made at
/// priority, including attempts (CR 732.3, 104.4b).
pub fn rollback_view(g: &Game) -> String {
    let mut c = g.clone();
    c.event_feed = Default::default();
    c.reversals = Default::default();
    c.actions_taken = 0;
    c.sba_checks = 0;
    c.turn.illegal_attempts = 0;
    c.log.clear();
    c.end.forced_asks = 0;
    c.end.loop_mark = 0;
    c.end.loop_len = 0;
    c.end.loop_states.clear();
    c.end.loop_controllers.clear();
    c.shortcuts = Default::default();
    format!("{c:?}")
}

/// For a reversal that kept library actions (CR 733.1), which change libraries, hands,
/// graveyards, exile and the permanents put into a library, and may trigger abilities:
/// what differs among what the reversal must still undo. Each player's life total,
/// counters and mana pool; each permanent that's still on the battlefield, as it was
/// (tapped, counters, damage, controller); and the stack, which may only have triggered
/// abilities added on top.
pub fn undone_difference(before: &Game, after: &Game) -> Option<String> {
    for (a, b) in before.players.iter().zip(&after.players) {
        if a.life != b.life {
            return Some(format!(
                "{}'s life went from {} to {}",
                a.id, a.life, b.life
            ));
        }
        if a.counters != b.counters {
            return Some(format!("{}'s counters changed", a.id));
        }
        if format!("{:?}", a.mana_pool) != format!("{:?}", b.mana_pool) {
            return Some(format!("{}'s mana pool changed", a.id));
        }
    }
    for id in &before.battlefield {
        if !after.is_in_zone(*id, crate::object::Zone::Battlefield) {
            continue;
        }
        let (x, y) = (before.obj(*id), after.obj(*id));
        if x.tapped != y.tapped
            || x.counters != y.counters
            || x.damage != y.damage
            || x.controller != y.controller
        {
            return Some(format!("{} changed on the battlefield", y.chars.name));
        }
    }
    if after.stack.get(..before.stack.len()) != Some(&before.stack[..]) {
        return Some("the stack changed".into());
    }
    None
}

/// Where two long strings first differ, with some context.
pub fn first_difference_text(a: &str, b: &str) -> String {
    let i = a
        .bytes()
        .zip(b.bytes())
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()));
    // MTG_PROPERTY_CONTEXT sets how much text is shown around the difference.
    let ctx: usize = std::env::var("MTG_PROPERTY_CONTEXT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(150);
    let from = i.saturating_sub(ctx);
    let snip = |s: &str| -> String {
        let lo = (from..=i.min(s.len()))
            .find(|k| s.is_char_boundary(*k))
            .unwrap_or(0);
        let hi = ((i + ctx).min(s.len())..=s.len())
            .find(|k| s.is_char_boundary(*k))
            .unwrap_or(s.len());
        s[lo..hi].to_string()
    };
    format!("before: …{}… after: …{}…", snip(a), snip(b))
}

/// How a [`PropertyAgent`] plays.
#[derive(Clone)]
pub struct AgentState {
    rng: ChaCha8Rng,
    /// The probability of attempting an action that isn't among the legal ones (casting a
    /// card that can't be cast or paid for, activating an ability that can't be
    /// activated, playing a land that can't be played).
    pub illegal_rate: f64,
    /// The probability of taking a mulligan with a first hand.
    pub mulligan_rate: f64,
    /// The turn and step of the latest priority decision and the actions taken in it (see
    /// [`crate::agents::RandomAgent`]).
    acted: (u32, Step, Vec<String>),
}

impl AgentState {
    pub fn new(seed: u64) -> AgentState {
        AgentState {
            rng: ChaCha8Rng::seed_from_u64(seed),
            illegal_rate: 0.0,
            mulligan_rate: 0.1,
            acted: (0, Step::Untap, vec![]),
        }
    }
}

/// A random agent choosing by canonical keys (see [`Canon`]), so that it makes the same
/// choices in two games that differ only in seats; it records into a [`Recorder`].
pub struct PropertyAgent {
    pub state: Arc<Mutex<AgentState>>,
    pub recorder: Arc<Mutex<Recorder>>,
}

impl PropertyAgent {
    pub fn new(state: Arc<Mutex<AgentState>>, recorder: Arc<Mutex<Recorder>>) -> Self {
        PropertyAgent { state, recorder }
    }
}

impl Agent for PropertyAgent {
    fn name(&self) -> &str {
        "property"
    }

    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        let mut rec = self.recorder.lock().unwrap_or_else(|e| e.into_inner());
        // A restarted game (CR 727) has objects of its own: fewer than the game had.
        if g.subgames.depth == 0 {
            if g.objects.len() < rec.objects_seen {
                rec.canon = rec.canon.for_new_game(g);
            }
            rec.objects_seen = g.objects.len();
        }
        // A subgame has objects of its own (CR 729).
        let main = rec
            .canon
            .for_subgame(g)
            .map(|sub| std::mem::replace(&mut rec.canon, sub));
        rec.before_decision(g, p, d);
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let a = st.choose(g, &rec.canon, p, d);
        drop(st);
        rec.after_decision(g, p, d, &a);
        if let Some(main) = main {
            rec.canon = main;
        }
        a
    }
}

/// Indices of `v` sorted by a key.
fn sorted_by<T, K: Ord>(v: &[T], key: impl Fn(&T) -> K) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by_key(|i| key(&v[*i]));
    idx
}

impl AgentState {
    fn pick_entities(
        &mut self,
        canon: &Canon,
        candidates: &[Entity],
        min: u32,
        max: u32,
    ) -> Vec<Entity> {
        let mut c: Vec<Entity> = sorted_by(candidates, |e| canon.entity(e))
            .into_iter()
            .map(|i| candidates[i])
            .collect();
        c.shuffle(&mut self.rng);
        let max = max.min(c.len() as u32).max(min);
        let n = if max > min {
            self.rng.gen_range(min..=max)
        } else {
            min
        };
        c.into_iter().take(n as usize).collect()
    }

    fn choose(&mut self, g: &Game, canon: &Canon, p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { actions } => self.priority(g, canon, p, actions),
            Decision::Mulligan { mulligans_taken } => {
                Answer::Bool(*mulligans_taken == 0 && self.rng.gen_bool(self.mulligan_rate))
            }
            Decision::PutOnBottom { cards, n } => {
                let e: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
                Answer::Entities(self.pick_entities(canon, &e, *n, *n))
            }
            Decision::ChooseModes {
                modes,
                min,
                max,
                available,
                ..
            } => {
                let mut idx: Vec<usize> = if available.is_empty() {
                    (0..modes.len()).collect()
                } else {
                    available.clone()
                };
                idx.shuffle(&mut self.rng);
                let lo = (*min).max(1).min(idx.len() as u32);
                let hi = (*max).min(idx.len() as u32).max(lo);
                let n = self.rng.gen_range(lo..=hi);
                Answer::Indices(idx.into_iter().take(n as usize).collect())
            }
            Decision::ChooseX { min, max, .. } => {
                let lo = (*min).max(0);
                let hi = (*max).min(lo + 12).max(lo);
                Answer::Number(self.rng.gen_range(lo..=hi))
            }
            Decision::ChooseCastingMethod { options, .. } => {
                Answer::Index(self.rng.gen_range(0..options.len().max(1)))
            }
            Decision::OptionalCost { repeatable, .. } => {
                if *repeatable {
                    Answer::Number(self.rng.gen_range(0..=2))
                } else {
                    Answer::Bool(self.rng.gen_bool(0.5))
                }
            }
            Decision::ChooseTargets {
                candidates,
                min,
                max,
                ..
            }
            | Decision::ChooseEntities {
                candidates,
                min,
                max,
                ..
            } => Answer::Entities(self.pick_entities(canon, candidates, *min, *max)),
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => {
                let order = sorted_by(recipients, |e| canon.entity(e));
                let mut amounts = vec![*min_each as i64; recipients.len()];
                let mut left = *total as i64 - (*min_each as i64 * recipients.len() as i64);
                while left > 0 && !order.is_empty() {
                    let k = order[self.rng.gen_range(0..order.len())];
                    amounts[k] += 1;
                    left -= 1;
                }
                Answer::Numbers(amounts)
            }
            Decision::YesNo { .. } => Answer::Bool(self.rng.gen_bool(0.5)),
            Decision::Order { items, .. } => {
                let mut idx = sorted_by(items, |s| canon.text(s));
                idx.shuffle(&mut self.rng);
                Answer::Indices(idx)
            }
            Decision::ChooseOption { options, .. } => {
                Answer::Index(self.rng.gen_range(0..options.len().max(1)))
            }
            Decision::ChooseNumber { min, max, .. } => {
                let hi = (*max).min(*min + 10).max(*min);
                Answer::Number(self.rng.gen_range(*min..=hi))
            }
            Decision::DeclareAttackers { options } => {
                let mut v = Vec::new();
                for i in sorted_by(options, |(a, _)| canon.obj(*a)) {
                    let (a, ts) = &options[i];
                    if self.rng.gen_bool(0.5) && !ts.is_empty() {
                        let order = sorted_by(ts, |e| canon.entity(e));
                        v.push((*a, ts[order[self.rng.gen_range(0..order.len())]]));
                    }
                }
                Answer::Attackers(v)
            }
            Decision::DeclareBlockers { options } => {
                let mut v = Vec::new();
                for i in sorted_by(options, |(b, _)| canon.obj(*b)) {
                    let (b, ts) = &options[i];
                    if self.rng.gen_bool(0.4) && !ts.is_empty() {
                        let order = sorted_by(ts, |o| canon.obj(*o));
                        v.push((*b, ts[order[self.rng.gen_range(0..order.len())]]));
                    }
                }
                Answer::Blockers(v)
            }
            Decision::Scry { cards } | Decision::Surveil { cards } => {
                let (mut top, mut bottom) = (vec![], vec![]);
                for c in cards {
                    if self.rng.gen_bool(0.5) {
                        top.push(*c);
                    } else {
                        bottom.push(*c);
                    }
                }
                Answer::Split(top, bottom)
            }
            Decision::ChooseReplacement { options } => {
                Answer::Index(self.rng.gen_range(0..options.len().max(1)))
            }
            Decision::NameCard { .. } | Decision::AssignCombatDamage { .. } => Answer::Default,
        }
    }

    fn priority(&mut self, g: &Game, canon: &Canon, p: PlayerId, actions: &[Action]) -> Answer {
        let mut legal: Vec<(String, &Action)> = actions
            .iter()
            .filter(|a| !matches!(a, Action::Pass | Action::Concede))
            .map(|a| (canon.action(g, a), a))
            .collect();
        legal.sort_by(|x, y| x.0.cmp(&y.0));
        if self.illegal_rate > 0.0 && self.rng.gen_bool(self.illegal_rate) {
            let mut other: Vec<(String, Action)> = conceivable_actions(g, p)
                .into_iter()
                .map(|a| (canon.action(g, &a), a))
                .filter(|(k, _)| !legal.iter().any(|(l, _)| l == k))
                .collect();
            other.sort_by(|x, y| x.0.cmp(&y.0));
            other.dedup_by(|x, y| x.0 == y.0);
            if !other.is_empty() {
                let i = self.rng.gen_range(0..other.len());
                return Answer::Action(other.swap_remove(i).1);
            }
        }
        if (self.acted.0, self.acted.1) != (g.turn.number, g.turn.step) {
            self.acted = (g.turn.number, g.turn.step, Vec::new());
        }
        let taken = self.acted.2.len().min(1000) as i32;
        let pass = 0.2f64.max(1.0 - 0.85f64.powi(taken));
        if legal.is_empty() || self.rng.gen_bool(pass) {
            return Answer::Action(Action::Pass);
        }
        let fresh: Vec<usize> = (0..legal.len())
            .filter(|i| !self.acted.2.contains(&legal[*i].0))
            .collect();
        let pick = if let Some(i) = legal
            .iter()
            .position(|(_, a)| matches!(a, Action::PlayLand { .. }))
        {
            i
        } else if !fresh.is_empty() && !self.rng.gen_bool(0.1) {
            fresh[self.rng.gen_range(0..fresh.len())]
        } else if self.rng.gen_bool(0.1) {
            self.rng.gen_range(0..legal.len())
        } else {
            return Answer::Action(Action::Pass);
        };
        self.acted.2.push(legal[pick].0.clone());
        Answer::Action(legal[pick].1.clone())
    }
}

/// Actions a player could try at priority, legal or not: casting any card in a hand,
/// activating any activated ability of a permanent, playing any land card in their hand.
fn conceivable_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    let mut v = Vec::new();
    for q in g.player_ids() {
        for c in &g.player(q).hand {
            v.push(Action::Cast {
                card: *c,
                method: CastMethod::Normal,
            });
            if q == p && g.obj(*c).chars.is_land() {
                v.push(Action::PlayLand { card: *c });
            }
        }
    }
    for id in &g.battlefield {
        for a in &g.obj(*id).chars.abilities {
            if matches!(a.kind, crate::ability::AbilityKind::Activated(_)) {
                v.push(Action::Activate {
                    source: *id,
                    ability: a.uid,
                });
            }
        }
    }
    v
}
