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
//! `--check N` runs the rules checks of `checks.rs` before every Nth priority decision (no
//! state-based action pending, CR 117.5; consistent zones; up-to-date characteristics;
//! damage only on creatures; life totals and counters that add up to the events), and
//! watches every event (the stack and mana pools empty as each step ends, damage gone as
//! each turn begins); a violation is reported like a panic. Fuzzing checks every 4th
//! decision unless told otherwise (`--check 0` turns it off).
//!
//! `mtg-sim --every-card ...` plays games built around every fully supported card in turn
//! (see `every_card.rs`).

mod checks;
mod coverage;
mod decks;
mod every_card;
mod runner;

use decks::{default_deck, load_deck, random_deck, DeckList};
use mtg_engine::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use runner::{install_panic_hook, kind_of, play, GameSpec, Outcome};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Prints the decks, the command that replays the game and the tail of its log.
pub fn report(decks: &[DeckList], repro: &str, log: &[String], tail: usize) {
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
    if args.iter().any(|a| a == "--every-card") {
        every_card::run(&args[1..]);
        return;
    }
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
    install_panic_hook();

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
        let spec = GameSpec {
            config,
            decks: decks.clone(),
            agent_seed: s.wrapping_mul(7),
            logging,
            check,
            commanders: vec![],
            focus: None,
            usage: None,
        };
        let Some(outcome) = play(
            spec,
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
            violations.entry(kind_of(&first.what)).or_default().push(gi);
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
