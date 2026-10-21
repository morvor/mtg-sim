//! Metamorphic properties of whole games: relations between two runs that must agree,
//! checked over random games by `tests/properties/` and by `mtg-sim --properties`
//! (see `docs/PROPERTIES.md`).
//!
//! * [`seat_symmetry`]: nothing in the rules depends on a player's seat, only on turn
//!   order and the players' choices (CR 101.4, 102.1, 103.1): two games that seat the same
//!   decks differently (rotated, so turn order is kept), with the same deck going first,
//!   the same library orders and agents choosing by seat-independent keys, are identical
//!   up to renaming players and objects.
//! * [`snapshot_equivalence`]: a copy of a game (`Game::clone`, which the engine uses to
//!   reverse illegal actions) continued with the same agents plays out exactly as the
//!   original does.
//! * [`rollback_identity`]: when the engine reverses an action a player started at
//!   priority but couldn't complete, the game is as it was before (CR 733.1, 733.2).
//! * [`determinism`]: the same seed and the same decisions give the same game. (Replaying
//!   a game played through the agent protocol is checked in `mtg-api`.)
//!
//! Every check returns a [`Summary`] or a description of the first difference.

pub mod agent;
pub mod canon;

pub use agent::{AgentState, PropertyAgent, Recorder};
pub use canon::Canon;

use crate::card::{CardDb, CardDef};
use crate::game::{Game, GameConfig};
use crate::turn::Stage;
use crate::types::PlayerId;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, Mutex, OnceLock};

/// A game to check: the decks by identity (deck 0 takes the first turn) and the seed.
#[derive(Clone)]
pub struct GameSetup {
    pub decks: Vec<Vec<Arc<CardDef>>>,
    pub seed: u64,
    pub max_turns: u32,
    pub max_actions: u64,
    /// See [`AgentState::illegal_rate`].
    pub illegal_rate: f64,
}

impl GameSetup {
    pub fn new(decks: Vec<Vec<Arc<CardDef>>>, seed: u64) -> GameSetup {
        GameSetup {
            decks,
            seed,
            max_turns: 40,
            max_actions: 4000,
            illegal_rate: 0.05,
        }
    }

    pub fn players(&self) -> usize {
        self.decks.len()
    }
}

/// What a checked game did.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub decisions: usize,
    pub turns: u32,
    pub result: String,
    /// Reversed actions checked (for [`rollback_identity`]).
    pub rollbacks: u64,
}

/// One game being played for a check, with its agents' states and recorder.
pub struct Run {
    pub game: Game,
    pub agents: Vec<Arc<Mutex<AgentState>>>,
    pub recorder: Arc<Mutex<Recorder>>,
}

/// What to record while playing.
#[derive(Clone, Copy, Default)]
pub struct Recording {
    pub states: bool,
    pub full_hashes: bool,
    pub full_text_turn: Option<u32>,
    pub check_rollbacks: bool,
}

fn agent_seed(seed: u64, identity: usize) -> u64 {
    seed.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(identity as u64 * 0x1000_0001)
}

/// The game of `setup` with its decks rotated by `rotation` seats: deck `i` sits in seat
/// `(i + rotation) % n`, so turn order is unchanged and deck 0 still goes first.
pub fn build(setup: &GameSetup, rotation: usize, rec: Recording) -> Run {
    let n = setup.players();
    let identity: Vec<u8> = (0..n).map(|s| ((s + n - rotation % n) % n) as u8).collect();
    let config = GameConfig {
        seed: setup.seed,
        starting_player: Some(PlayerId((rotation % n) as u8)),
        max_turns: setup.max_turns,
        max_actions: setup.max_actions,
        player_rng_keys: Some(
            identity
                .iter()
                .map(|i| agent_seed(setup.seed ^ 0x5eed, *i as usize + 1))
                .collect(),
        ),
        ..Default::default()
    };
    let decks: Vec<Vec<Arc<CardDef>>> = identity
        .iter()
        .map(|i| setup.decks[*i as usize].clone())
        .collect();
    let recorder = Arc::new(Mutex::new(Recorder::default()));
    let agents: Vec<Arc<Mutex<AgentState>>> = identity
        .iter()
        .map(|i| {
            let mut s = AgentState::new(agent_seed(setup.seed, *i as usize));
            s.illegal_rate = setup.illegal_rate;
            Arc::new(Mutex::new(s))
        })
        .collect();
    let boxed: Vec<Box<dyn crate::decision::Agent>> = agents
        .iter()
        .map(|s| {
            Box::new(PropertyAgent::new(s.clone(), recorder.clone()))
                as Box<dyn crate::decision::Agent>
        })
        .collect();
    let game = Game::new(config, decks, boxed);
    {
        let mut r = recorder.lock().unwrap();
        r.canon = Canon::new(&game, identity);
        r.states = rec.states;
        r.full_hashes = rec.full_hashes;
        r.full_text_turn = rec.full_text_turn;
        r.check_rollbacks = rec.check_rollbacks;
    }
    Run {
        game,
        agents,
        recorder,
    }
}

