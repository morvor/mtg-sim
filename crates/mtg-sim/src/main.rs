//! Command-line simulator: runs games between decks with built-in agents.
//!
//! Usage: mtg-sim [--games N] [--seed S] [--deck FILE]... [--players N] [--random-decks]
//!                [--only G] [--max-turns N] [--max-actions N] [--timeout SECS] [--log]
//!
//! A deck file lists "4 Lightning Bolt" lines; the cards after a "Sideboard" line are the
//! player's sideboard, from which a companion may be revealed as the game starts.
//!
//! `--random-decks` fuzzes the engine: each game gets new random decks of fully supported
//! cards for the players without a `--deck`. A panic doesn't stop the run: the game is
//! reported with the command that reproduces it (`--only G` replays game G of a run), and
//! the process exits with status 1 at the end. A game still running after `--timeout`
//! seconds is reported as a hang and ends the run with status 2.

mod decks;

use decks::{default_deck, load_deck, random_deck, DeckList};
use mtg_engine::agents::RandomAgent;
use mtg_engine::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The message and location of the latest panic, recorded by the panic hook.
static LAST_PANIC: Mutex<Option<String>> = Mutex::new(None);

enum Outcome {
    Finished {
        result: GameResult,
        turns: u32,
        log: Vec<String>,
    },
    Panicked {
        message: String,
        turn: u32,
        log: Vec<String>,
    },
}

/// Plays one game on its own thread (with a large stack) and catches a panic.
fn play(
    config: GameConfig,
    decks: Vec<DeckList>,
    agent_seed: u64,
    logging: bool,
    timeout: Duration,
) -> Option<Outcome> {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let mut game: Option<Game> = None;
            let r = panic::catch_unwind(AssertUnwindSafe(|| {
                let agents: Vec<Box<dyn Agent>> = (0..decks.len())
                    .map(|p| {
                        Box::new(RandomAgent::new(agent_seed.wrapping_add(p as u64)))
                            as Box<dyn Agent>
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
                g.run()
            }));
            let turn = game.as_ref().map_or(0, |g| g.turn.number);
            let log: Vec<String> = game.as_ref().map_or(Vec::new(), |g| {
                g.log
                    .iter()
                    .map(|l| format!("[T{}] {}", l.turn, l.text))
                    .collect()
            });
            let outcome = match r {
                Ok(result) => Outcome::Finished {
                    result,
                    turns: turn,
                    log,
                },
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
    rx.recv_timeout(timeout).ok()
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
    let mut timeout = 120u64;
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
    let fixed: Vec<DeckList> = deck_files.iter().map(|f| load_deck(f)).collect();
    panic::set_hook(Box::new(|info| {
        *LAST_PANIC.lock().unwrap_or_else(|e| e.into_inner()) = Some(info.to_string());
    }));

    let start = Instant::now();
    let mut wins = vec![0u64; players];
    let (mut played, mut draws, mut turns) = (0u64, 0u64, 0u64);
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
            Duration::from_secs(timeout),
        ) else {
            println!("HANG: game {gi} (seed {s}) still running after {timeout}s");
            for (p, d) in decks.iter().enumerate() {
                println!("  deck {}: {}", p + 1, d.summary());
            }
            println!("  reproduce: {repro}");
            std::process::exit(2);
        };
        played += 1;
        match outcome {
            Outcome::Finished {
                result,
                turns: t,
                log: lines,
            } => {
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
            Outcome::Panicked {
                message,
                turn,
                log: lines,
            } => {
                println!("PANIC: game {gi} (seed {s}), turn {turn}: {message}");
                for (p, d) in decks.iter().enumerate() {
                    println!("  deck {}: {}", p + 1, d.summary());
                }
                println!("  reproduce: {repro}");
                let tail = if log { lines.len() } else { 12 };
                for l in &lines[lines.len().saturating_sub(tail)..] {
                    println!("    {l}");
                }
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
    println!(
        "  avg turns: {:.1}",
        turns as f64 / (played - panics.values().map(Vec::len).sum::<usize>() as u64).max(1) as f64
    );
    if !panics.is_empty() {
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
