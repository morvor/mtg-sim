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
        "hidden" => run_hidden(&a),
        "payment" => run_cases(&a, "payment", |s| {
            mtg_engine::properties::payment::check(s).map(|o| u64::from(o.payable))
        }),
        "damage" => run_cases(&a, "damage", |s| {
            mtg_engine::properties::damage::check(s).map(|c| c.assignments as u64)
        }),
        "copies-dynamic" => run_copies_dynamic(&a),
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

/// The case seeds of a run: `--games` cases from `--seed` (or the one `--only`).
fn case_seeds(a: &Args) -> Vec<(u64, u64)> {
    let range = match a.only {
        Some(g) => g..g + 1,
        None => 0..a.games,
    };
    range
        .map(|i| (i, a.seed.wrapping_mul(1_000_003).wrapping_add(i)))
        .collect()
}

/// Checks `check` on each case seed: a count it returns is summed (payable boards,
/// assignments compared).
fn run_cases(a: &Args, name: &str, check: impl Fn(u64) -> Result<u64, String> + Sync) -> bool {
    let seeds = case_seeds(a);
    let tally = Mutex::new((0u64, 0u64, Vec::<String>::new(), 0u64));
    parallel(&seeds, a.jobs, |(i, s)| {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(*s)));
        let mut t = tally.lock().unwrap();
        match r {
            Ok(Ok(n)) => {
                t.0 += 1;
                t.1 += n;
                t.3 += u64::from(n == 0);
            }
            Ok(Err(e)) => t.2.push(format!(
                "VIOLATION: {e}\n  reproduce: mtg-sim --properties --property {name} --seed {} --only {i}",
                a.seed
            )),
            Err(_) => t.2.push(format!(
                "PANIC: case {i} (seed {s}): {}\n  reproduce: mtg-sim --properties --property {name} --seed {} --only {i}",
                crate::runner::take_panic().unwrap_or_default(),
                a.seed
            )),
        }
    });
    let (held, n, mut bad, zero) = tally.into_inner().unwrap();
    bad.sort();
    for b in &bad {
        println!("{b}");
    }
    let what = if name == "payment" {
        format!("{n} payable")
    } else {
        format!("{n} assignments compared; {zero} skipped")
    };
    println!(
        "{name}: {} cases, {held} held ({what}), {} violations",
        seeds.len(),
        bad.len()
    );
    !bad.is_empty()
}

/// Hidden information over random games (2, 2, 3 and 4 players, new random decks of fully
/// supported cards), checking every 4th priority decision and every other decision.
fn run_hidden(a: &Args) -> bool {
    use rand::SeedableRng;
    let seeds = case_seeds(a);
    let tally = Mutex::new((0u64, 0u64, 0u64, Vec::<String>::new()));
    parallel(&seeds, a.jobs, |(i, s)| {
        let n = [2, 2, 3, 4][(*i % 4) as usize];
        let mut rng = rand::rngs::StdRng::seed_from_u64(s ^ 0x5eed_dec5);
        let decks: Vec<Vec<std::sync::Arc<mtg_engine::CardDef>>> = (0..n)
            .map(|_| crate::decks::random_deck(&mut rng, None).main)
            .collect();
        let config = mtg_engine::GameConfig {
            seed: *s,
            max_turns: 25,
            max_actions: 3000,
            ..Default::default()
        };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mtg_api::hidden::check_start(&config, &decks, *s)?;
            mtg_api::hidden::check_game(config.clone(), decks.clone(), *s, 4)
        }));
        let repro = format!(
            "mtg-sim --properties --property hidden --seed {} --only {i}",
            a.seed
        );
        let mut t = tally.lock().unwrap();
        match r {
            Ok(Ok(sum)) => {
                t.0 += 1;
                t.1 += sum.checked;
                t.2 += sum.changed;
            }
            Ok(Err(e)) => t.3.push(format!(
                "VIOLATION: game {i} (seed {s}, {n} players): {e}\n  reproduce: {repro}"
            )),
            Err(_) => t.3.push(format!(
                "PANIC: game {i} (seed {s}): {}\n  reproduce: {repro}",
                crate::runner::take_panic().unwrap_or_default()
            )),
        }
    });
    let (held, checked, changed, mut bad) = tally.into_inner().unwrap();
    bad.sort();
    for b in &bad {
        println!("{b}");
    }
    println!(
        "hidden: {} games, {held} held ({checked} decisions compared with a copy, {changed} \
         objects changed in the copies), {} violations",
        seeds.len(),
        bad.len()
    );
    !bad.is_empty()
}

/// The dynamic copy check over every `--stride`th card of the pool from `--from` (or the
/// one `--card`).
fn run_copies_dynamic(a: &Args) -> bool {
    let cards = match &a.card {
        Some(name) => copies::supported(std::slice::from_ref(name)),
        None => copies::sample(a.from, a.stride),
    };
    let chunks: Vec<&[std::sync::Arc<mtg_engine::CardDef>]> = cards.chunks(20).collect();
    let total = Mutex::new((0usize, 0usize, Vec::<String>::new()));
    parallel(&chunks, a.jobs, |chunk| {
        let (n, steps, bad) = copies::check_dynamic_cards(chunk);
        let mut t = total.lock().unwrap();
        t.0 += n;
        t.1 += steps;
        t.2.extend(bad);
    });
    let (n, steps, mut bad) = total.into_inner().unwrap();
    bad.sort();
    for b in &bad {
        let card = b.split(':').next().unwrap_or("");
        println!("VIOLATION: {b}\n  reproduce: mtg-sim --properties --property copies-dynamic --card {card:?}");
    }
    println!(
        "copies-dynamic: {} cards sampled, {n} permanents checked, {steps} steps compared, {} \
         differences",
        cards.len(),
        bad.len()
    );
    !bad.is_empty()
}
