//! `mtg-sim --properties --property copies|copies-dynamic|hidden|payment|damage`: the
//! properties checked over every card or over many small cases rather than over whole
//! random games (`docs/PROPERTIES.md`, properties 5–8).
//!
//! Usage: mtg-sim --properties --property NAME [--from N] [--count M] [--card NAME]
//!                [--seed S] [--games N] [--only G] [--jobs N]
//!
//! * `copies`: the static copy check of `mtg_engine::properties::copies` over every fully
//!   supported card (cards `--from N`, `--count M` of the pool sorted by name, or the one
//!   `--card`).
//! * `copies-dynamic`: the dynamic copy check over every `--stride`th card (default 100).
//! * `hidden`: hidden information (`mtg_api::hidden`) over `--games` random games.
//! * `payment`: mana payment (`mtg_engine::properties::payment`) over `--games` boards.
//! * `damage`: combat damage assignment (`mtg_engine::properties::damage`) over `--games`
//!   combats.
//!
//! A violation is printed with the command that reproduces it; the process exits with
//! status 1 if there was one.

use crate::runner::install_panic_hook;
use mtg_engine::properties::copies;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// The properties this module checks.
pub const NAMES: [&str; 5] = ["copies", "copies-dynamic", "hidden", "payment", "damage"];

struct Args {
    property: String,
    from: usize,
    count: Option<usize>,
    card: Option<String>,
    seed: u64,
    games: u64,
    only: Option<u64>,
    jobs: usize,
    stride: usize,
}

fn parse(args: &[String]) -> Args {
    let mut a = Args {
        property: String::new(),
        from: 0,
        count: None,
        card: None,
        seed: 1,
        games: 100,
        only: None,
        jobs: 1,
        stride: 100,
    };
    let value = |i: usize, what: &str| -> String {
        args.get(i + 1)
            .cloned()
            .unwrap_or_else(|| panic!("{what} needs a value"))
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--properties" => {
                i += 1;
                continue;
            }
            "--property" => a.property = value(i, "--property"),
            "--from" => a.from = value(i, "--from").parse().expect("--from N"),
            "--count" => a.count = Some(value(i, "--count").parse().expect("--count M")),
            "--card" => a.card = Some(value(i, "--card")),
            "--seed" => a.seed = value(i, "--seed").parse().expect("--seed S"),
            "--games" => a.games = value(i, "--games").parse().expect("--games N"),
            "--only" => a.only = Some(value(i, "--only").parse().expect("--only G")),
            "--jobs" => a.jobs = value(i, "--jobs").parse().expect("--jobs N"),
            "--stride" => a.stride = value(i, "--stride").parse().expect("--stride N"),
            other => panic!("unknown argument {other}"),
        }
        i += 2;
    }
    a
}

/// Runs `work(i)` for each `i` in `items` on `jobs` threads.
fn parallel<T: Sync, F: Fn(&T) + Sync>(items: &[T], jobs: usize, work: F) {
    let next = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..jobs.max(1) {
            s.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(k) else { break };
                work(item);
            });
        }
    });
}

pub fn run(args: &[String]) {
    let a = parse(args);
    install_panic_hook();
    let start = Instant::now();
    let failed = match a.property.as_str() {
        "copies" => run_copies(&a),
        other => panic!("unknown property {other} ({NAMES:?})"),
    };
    println!("{:.2?}", start.elapsed());
    if failed {
        std::process::exit(1);
    }
}

fn run_copies(a: &Args) -> bool {
    let t0 = Instant::now();
    let cards = match &a.card {
        Some(name) => copies::supported(std::slice::from_ref(name)),
        None => {
            let names = copies::pool_names();
            let end = a
                .count
                .map(|c| (a.from + c).min(names.len()))
                .unwrap_or(names.len());
            copies::supported(&names[a.from.min(end)..end])
        }
    };
    println!(
        "{} fully supported cards compiled in {:.1?}",
        cards.len(),
        t0.elapsed()
    );
    let chunks: Vec<&[std::sync::Arc<mtg_engine::CardDef>]> = cards.chunks(50).collect();
    let total = Mutex::new((
        0usize,
        std::collections::BTreeMap::<String, usize>::new(),
        Vec::<copies::Mismatch>::new(),
    ));
    parallel(&chunks, a.jobs, |chunk| {
        let (n, mismatches, skipped) = copies::check_cards(chunk);
        let mut t = total.lock().unwrap();
        t.0 += n;
        for (k, v) in skipped {
            *t.1.entry(k).or_insert(0) += v;
        }
        t.2.extend(mismatches);
    });
    let (n, skipped_by, mut mismatches) = total.into_inner().unwrap();
    for (why, k) in &skipped_by {
        println!("  not made: {why}: {k}");
    }
    let skipped: usize = skipped_by.values().sum();
    mismatches.sort_by(|x, y| x.card.cmp(&y.card));
    let mut cards_failing: Vec<&str> = mismatches.iter().map(|m| m.card.as_str()).collect();
    cards_failing.dedup();
    for m in &mismatches {
        println!("VIOLATION: {m}");
    }
    for c in &cards_failing {
        println!("  reproduce: mtg-sim --properties --property copies --card {c:?}");
    }
    println!(
        "copies: {} cards, {n} copies compared, {skipped} not made in the fixture; {} \
         mismatches on {} cards",
        cards.len(),
        mismatches.len(),
        cards_failing.len()
    );
    !mismatches.is_empty()
}
