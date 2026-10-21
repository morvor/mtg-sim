//! `mtg-sim --properties`: checks the metamorphic properties of `docs/PROPERTIES.md`
//! (`mtg_engine::properties`, and `mtg_api::replay` for replays) over random games.
//!
//! Usage: mtg-sim --properties [--property NAME]... [--games N] [--seed S] [--only G]
//!                [--players N] [--max-turns N] [--slow SECS] [--jobs N]
//!
//! NAME is `seat`, `snapshot`, `rollback`, `determinism`, `replay` or `all` (the default).
//! Each game gets new random decks; without `--players`, games cycle through 2, 2, 3 and
//! 4 players. A violation or a panic is reported with the command that reproduces it, and
//! the process exits with status 1 at the end; a game still going after `--slow` seconds
//! is reported as slow. `--jobs N` checks N games at a time.

use crate::decks::random_deck;
use crate::runner::{install_panic_hook, take_panic};
use mtg_engine::properties::{self, GameSetup};
use mtg_engine::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const ALL: [&str; 5] = ["seat", "snapshot", "rollback", "determinism", "replay"];

enum Outcome {
    /// Decisions made and reversed actions checked.
    Held(usize, u64),
    Violated(String),
    Panicked(String),
    Slow,
}

/// Checks `property` on the game of `setup` on its own thread.
fn check(property: &'static str, setup: GameSetup, slow: Duration) -> Outcome {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let n = setup.players();
            let r = panic::catch_unwind(AssertUnwindSafe(|| match property {
                "seat" => {
                    let rotation = 1 + (setup.seed as usize % (n - 1).max(1));
                    properties::seat_symmetry(&setup, rotation).map(|s| (s.decisions, s.rollbacks))
                }
                "snapshot" => {
                    let at = setup.seed % 1200;
                    properties::snapshot_equivalence(&setup, at).map(|s| (s.decisions, s.rollbacks))
                }
                "rollback" => {
                    properties::rollback_identity(&setup).map(|s| (s.decisions, s.rollbacks))
                }
                "determinism" => {
                    properties::determinism(&setup).map(|s| (s.decisions, s.rollbacks))
                }
                _ => {
                    // Requests carry whole observations, so replayed games are kept
                    // shorter.
                    let config = GameConfig {
                        seed: setup.seed,
                        max_turns: setup.max_turns.min(25),
                        max_actions: setup.max_actions * 2,
                        ..Default::default()
                    };
                    mtg_api::replay::replay_check(config, setup.decks.clone(), setup.seed, 100_000)
                        .map(|(s, _)| (s.requests, 0))
                }
            }));
            let out = match r {
                Ok(Ok((d, rb))) => Outcome::Held(d, rb),
                Ok(Err(e)) => Outcome::Violated(e),
                Err(_) => Outcome::Panicked(take_panic().unwrap_or_else(|| "panic".into())),
            };
            let _ = tx.send(out);
        })
        .expect("spawn a game thread");
    rx.recv_timeout(slow).unwrap_or(Outcome::Slow)
}

#[derive(Default)]
struct Tally {
    games: u64,
    held: u64,
    decisions: u64,
    rollbacks: u64,
    violations: Vec<u64>,
    panics: Vec<u64>,
    slow: Vec<u64>,
    time: Duration,
}