impl Run {
    /// Advances the game by one unit (as `Game::run` does). False once it's over.
    pub fn step(&mut self) -> bool {
        let g = &mut self.game;
        if g.result.is_some() {
            return false;
        }
        if g.turn.stage == Stage::PreGame {
            g.start();
        } else {
            g.advance();
        }
        if g.turn.number > g.config.max_turns || g.actions_taken > g.config.max_actions {
            g.draw_game();
        }
        g.result.is_none()
    }

    pub fn finish(&mut self) {
        while self.step() {}
    }

    /// A copy of the game and its agents, recording into a new recorder.
    pub fn copy(&self) -> Run {
        let recorder = Arc::new(Mutex::new(self.recorder.lock().unwrap().for_copy()));
        let agents: Vec<Arc<Mutex<AgentState>>> = self
            .agents
            .iter()
            .map(|s| Arc::new(Mutex::new(s.lock().unwrap().clone())))
            .collect();
        let mut game = self.game.clone();
        game.set_agents(
            agents
                .iter()
                .map(|s| {
                    Box::new(PropertyAgent::new(s.clone(), recorder.clone()))
                        as Box<dyn crate::decision::Agent>
                })
                .collect(),
        );
        Run {
            game,
            agents,
            recorder,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.recorder.lock().unwrap().lines.clone()
    }

    fn summary(&self) -> Summary {
        let r = self.recorder.lock().unwrap();
        Summary {
            decisions: r.lines.len(),
            turns: self.game.turn.number,
            result: r.canon.result(&self.game),
            rollbacks: r.rollbacks_checked,
        }
    }

    /// The final line: the canonical state and the result.
    fn final_line(&self) -> String {
        let r = self.recorder.lock().unwrap();
        // Characteristics (and what changes with them, such as summoning sickness after
        // a change of control) are computed as they're needed; the game may have ended
        // before they were.
        let mut g = self.game.clone();
        g.set_agents(vec![]);
        if g.dirty {
            g.recompute();
        }
        format!("end {} {}", r.canon.result(&g), r.canon.state(&g))
    }
}

/// The first difference between two transcripts, with the line before it.
pub fn compare(what: &str, a: &[String], b: &[String]) -> Result<(), String> {
    let n = a.len().min(b.len());
    let i = match (0..n).find(|i| a[*i] != b[*i]) {
        Some(i) => i,
        None if a.len() == b.len() => return Ok(()),
        None => n,
    };
    // MTG_PROPERTY_DUMP=DIR writes both transcripts there, to look at a difference.
    if let Ok(dir) = std::env::var("MTG_PROPERTY_DUMP") {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(format!("{dir}/first.txt"), a.join("\n"));
        let _ = std::fs::write(format!("{dir}/second.txt"), b.join("\n"));
    }
    let cut = |s: Option<&String>| -> String {
        match s {
            None => "(nothing)".into(),
            Some(s) if s.len() > 1500 => {
                let mut k = 1500;
                while !s.is_char_boundary(k) {
                    k -= 1;
                }
                format!("{}…", &s[..k])
            }
            Some(s) => s.clone(),
        }
    };
    let detail = match (a.get(i), b.get(i)) {
        (Some(x), Some(y)) if x.len() > 1500 || y.len() > 1500 => {
            agent::first_difference_text(x, y)
        }
        (x, y) => format!("\n    first:  {}\n    second: {}", cut(x), cut(y)),
    };
    Err(format!(
        "{what}: the games differ at line {i} of {}/{} (after: {}){detail}",
        a.len(),
        b.len(),
        cut(i.checked_sub(1).and_then(|k| a.get(k))),
    ))
}

/// Seat symmetry: the game of `setup` and the same game with the decks rotated by
/// `rotation` seats are identical up to renaming players and objects.
pub fn seat_symmetry(setup: &GameSetup, rotation: usize) -> Result<Summary, String> {
    let rec = Recording {
        states: true,
        ..Default::default()
    };
    let mut a = build(setup, 0, rec);
    a.finish();
    let mut b = build(setup, rotation, rec);
    b.finish();
    let (mut la, mut lb) = (a.lines(), b.lines());
    la.push(a.final_line());
    lb.push(b.final_line());
    compare(&format!("seat symmetry (rotation {rotation})"), &la, &lb)?;
    Ok(a.summary())
}

/// Snapshot equivalence: after `at` units of play, a copy of the game continued with
/// copies of the agents plays out as the original does.
pub fn snapshot_equivalence(setup: &GameSetup, at: u64) -> Result<Summary, String> {
    let play = |text_turn: Option<u32>| {
        let rec = Recording {
            states: true,
            full_hashes: true,
            full_text_turn: text_turn,
            ..Default::default()
        };
        let mut a = build(setup, 0, rec);
        let mut units = 0;
        while units < at && a.step() {
            units += 1;
        }
        let mut b = a.copy();
        let from = a.recorder.lock().unwrap().lines.len();
        a.finish();
        b.finish();
        let mut la = a.lines().split_off(from);
        let mut lb = b.lines();
        la.push(agent::full_text(&a.game));
        lb.push(agent::full_text(&b.game));
        (a.summary(), la, lb, units)
    };
    let (summary, la, lb, units) = play(None);
    compare_hashed(&format!("snapshot after {units} units"), &la, &lb, |t| {
        let (_, la, lb, _) = play(Some(t));
        (la, lb)
    })?;
    Ok(summary)
}

/// Rollback identity: every action the engine reversed left the game as it was before
/// the action began (CR 733.1). The agents attempt actions that aren't legal at
/// `illegal_rate` (at least 0.2).
pub fn rollback_identity(setup: &GameSetup) -> Result<Summary, String> {
    let mut s = setup.clone();
    s.illegal_rate = s.illegal_rate.max(0.2);
    let mut a = build(
        &s,
        0,
        Recording {
            check_rollbacks: true,
            ..Default::default()
        },
    );
    a.finish();
    let r = a.recorder.lock().unwrap();
    if let Some(v) = r.violations.first() {
        return Err(format!(
            "rollback identity: {} violation(s); first: {v}",
            r.violations.len()
        ));
    }
    drop(r);
    Ok(a.summary())
}

/// Determinism: playing the same game twice gives the same decisions and the same game.
pub fn determinism(setup: &GameSetup) -> Result<Summary, String> {
    let play = |text_turn: Option<u32>| {
        let rec = Recording {
            states: true,
            full_hashes: true,
            full_text_turn: text_turn,
            ..Default::default()
        };
        let mut a = build(setup, 0, rec);
        a.finish();
        let mut l = a.lines();
        l.push(agent::full_text(&a.game));
        (a.summary(), l)
    };
    let (summary, la) = play(None);
    let (_, lb) = play(None);
    compare_hashed("determinism", &la, &lb, |t| {
        (play(Some(t)).1, play(Some(t)).1)
    })?;
    Ok(summary)
}

/// [`compare`], and when the first difference is in the hash of the whole game at the
/// beginning of a turn, the difference in the games' debug output (from `replay`, which
/// plays both games again recording that turn's text).
fn compare_hashed(
    what: &str,
    a: &[String],
    b: &[String],
    replay: impl Fn(u32) -> (Vec<String>, Vec<String>),
) -> Result<(), String> {
    let Err(e) = compare(what, a, b) else {
        return Ok(());
    };
    let i = (0..a.len().min(b.len())).find(|i| a[*i] != b[*i]);
    let turn = i
        .and_then(|i| a[i].strip_prefix("hash T"))
        .and_then(|r| r.split(' ').next())
        .and_then(|t| t.parse::<u32>().ok());
    let Some(t) = turn else {
        return Err(e);
    };
    let (ra, rb) = replay(t);
    let key = format!("hash T{t} ");
    match (
        ra.iter().find(|l| l.starts_with(&key)),
        rb.iter().find(|l| l.starts_with(&key)),
    ) {
        (Some(x), Some(y)) => Err(format!(
            "{e}\n    the games at the beginning of turn {t}: {}",
            agent::first_difference_text(x, y)
        )),
        _ => Err(e),
    }
}

/// Nonland cards for random decks: fully supported paper cards legal somewhere, with mana
/// value at most 6, and their color identity as a WUBRG mask.
fn pool() -> &'static [(String, u8)] {
    static POOL: OnceLock<Vec<(String, u8)>> = OnceLock::new();
    POOL.get_or_init(|| {
        let mut v = Vec::new();
        for c in mtg_data::cards().iter() {
            if !c.is_playable_card() || !c.is_legal_somewhere() {
                continue;
            }
            if !c.games.is_empty() && c.games.iter().all(|g| g != "paper") {
                continue;
            }
            let front = c.type_line.as_deref().unwrap_or("");
            let front = front.split(" // ").next().unwrap_or(front);
            if front.is_empty() || front.contains("Land") || c.cmc.unwrap_or(0.0) > 6.0 {
                continue;
            }
            let mask = c.color_identity.iter().fold(0u8, |m, s| {
                m | match s.as_str() {
                    "W" => 1,
                    "U" => 2,
                    "B" => 4,
                    "R" => 8,
                    "G" => 16,
                    _ => 0,
                }
            });
            v.push((c.name.clone(), mask));
        }
        v
    })
}

