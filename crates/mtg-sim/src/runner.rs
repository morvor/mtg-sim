//! Playing one game on its own thread, with a watchdog, panic catching and the rules
//! checks.

use crate::checks::{self, CheckingAgent, Ledger, Violation};
use crate::coverage::{Focus, FocusAgent, Usage};
use crate::decks::DeckList;
use mtg_api::{spawn_external, ChildTransport, ProtocolOptions};
use mtg_engine::agents::RandomAgent;
use mtg_engine::decision::PassiveAgent;
use mtg_engine::events::Event;
use mtg_engine::turn::Stage;
use mtg_engine::*;
use smol_str::SmolStr;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The message and location of the latest panic on each thread, recorded by the panic
/// hook (see [`install_panic_hook`]).
static LAST_PANIC: Mutex<Vec<(std::thread::ThreadId, String)>> = Mutex::new(Vec::new());

/// Records panics (instead of printing them) so a game's panic can be reported with it.
pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let mut v = LAST_PANIC.lock().unwrap_or_else(|e| e.into_inner());
        let me = std::thread::current().id();
        v.retain(|(t, _)| *t != me);
        v.push((me, info.to_string()));
    }));
}

fn take_panic() -> Option<String> {
    let mut v = LAST_PANIC.lock().unwrap_or_else(|e| e.into_inner());
    let me = std::thread::current().id();
    let i = v.iter().position(|(t, _)| *t == me)?;
    Some(v.remove(i).1)
}

pub enum Outcome {
    Finished {
        result: GameResult,
        turns: u32,
        log: Vec<String>,
    },
    /// Stopped after the `--slow` limit.
    Slow { turn: u32, log: Vec<String> },
    Panicked {
        message: String,
        turn: u32,
        log: Vec<String>,
    },
}

/// Who plays a seat.
#[derive(Clone, Debug)]
pub enum AgentSpec {
    Random,
    Passive,
    /// An external program speaking the JSON protocol.
    Command(String),
}

impl AgentSpec {
    pub fn parse(s: &str) -> AgentSpec {
        match s {
            "random" => AgentSpec::Random,
            "passive" => AgentSpec::Passive,
            _ => match s.strip_prefix("cmd:") {
                Some(c) if !c.trim().is_empty() => AgentSpec::Command(c.to_string()),
                _ => panic!("unknown agent {s:?} (random, passive or cmd:COMMAND)"),
            },
        }
    }
}

/// How the external agents are run.
#[derive(Clone, Default)]
pub struct ExternalSettings {
    pub transcript: Option<String>,
    pub auto_pass: bool,
}

/// Everything needed to play one game.
pub struct GameSpec {
    pub config: GameConfig,
    pub decks: Vec<DeckList>,
    /// Who plays each seat (random players for seats not listed).
    pub specs: Vec<AgentSpec>,
    pub external: ExternalSettings,
    pub agent_seed: u64,
    pub logging: bool,
    /// Check every `check`th priority decision (0: no checks).
    pub check: u32,
    /// The players' commanders (CR 903.3), by name.
    pub commanders: Vec<Vec<SmolStr>>,
    /// Agents prefer using this card, and record what they were offered.
    pub focus: Option<Arc<Focus>>,
    /// Records card and ability use.
    pub usage: Option<Arc<Mutex<Usage>>>,
}

/// `a`, checked every `check`th priority decision (unless `check` is 0).
fn checked<A: Agent + 'static>(
    a: A,
    check: u32,
    violations: &Arc<Mutex<Vec<Violation>>>,
    ledger: &Arc<Mutex<Ledger>>,
) -> Box<dyn Agent> {
    if check > 0 {
        Box::new(CheckingAgent::new(a, check, violations.clone()).with_ledger(ledger.clone()))
    } else {
        Box::new(a)
    }
}

