//! Command-line simulator: runs games between decks with built-in agents.
//!
//! Usage: mtg-sim [--games N] [--seed S] [--deck FILE]... [--players N] [--random-decks]
//!                [--only G] [--max-turns N] [--max-actions N] [--slow SECS]
//!                [--timeout SECS] [--check N] [--log]
//!
//! A deck file lists "4 Lightning Bolt" lines; the cards after a "Sideboard" line are the
//! player's sideboard, from which a companion may be revealed as the game starts.
//!
//! `--random-decks` fuzzes the engine: each game gets new random decks of fully supported
//! cards for the players without a `--deck`. A panic doesn't stop the run: the game is
//! reported with the command that reproduces it (`--only G` replays game G of a run), and
//! the process exits with status 1 at the end. A game still going after `--slow` seconds
//! (random agents can pile up effects) is stopped and reported as slow. A game in which no
//! decision was made for `--timeout` seconds is reported as a hang and ends the run with
//! status 2.
//!
//! `--check N` checks every Nth priority decision that no state-based action was pending
//! when the player got priority (CR 117.5); a violation is reported like a panic. Fuzzing
//! checks every 4th decision unless told otherwise (`--check 0` turns it off).

mod checks;
mod decks;

use checks::{CheckingAgent, Violation};
use decks::{default_deck, load_deck, random_deck, DeckList};
use mtg_engine::agents::RandomAgent;
use mtg_engine::turn::Stage;
use mtg_engine::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The message and location of the latest panic, recorded by the panic hook.
static LAST_PANIC: Mutex<Option<String>> = Mutex::new(None);