/// A random 40-card deck: one or two colors, 17 basic lands, and 23 cards of 10 distinct
/// fully supported spells in those colors (or colorless).
pub fn random_deck(seed: u64) -> Vec<Arc<CardDef>> {
    const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut colors: Vec<usize> = (0..5).collect();
    colors.shuffle(&mut rng);
    colors.truncate(rng.gen_range(1..=2));
    let mask = colors.iter().fold(0u8, |m, &c| m | 1 << c);
    let candidates: Vec<&str> = pool()
        .iter()
        .filter(|(_, ci)| ci & !mask == 0)
        .map(|(n, _)| n.as_str())
        .collect();
    let db = CardDb::global();
    let mut deck: Vec<Arc<CardDef>> = (0..17)
        .map(|i| {
            db.get(BASICS[colors[i % colors.len()]])
                .expect("basic land")
        })
        .collect();
    let mut spells: Vec<Arc<CardDef>> = Vec::new();
    for _ in 0..3000 {
        if spells.len() == 10 || candidates.is_empty() {
            break;
        }
        let name = candidates[rng.gen_range(0..candidates.len())];
        if spells.iter().any(|d| d.name.eq_ignore_ascii_case(name)) {
            continue;
        }
        if let Some(d) = db.get(name).filter(|d| d.is_fully_supported()) {
            spells.push(d);
        }
    }
    for i in 0..23 {
        if let Some(s) = spells.get(i % spells.len().max(1)) {
            deck.push(s.clone());
        }
    }
    deck
}

/// A deck from "4 Lightning Bolt" style lines, separated by commas or newlines.
pub fn deck(list: &str) -> Vec<Arc<CardDef>> {
    let db = CardDb::global();
    let mut v = Vec::new();
    for part in list.split([',', '\n']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (n, name) = match part.split_once(' ') {
            Some((n, rest)) if n.parse::<usize>().is_ok() => (n.parse().unwrap(), rest.trim()),
            _ => (1, part),
        };
        let c = db
            .get(name)
            .unwrap_or_else(|| panic!("unknown card {name}"));
        for _ in 0..n {
            v.push(c.clone());
        }
    }
    v
}