pub fn run(args: &[String]) {
    let mut games = 100u64;
    let mut seed = 1u64;
    let mut only: Option<u64> = None;
    let mut players: Option<usize> = None;
    let mut max_turns = 40u32;
    let mut slow = 120u64;
    let mut jobs = 1usize;
    let mut chosen: Vec<&'static str> = Vec::new();
    let mut i = 0;
    let value = |i: usize, what: &str| -> String {
        args.get(i + 1)
            .cloned()
            .unwrap_or_else(|| panic!("{what} needs a value"))
    };
    while i < args.len() {
        match args[i].as_str() {
            "--properties" => {
                i += 1;
                continue;
            }
            "--games" => games = value(i, "--games").parse().expect("--games N"),
            "--seed" => seed = value(i, "--seed").parse().expect("--seed S"),
            "--only" => only = Some(value(i, "--only").parse().expect("--only G")),
            "--players" => players = Some(value(i, "--players").parse().expect("--players N")),
            "--max-turns" => max_turns = value(i, "--max-turns").parse().expect("--max-turns N"),
            "--slow" => slow = value(i, "--slow").parse().expect("--slow SECS"),
            "--jobs" => jobs = value(i, "--jobs").parse().expect("--jobs N"),
            "--property" => {
                let v = value(i, "--property");
                if v == "all" {
                    chosen.extend(ALL);
                } else {
                    chosen.push(
                        ALL.iter()
                            .copied()
                            .find(|p| *p == v)
                            .unwrap_or_else(|| panic!("unknown property {v} ({ALL:?} or all)")),
                    );
                }
            }
            other => panic!("unknown argument {other}"),
        }
        i += 2;
    }
    if chosen.is_empty() {
        chosen.extend(ALL);
    }
    chosen.dedup();
    install_panic_hook();
    let start = Instant::now();
    let mut tallies: BTreeMap<&str, Tally> = BTreeMap::new();
    let range = match only {
        Some(g) => g..g + 1,
        None => 0..games,
    };
    // Games are checked by `jobs` workers; results are tallied as they come.
    let games_list: Vec<u64> = range.collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let (tx, rx) = mpsc::channel::<(&'static str, u64, u64, usize, Outcome, Duration)>();
    std::thread::scope(|scope| {
        for _ in 0..jobs.max(1) {
            let tx = tx.clone();
            let (next, games_list, chosen) = (&next, &games_list, &chosen);
            scope.spawn(move || loop {
                let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(&gi) = games_list.get(k) else { break };
                let s = seed.wrapping_mul(1_000_003).wrapping_add(gi);
                let n = players.unwrap_or([2, 2, 3, 4][(gi % 4) as usize]);
                let mut deck_rng = StdRng::seed_from_u64(s ^ 0x5eed_dec5);
                let decks: Vec<Vec<std::sync::Arc<CardDef>>> =
                    (0..n).map(|_| random_deck(&mut deck_rng).main).collect();
                let mut setup = GameSetup::new(decks, s);
                setup.max_turns = max_turns;
                for &property in chosen {
                    let t0 = Instant::now();
                    let out = check(property, setup.clone(), Duration::from_secs(slow));
                    let _ = tx.send((property, gi, s, n, out, t0.elapsed()));
                }
            });
        }
        drop(tx);
        for (property, gi, s, n, out, took) in rx {
            let t = tallies.entry(property).or_default();
            t.games += 1;
            t.time += took;
            let repro = format!(
                "mtg-sim --properties --property {property} --seed {seed} --only {gi} \
                 --players {n} --max-turns {max_turns}"
            );
            match out {
                Outcome::Held(d, rb) => {
                    t.held += 1;
                    t.decisions += d as u64;
                    t.rollbacks += rb;
                }
                Outcome::Violated(e) => {
                    println!("VIOLATION: {property}, game {gi} (seed {s}, {n} players): {e}");
                    println!("  reproduce: {repro}");
                    t.violations.push(gi);
                }
                Outcome::Panicked(m) => {
                    println!("PANIC: {property}, game {gi} (seed {s}, {n} players): {m}");
                    println!("  reproduce: {repro}");
                    t.panics.push(gi);
                }
                Outcome::Slow => {
                    println!("SLOW: {property}, game {gi} (seed {s}): stopped after {slow}s");
                    println!("  reproduce: {repro}");
                    t.slow.push(gi);
                }
            }
        }
    });
    println!("{:.2?}", start.elapsed());
    let mut failed = false;
    for (p, t) in &tallies {
        println!(
            "  {p}: {} games ({:.0?}), {} held, {} decisions{}; violations {:?}, panics {:?}, \
             slow {:?}",
            t.games,
            t.time,
            t.held,
            t.decisions,
            if *p == "rollback" {
                format!(", {} reversed actions checked", t.rollbacks)
            } else {
                String::new()
            },
            t.violations,
            t.panics,
            t.slow
        );
        failed |= !t.violations.is_empty() || !t.panics.is_empty();
    }
    if failed {
        std::process::exit(1);
    }
}