enum Outcome {
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

/// Plays one game on its own thread (with a large stack), catching a panic. Returns
/// `None` if no decision was made for `timeout` (the game thread is left running).
fn play(
    config: GameConfig,
    decks: Vec<DeckList>,
    agent_seed: u64,
    logging: bool,
    check: u32,
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
            let mut game: Option<Game> = None;
            let r = panic::catch_unwind(AssertUnwindSafe(|| {
                let agents: Vec<Box<dyn Agent>> = (0..decks.len())
                    .map(|p| {
                        let a = RandomAgent::new(agent_seed.wrapping_add(p as u64));
                        if check > 0 {
                            Box::new(CheckingAgent::new(a, check, violations2.clone()))
                                as Box<dyn Agent>
                        } else {
                            Box::new(a) as Box<dyn Agent>
                        }
                    })
                    .collect();
                let g = game.insert(Game::new(
                    config,
                    decks.iter().map(|d| d.main.clone()).collect(),
                    agents,
                ));
                // Sideboards stay outside the game (a companion may be revealed from them).
                for (i, d) in decks.iter().enumerate() {
                    if !d.sideboard.is_empty() {
                        g.add_to_sideboard(PlayerId(i as u8), d.sideboard.clone());
                    }
                }
                g.logging = logging;
                // `Game::run`, one unit at a time so the watchdog sees progress and can
                // stop a slow game.
                if g.turn.stage == Stage::PreGame {
                    g.start();
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
                    message: LAST_PANIC
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .take()
                        .unwrap_or_else(|| "panic".into()),
                    turn,
                    log,
                },
            };
            let _ = tx.send(outcome);
        })
        .expect("spawn game thread");
    let start = Instant::now();
    let (mut seen, mut since) = (0, Instant::now());
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(outcome) => {
                let v = std::mem::take(&mut *violations.lock().unwrap_or_else(|e| e.into_inner()));
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

/// Prints the decks, the command that replays the game and the tail of its log.
fn report(decks: &[DeckList], repro: &str, log: &[String], tail: usize) {
    for (p, d) in decks.iter().enumerate() {
        println!("  deck {}: {}", p + 1, d.summary());
    }
    println!("  reproduce: {repro}");
    for l in &log[log.len().saturating_sub(tail)..] {
        println!("    {l}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut games = 100u64;
    let mut seed = 1u64;
    let mut deck_files: Vec<String> = Vec::new();
    let mut players = 0usize;
    let mut random = false;
    let mut only: Option<u64> = None;
    let mut max_turns = 100u32;
    let mut max_actions = GameConfig::default().max_actions;
    let mut slow = 60u64;
    let mut timeout = 60u64;
    let mut check: Option<u32> = None;
    let mut log = false;
    let mut i = 1;
    let value = |i: usize, what: &str| -> String {
        args.get(i + 1)
            .cloned()
            .unwrap_or_else(|| panic!("{} needs a value", what))
    };
    while i < args.len() {
        match args[i].as_str() {
            "--games" => games = value(i, "--games").parse().expect("--games N"),
            "--seed" => seed = value(i, "--seed").parse().expect("--seed S"),
            "--deck" => deck_files.push(value(i, "--deck")),
            "--players" => players = value(i, "--players").parse().expect("--players N"),
            "--only" => only = Some(value(i, "--only").parse().expect("--only G")),
            "--max-turns" => max_turns = value(i, "--max-turns").parse().expect("--max-turns N"),
            "--max-actions" => {
                max_actions = value(i, "--max-actions").parse().expect("--max-actions N")
            }
            "--check" => check = Some(value(i, "--check").parse().expect("--check N")),
            "--slow" => slow = value(i, "--slow").parse().expect("--slow SECS"),
            "--timeout" => timeout = value(i, "--timeout").parse().expect("--timeout SECS"),
            "--random-decks" => {
                random = true;
                i -= 1;
            }
            "--log" => {
                log = true;
                i -= 1;
            }
            other => panic!("unknown argument {other}"),
        }
        i += 2;
    }
    let players = players.max(deck_files.len()).max(2);
    let check = check.unwrap_or(if random { 4 } else { 0 });
    let fixed: Vec<DeckList> = deck_files.iter().map(|f| load_deck(f)).collect();
    panic::set_hook(Box::new(|info| {
        *LAST_PANIC.lock().unwrap_or_else(|e| e.into_inner()) = Some(info.to_string());
    }));

    let start = Instant::now();
    let mut wins = vec![0u64; players];
    let (mut played, mut draws, mut turns, mut finished) = (0u64, 0u64, 0u64, 0u64);
    let mut slow_games: Vec<u64> = Vec::new();
    let mut violations: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut panics: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let range = match only {
        Some(g) => g..g + 1,
        None => 0..games,
    };
    for gi in range {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(gi);
        let mut deck_rng = StdRng::seed_from_u64(s ^ 0x5eed_dec5);
        let decks: Vec<DeckList> = (0..players)
            .map(|p| match fixed.get(p) {
                Some(d) => d.clone(),
                None if random => random_deck(&mut deck_rng),
                None => default_deck(),
            })
            .collect();
        let config = GameConfig {
            seed: s,
            max_turns,
            max_actions,
            ..Default::default()
        };
        let repro = format!(
            "mtg-sim --seed {seed} --only {gi} --players {players}{}{} --max-turns {max_turns} --log",
            if random { " --random-decks" } else { "" },
            deck_files
                .iter()
                .map(|f| format!(" --deck {f}"))
                .collect::<String>()
        );
        // A panic mid-game needs the log to be useful, so fuzzing always records it.
        let logging = log || random;
        let Some(outcome) = play(
            config,
            decks.clone(),
            s.wrapping_mul(7),
            logging,
            check,
            Duration::from_secs(slow),
            Duration::from_secs(timeout),
        ) else {
            println!("HANG: game {gi} (seed {s}): no decision for {timeout}s");
            report(&decks, &repro, &[], 0);
            std::process::exit(2);
        };
        let (outcome, found) = outcome;
        played += 1;
        if let Some(first) = found.first() {
            println!(
                "RULES: game {gi} (seed {s}): {} violation(s); first on turn {}: {}",
                found.len(),
                first.turn,
                first.what
            );
            report(&decks, &repro, &[], 0);
            // Group by the kind of violation, without object numbers.
            let kind: String = first
                .what
                .split(|c: char| c.is_ascii_digit())
                .collect::<Vec<_>>()
                .join("N");
            violations.entry(kind).or_default().push(gi);
        }
        match outcome {
            Outcome::Finished {
                result,
                turns: t,
                log: lines,
            } => {
                finished += 1;
                turns += t as u64;
                match result {
                    GameResult::Win(ws) => {
                        for w in ws {
                            wins[w.idx()] += 1;
                        }
                    }
                    GameResult::Draw => draws += 1,
                }
                if log {
                    for l in lines {
                        println!("{l}");
                    }
                }
            }
            Outcome::Slow { turn, log: lines } => {
                println!("SLOW: game {gi} (seed {s}) stopped after {slow}s, on turn {turn}");
                report(&decks, &repro, &lines, if log { lines.len() } else { 8 });
                slow_games.push(gi);
            }
            Outcome::Panicked {
                message,
                turn,
                log: lines,
            } => {
                println!("PANIC: game {gi} (seed {s}), turn {turn}: {message}");
                report(&decks, &repro, &lines, if log { lines.len() } else { 12 });
                let place = message.lines().next().unwrap_or("").to_string();
                panics.entry(place).or_default().push(gi);
            }
        }
    }
    let el = start.elapsed();
    println!(
        "{played} games in {:.2?} ({:.1} games/s)",
        el,
        played as f64 / el.as_secs_f64()
    );
    for (i, w) in wins.iter().enumerate() {
        println!(
            "  player {}: {} wins ({:.1}%)",
            i + 1,
            w,
            100.0 * *w as f64 / played.max(1) as f64
        );
    }
    println!("  draws: {draws}");
    println!("  avg turns: {:.1}", turns as f64 / finished.max(1) as f64);
    if !slow_games.is_empty() {
        println!(
            "  slow (stopped): {} games {:?}",
            slow_games.len(),
            slow_games
        );
    }
    if !violations.is_empty() {
        println!(
            "  rules violations: {} games, {} kinds",
            violations.values().map(Vec::len).sum::<usize>(),
            violations.len()
        );
        for (kind, gs) in &violations {
            println!("    {} x {kind} (games {:?})", gs.len(), gs);
        }
    }
    if !panics.is_empty() || !violations.is_empty() {
        println!(
            "  panics: {} games, {} places",
            panics.values().map(Vec::len).sum::<usize>(),
            panics.len()
        );
        for (place, gs) in &panics {
            println!("    {} x {place} (games {:?})", gs.len(), gs);
        }
        std::process::exit(1);
    }
}