/// Plays one game on its own thread (with a large stack), catching a panic. Returns
/// `None` if no decision was made for `timeout` (the game thread is left running).
pub fn play(
    spec: GameSpec,
    slow: Duration,
    timeout: Duration,
) -> Option<(Outcome, Vec<Violation>)> {
    let (tx, rx) = mpsc::channel();
    let violations: Arc<Mutex<Vec<Violation>>> = Arc::default();
    let violations2 = violations.clone();
    let progress = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let (progress2, stop2) = (progress.clone(), stop.clone());
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let GameSpec {
                config,
                decks,
                specs,
                external,
                agent_seed,
                logging,
                check,
                commanders,
                focus,
                usage,
            } = spec;
            let ledger: Arc<Mutex<Ledger>> = Arc::default();
            let mut game: Option<Game> = None;
            let mut connections: Vec<(PlayerId, ChildTransport)> = Vec::new();
            let r = panic::catch_unwind(AssertUnwindSafe(|| {
                let agents: Vec<Box<dyn Agent>> = (0..decks.len())
                    .map(|p| {
                        let seat = PlayerId(p as u8);
                        let seed = agent_seed.wrapping_add(p as u64);
                        match specs.get(p).cloned().unwrap_or(AgentSpec::Random) {
                            AgentSpec::Random => match (&focus, &usage) {
                                (Some(f), Some(u)) => checked(
                                    FocusAgent::new(seed, f.clone(), u.clone()),
                                    check,
                                    &violations2,
                                    &ledger,
                                ),
                                _ => checked(RandomAgent::new(seed), check, &violations2, &ledger),
                            },
                            AgentSpec::Passive => Box::new(PassiveAgent) as Box<dyn Agent>,
                            AgentSpec::Command(cmd) => {
                                let options = ProtocolOptions {
                                    auto_pass: external.auto_pass,
                                    ..Default::default()
                                };
                                let (agent, conn) = spawn_external(&cmd, seat, options)
                                    .unwrap_or_else(|e| panic!("can't start {cmd:?}: {e}"));
                                if let Some(path) = &external.transcript {
                                    let file = std::fs::OpenOptions::new()
                                        .create(true)
                                        .append(true)
                                        .open(path)
                                        .unwrap_or_else(|e| panic!("can't open {path}: {e}"));
                                    conn.transcript(file);
                                }
                                connections.push((seat, conn));
                                Box::new(agent)
                            }
                        }
                    })
                    .collect();
                let g = game.insert(Game::new(
                    config,
                    decks.iter().map(|d| d.main.clone()).collect(),
                    agents,
                ));
                if !connections.is_empty() {
                    mtg_api::prepare_game(g);
                }
                // Sideboards stay outside the game (a companion may be revealed from them).
                for (i, d) in decks.iter().enumerate() {
                    if !d.sideboard.is_empty() {
                        g.add_to_sideboard(PlayerId(i as u8), d.sideboard.clone());
                    }
                }
                for (i, names) in commanders.iter().enumerate() {
                    for name in names {
                        g.designate_commander(PlayerId(i as u8), name);
                    }
                }
                if check > 0 || usage.is_some() {
                    let also = usage.clone().map(|u| {
                        Arc::new(move |g: &Game, ev: &Event| {
                            u.lock().unwrap_or_else(|e| e.into_inner()).observe(g, ev)
                        }) as Arc<dyn Fn(&Game, &Event) + Send + Sync>
                    });
                    g.observer = Some(checks::observer(ledger.clone(), also));
                }
                g.logging = logging;
                // `Game::run`, one unit at a time so the watchdog sees progress and can
                // stop a slow game.
                if g.turn.stage == Stage::PreGame {
                    g.start();
                }
                if check > 0 {
                    ledger.lock().unwrap_or_else(|e| e.into_inner()).baseline(g);
                }
                while g.result.is_none() {
                    if stop2.load(Ordering::Relaxed) {
                        return None;
                    }
                    g.advance();
                    if g.turn.number > g.config.max_turns || g.actions_taken > g.config.max_actions
                    {
                        g.draw_game();
                    }
                    progress2.store(g.actions_taken, Ordering::Relaxed);
                }
                g.result.clone()
            }));
            // Tell the external agents how the game ended, and let them exit.
            for (seat, mut conn) in connections {
                if let Some(g) = game.as_ref() {
                    mtg_api::Transport::game_over(&mut conn, &mtg_api::GameOver::new(g, seat));
                }
                conn.shut_down();
            }
            let turn = game.as_ref().map_or(0, |g| g.turn.number);
            let log: Vec<String> = game.as_ref().map_or(Vec::new(), |g| {
                g.log
                    .iter()
                    .map(|l| format!("[T{}] {}", l.turn, l.text))
                    .collect()
            });
            let outcome = match r {
                Ok(Some(result)) => Outcome::Finished {
                    result,
                    turns: turn,
                    log,
                },
                Ok(None) => Outcome::Slow { turn, log },
                Err(_) => Outcome::Panicked {
                    message: take_panic().unwrap_or_else(|| "panic".into()),
                    turn,
                    log,
                },
            };
            let found =
                std::mem::take(&mut ledger.lock().unwrap_or_else(|e| e.into_inner()).violations);
            violations2
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .extend(found);
            let _ = tx.send(outcome);
        })
        .expect("spawn game thread");
    let start = Instant::now();
    let (mut seen, mut since) = (0, Instant::now());
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(outcome) => {
                let mut v =
                    std::mem::take(&mut *violations.lock().unwrap_or_else(|e| e.into_inner()));
                v.sort_by_key(|x| x.turn);
                return Some((outcome, v));
            }
            Err(RecvTimeoutError::Disconnected) => return None,
            Err(RecvTimeoutError::Timeout) => {
                let now = progress.load(Ordering::Relaxed);
                if now != seen {
                    (seen, since) = (now, Instant::now());
                } else if since.elapsed() > timeout {
                    return None;
                }
                if start.elapsed() > slow {
                    stop.store(true, Ordering::Relaxed);
                }
            }
        }
    }
}

/// Groups a violation or panic message by its kind: digits (object numbers, amounts)
/// replaced by N.
pub fn kind_of(what: &str) -> String {
    let mut out = String::new();
    let mut in_digits = false;
    for c in what.chars() {
        if c.is_ascii_digit() {
            if !in_digits {
                out.push('N');
            }
            in_digits = true;
        } else {
            in_digits = false;
            out.push(c);
        }
    }
    out
}
