//! `structure-coverage`: groups the abilities of every supported card by their compiled
//! structure (see `mtg_engine::structure`) and reports which structures no test exercises.
//!
//! Usage:
//!
//! ```sh
//! rm -rf target/structlog
//! MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace
//! cargo run --release -p mtg-tools -- structure-coverage --write docs/STRUCTURE_COVERAGE.md
//! ```
//!
//! Options: `--logs DIR` (default `target/structlog`), `--limit N` (unexercised structures
//! listed, default 100), `--write PATH`, `--card NAME` (that card's abilities, their
//! structures and status), `--show-fingerprints` (with `--card`).

use mtg_engine::card::CardDef;
use mtg_engine::structure;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

struct Structure {
    kind: &'static str,
    /// Cards with an ability of this structure.
    cards: BTreeSet<String>,
    /// (card, ability text) examples.
    examples: Vec<(String, String)>,
}

fn md_escape(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " / ")
}

pub fn run(args: &[String], repo_root: PathBuf) {
    let repo_root = repo_root.canonicalize().unwrap_or(repo_root);
    let mut logs = repo_root.join("target/structlog");
    let mut limit = 100usize;
    let mut write: Option<String> = None;
    let mut card: Option<String> = None;
    let mut show_fp = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--logs" => {
                logs = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            "--limit" => {
                limit = args[i + 1].parse().expect("--limit N");
                i += 1;
            }
            "--write" => {
                write = Some(args[i + 1].clone());
                i += 1;
            }
            "--card" => {
                card = Some(args[i + 1].to_lowercase());
                i += 1;
            }
            "--show-fingerprints" => show_fp = true,
            other => {
                eprintln!("unknown option {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let exercised = structure::read_logs(&logs);
    if exercised.is_empty() {
        eprintln!(
            "warning: no structure logs in {} (run `MTG_STRUCTURE_LOG={} cargo test --workspace` first)",
            logs.display(),
            logs.display()
        );
    }

    let mut structures: HashMap<u64, Structure> = HashMap::new();
    // Card → its abilities' structures.
    let mut cards: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut total_cards = 0usize;
    for c in mtg_data::cards().iter() {
        if !c.is_playable_card() || !c.is_legal_somewhere() {
            continue;
        }
        if c.games.iter().all(|g| g != "paper") && !c.games.is_empty() {
            continue;
        }
        total_cards += 1;
        let def = CardDef::from_scryfall(c);
        if !def.is_fully_supported() {
            continue;
        }
        let mut names: Vec<&str> = vec![def.name.as_str()];
        names.extend(def.faces.iter().map(|f| f.chars.name.as_str()));
        let mut hashes = Vec::new();
        for f in &def.faces {
            for a in &f.chars.abilities {
                let fp = structure::fingerprint(a, &names);
                let h = structure::hash(&fp);
                if let Some(n) = &card {
                    if def.name.to_lowercase() == *n {
                        let status = if exercised.contains_key(&h) {
                            "exercised"
                        } else {
                            "NOT exercised"
                        };
                        println!(
                            "{h:016x}  {:<9}  {status:<13}  {}",
                            structure::kind_label(a),
                            a.text.replace('\n', " / ")
                        );
                        if show_fp {
                            println!("    {fp}");
                        }
                        if let Some(by) = exercised.get(&h) {
                            let v: Vec<String> =
                                by.iter().map(|(how, n)| format!("{n} ({how})")).collect();
                            println!("    exercised by: {}", v.join(", "));
                        }
                    }
                }
                let s = structures.entry(h).or_insert_with(|| Structure {
                    kind: structure::kind_label(a),
                    cards: BTreeSet::new(),
                    examples: Vec::new(),
                });
                if s.cards.insert(def.name.to_string()) && s.examples.len() < 3 {
                    s.examples.push((def.name.to_string(), a.text.clone()));
                }
                if !hashes.contains(&h) {
                    hashes.push(h);
                }
            }
        }
        cards.insert(def.name.to_string(), hashes);
    }
    if card.is_some() {
        return;
    }

    let is_ex = |h: &u64| exercised.contains_key(h);
    let n_struct = structures.len();
    let n_ex = structures.keys().filter(|h| is_ex(h)).count();
    let supported = cards.len();
    let with_abilities = cards.values().filter(|v| !v.is_empty()).count();
    let verified = cards
        .values()
        .filter(|v| !v.is_empty() && v.iter().all(is_ex))
        .count();
    // Card-weighted: (card, structure) pairs.
    let pairs: usize = cards.values().map(Vec::len).sum();
    let pairs_ex: usize = cards
        .values()
        .map(|v| v.iter().filter(|h| is_ex(h)).count())
        .sum();
    let pct = |a: usize, b: usize| 100.0 * a as f64 / b.max(1) as f64;

    // By kind.
    let mut by_kind: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (h, s) in &structures {
        let e = by_kind.entry(s.kind).or_default();
        e.0 += 1;
        if is_ex(h) {
            e.1 += 1;
        }
    }

    let mut unex: Vec<(&u64, &Structure)> = structures.iter().filter(|(h, _)| !is_ex(h)).collect();
    unex.sort_by(|a, b| {
        b.1.cards
            .len()
            .cmp(&a.1.cards.len())
            .then(a.1.examples[0].cmp(&b.1.examples[0]))
    });
    // Cards that a single unexercised structure keeps from being verified.
    let mut blocks: HashMap<u64, usize> = HashMap::new();
    for v in cards.values() {
        let missing: Vec<&u64> = v.iter().filter(|h| !is_ex(h)).collect();
        if missing.len() == 1 {
            *blocks.entry(*missing[0]).or_default() += 1;
        }
    }

    println!(
        "Structures: {n_struct}; exercised: {n_ex} ({:.1}%). Card abilities (card, structure) exercised: {pairs_ex}/{pairs} ({:.1}%). Structure-verified cards: {verified}/{with_abilities} with abilities ({:.1}%); {supported} supported cards of {total_cards}.",
        pct(n_ex, n_struct),
        pct(pairs_ex, pairs),
        pct(verified, with_abilities),
    );
    for (n, (h, s)) in unex.iter().take(limit.min(30)).enumerate() {
        println!(
            "{:>3}. {:>5} cards  {h:016x} {:<9} {}: {}",
            n + 1,
            s.cards.len(),
            s.kind,
            s.examples[0].0,
            s.examples[0].1.replace('\n', " / ")
        );
    }

    let Some(path) = write else {
        return;
    };
    let mut md = String::new();
    md.push_str("# Structure coverage\n\n");
    md.push_str("Generated by `structure-coverage` in `mtg-tools` from structure logs written while the tests run:\n\n```sh\nrm -rf target/structlog\nMTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace\ncargo run --release -p mtg-tools -- structure-coverage --write docs/STRUCTURE_COVERAGE.md\n```\n\n");
    md.push_str("Every ability of every supported card (paper, legal in some format, fully understood by the oracle compiler) compiles to a *structure*: its AST with literal parameters abstracted (numbers, mana amounts and colors, the card's own name and other names, creature types, counter kinds without rules of their own) and every semantic choice kept (see `crates/mtg-engine/src/structure.rs`). Abilities with the same structure run the same engine code. A structure is *exercised* when some test resolves a spell or ability with it, applies its static or replacement effect, or has the engine consult the keyword on an object. A card is *structure-verified* when every one of its abilities' structures is exercised. Use `--card \"Name\"` to see a card's abilities and their status.\n\n");
    md.push_str(&format!(
        "| | Count | Exercised | % |\n|---|---:|---:|---:|\n| Distinct structures | {n_struct} | {n_ex} | {:.1}% |\n| Card abilities (card, structure) | {pairs} | {pairs_ex} | {:.1}% |\n| Supported cards with abilities (structure-verified) | {with_abilities} | {verified} | {:.1}% |\n\n{supported} of {total_cards} cards are supported ({} of them have no abilities).\n\n",
        pct(n_ex, n_struct),
        pct(pairs_ex, pairs),
        pct(verified, with_abilities),
        supported - with_abilities,
    ));
    md.push_str("| Kind | Structures | Exercised | % |\n|---|---:|---:|---:|\n");
    for (k, (n, e)) in &by_kind {
        md.push_str(&format!("| {k} | {n} | {e} | {:.1}% |\n", pct(*e, *n)));
    }
    md.push_str(&format!(
        "\n## Unexercised structures by number of cards (top {limit})\n\n*Blocks*: cards for which this is the only unexercised structure.\n\n| # | Cards | Blocks | Kind | Structure | Examples |\n|---:|---:|---:|---|---|---|\n"
    ));
    for (n, (h, s)) in unex.iter().take(limit).enumerate() {
        let ex: Vec<String> = s
            .examples
            .iter()
            .map(|(c, t)| format!("**{}**: {}", md_escape(c), md_escape(t)))
            .collect();
        md.push_str(&format!(
            "| {} | {} | {} | {} | `{h:016x}` | {} |\n",
            n + 1,
            s.cards.len(),
            blocks.get(h).copied().unwrap_or(0),
            s.kind,
            ex.join("<br>")
        ));
    }
    std::fs::write(repo_root.join(&path), md).expect("write report");
    println!("wrote {path}");
}
