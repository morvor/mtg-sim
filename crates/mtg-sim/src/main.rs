//! Command-line simulator: runs games between decks with built-in agents.
//!
//! Usage: mtg-sim [--games N] [--seed S] [--deck FILE] [--deck FILE] [--log]

use mtg_engine::agents::RandomAgent;
use mtg_engine::*;
use std::sync::Arc;
use std::time::Instant;

fn load_deck(path: &str) -> Vec<Arc<CardDef>> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("can't read {path}: {e}"));
    parse_decklist(&text)
}

/// Parses "4 Lightning Bolt" style lines (sideboard and comments ignored).
fn parse_decklist(text: &str) -> Vec<Arc<CardDef>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') || l.starts_with("//") {
            continue;
        }
        if l.eq_ignore_ascii_case("sideboard") {
            break;
        }
        let (n, name) = match l.split_once(' ') {
            Some((n, rest)) if n.trim_end_matches('x').parse::<usize>().is_ok() => {
                (n.trim_end_matches('x').parse().unwrap(), rest.trim())
            }
            _ => (1, l),
        };
        let c = CardDb::global()
            .get(name)
            .unwrap_or_else(|| panic!("unknown card {name}"));
        for _ in 0..n {
            out.push(c.clone());
        }
    }
    out
}

fn default_deck() -> Vec<Arc<CardDef>> {
    parse_decklist(
        "12 Mountain\n12 Forest\n4 Grizzly Bears\n4 Lightning Bolt\n4 Hill Giant\n4 Shock\n4 Raging Goblin\n4 Gray Ogre\n4 Giant Growth\n4 Llanowar Elves\n4 Craw Wurm",
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut games = 100u64;
    let mut seed = 1u64;
    let mut decks: Vec<Vec<Arc<CardDef>>> = Vec::new();
    let mut log = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--games" => {
                games = args[i + 1].parse().expect("--games N");
                i += 1;
            }
            "--seed" => {
                seed = args[i + 1].parse().expect("--seed S");
                i += 1;
            }
            "--deck" => {
                decks.push(load_deck(&args[i + 1]));
                i += 1;
            }
            "--log" => log = true,
            other => panic!("unknown argument {other}"),
        }
        i += 1;
    }
    while decks.len() < 2 {
        decks.push(default_deck());
    }
    let start = Instant::now();
    let mut wins = vec![0u64; decks.len()];
    let mut draws = 0u64;
    let mut turns = 0u64;
    let mut spells = 0u64;
    for gi in 0..games {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(gi);
        let config = GameConfig {
            seed: s,
            max_turns: 100,
            ..Default::default()
        };
        let agents: Vec<Box<dyn Agent>> = (0..decks.len())
            .map(|p| Box::new(RandomAgent::new(s * 7 + p as u64)) as Box<dyn Agent>)
            .collect();
        let mut g = Game::new(config, decks.clone(), agents);
        g.logging = log;
        let result = g.run();
        turns += g.turn.number as u64;
        spells += g.turn_events.len() as u64;
        match result {
            GameResult::Win(ws) => {
                for w in ws {
                    wins[w.idx()] += 1;
                }
            }
            GameResult::Draw => draws += 1,
        }
        if log {
            for l in &g.log {
                println!("[T{}] {}", l.turn, l.text);
            }
        }
    }
    let el = start.elapsed();
    println!(
        "{games} games in {:.2?} ({:.1} games/s)",
        el,
        games as f64 / el.as_secs_f64()
    );
    for (i, w) in wins.iter().enumerate() {
        println!(
            "  player {}: {} wins ({:.1}%)",
            i + 1,
            w,
            100.0 * *w as f64 / games as f64
        );
    }
    println!("  draws: {draws}");
    println!("  avg turns: {:.1}", turns as f64 / games as f64);
    let _ = spells;
}
