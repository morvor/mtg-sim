//! `verified`: every card's tier (`mtg_engine::card_tiers`): unsupported < compiled <
//! round-trip < verified, with the checks each card fails, written to
//! `docs/verified-cards.tsv`.
//!
//! Inputs:
//! * the live compile status and the live Oracle round trip;
//! * `docs/roundtrip-passing.txt` (`--passing PATH`);
//! * `manual-check` for cards with hand-written abilities;
//! * a structure log directory (`--logs DIR`, default `target/structlog`), from
//!   `MTG_STRUCTURE_LOG=<dir> cargo test --workspace`;
//! * an every-card report (`--report FILE`, default `target/every-card.tsv`), from
//!   `mtg-sim --every-card --report FILE`.
//!
//! Options: `--write` (write `docs/verified-cards.tsv`), `--out PATH` (write there
//! instead), `--card NAME` (print one card's tier and checks).
//!
//! The regeneration recipe (about three hours, most of it the sweep, run in chunks):
//!
//! ```sh
//! rm -rf target/structlog
//! MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace          # structure log
//! # The every-card sweep: about 2.3 hours with 3 threads, so three chunks of ~46 min each
//! cargo run --release -p mtg-sim -- --every-card --threads 3 --from 0 --count 10100 --report target/every-card-0.tsv
//! cargo run --release -p mtg-sim -- --every-card --threads 3 --from 10100 --count 10100 --report target/every-card-1.tsv
//! cargo run --release -p mtg-sim -- --every-card --threads 3 --from 20200 --count 10100 --report target/every-card-2.tsv
//! cat target/every-card-[012].tsv > target/every-card.tsv
//! cargo run --release -p mtg-tools -- roundtrip --write docs/ROUNDTRIP.md     # if round trips changed
//! cargo run --release -p mtg-tools -- verified --logs target/structlog --report target/every-card.tsv --write
//! cargo run --release -p mtg-tools -- card-coverage --write docs/CARD_COVERAGE.md
//! ```

use mtg_engine::card::CardDef;
use mtg_engine::card_tiers::{check, classify, CardTier, Evidence, FuzzOutcome, Tier, TIERS_FILE};
use mtg_engine::structure;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

/// The cards the tier file lists: the ones `card-coverage` counts (playable paper cards
/// legal in some format, sticker sheets aside).
pub fn listed(c: &mtg_data::scryfall::ScryfallCard) -> bool {
    crate::roundtrip::counted(c) && !c.is_sticker_sheet()
}

/// Parses an every-card report's outcome lines
/// (`outcome<TAB>index<TAB>card<TAB>cast | exempt: why | not cast: why<TAB>kinds`).
pub fn parse_report(text: &str) -> HashMap<String, FuzzOutcome> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.first() != Some(&"outcome") || cols.len() < 4 {
            continue;
        }
        let name = cols[2];
        let cast = cols[3];
        let failures: Vec<String> = cols
            .get(4)
            .unwrap_or(&"")
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        let (cast, exempt, not_cast) = if cast == "cast" {
            (true, None, None)
        } else if let Some(why) = cast.strip_prefix("exempt: ") {
            (false, Some(why.to_string()), None)
        } else {
            let why = cast.strip_prefix("not cast: ").unwrap_or(cast);
            (false, None, Some(why.to_string()))
        };
        out.insert(
            name.to_lowercase(),
            FuzzOutcome::Swept {
                cast,
                exempt,
                not_cast,
                failures,
            },
        );
    }
    out
}

/// The cards a failed game's message names as objects ("Sight Beyond Sight #53 is listed in
/// ..."; the report writes the ids as `#N`): each object name is looked up in `names`
/// (lowercased card and face names to the card's key), longest match first.
pub fn named_in(message: &str, names: &HashMap<String, String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes = message.as_bytes();
    let mut i = 0;
    while let Some(k) = message[i..].find(" #") {
        let at = i + k;
        i = at + 2;
        if !bytes
            .get(at + 2)
            .is_some_and(|b| *b == b'N' || b.is_ascii_digit())
        {
            continue;
        }
        let before = &message[..at];
        // Every start of a word before the id, longest name first.
        let starts = std::iter::once(0).chain(
            before
                .char_indices()
                .filter(|(_, c)| *c == ' ' || *c == '(' || *c == '[')
                .map(|(j, _)| j + 1),
        );
        if let Some(card) = starts
            .filter_map(|j| names.get(&before[j..].to_lowercase()))
            .next()
        {
            if !out.contains(card) {
                out.push(card.clone());
            }
        }
    }
    out
}

/// Adds the report's failed games to the cards their messages name as objects (besides
/// the card each game was built around, which its outcome line already charges): a card
/// in a rules-check violation, panic or hang message was involved in it, so it isn't
/// fuzz-clean either. (The report doesn't list the other cards of a failed game's decks,
/// so those aren't charged.)
pub fn charge_named(
    text: &str,
    names: &HashMap<String, String>,
    fuzz: &mut HashMap<String, FuzzOutcome>,
) {
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.first() != Some(&"failure") || cols.len() < 3 {
            continue;
        }
        let around = cols[1]
            .split(" (--from ")
            .next()
            .unwrap_or("")
            .to_lowercase();
        let kind = cols[2].split(':').next().unwrap_or("").trim();
        for card in named_in(cols[2], names) {
            if card == around {
                continue;
            }
            let what = format!(
                "{kind} in a game of {}",
                cols[1].split(" (--from ").next().unwrap_or("")
            );
            if let Some(FuzzOutcome::Swept { failures, .. }) = fuzz.get_mut(&card) {
                if !failures.contains(&what) {
                    failures.push(what);
                }
            }
        }
    }
}

/// The notes column: why a check doesn't apply.
fn notes(e: &Evidence) -> String {
    match &e.fuzz {
        FuzzOutcome::Swept {
            cast: false,
            exempt: Some(why),
            ..
        } => format!("{} exempt: {why}", check::FUZZ_CAST),
        _ => String::new(),
    }
}

/// One card's row: its tier, the evidence and the notes.
pub struct Row {
    pub name: String,
    pub tier: CardTier,
    pub notes: String,
}

/// The tier file's text.
pub fn render(rows: &[Row], inputs: &str) -> String {
    let mut s = String::new();
    s.push_str("# Card tiers (crates/mtg-engine/src/card_tiers.rs): unsupported < compiled < round-trip < verified.\n");
    s.push_str("# Generated by `mtg-tools verified --write` (recipe in docs/CARD_COVERAGE.md); do not edit by hand.\n");
    s.push_str(&format!("# Inputs: {inputs}\n"));
    s.push_str("name\ttier\tfailed\tnotes\n");
    for r in rows {
        s.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            r.name,
            r.tier.tier,
            r.tier.failed_text(),
            r.notes
        ));
    }
    s
}

pub fn run(args: &[String], repo_root: PathBuf) {
    let root = repo_root.canonicalize().unwrap_or(repo_root);
    let mut logs = root.join("target/structlog");
    let mut report = root.join("target/every-card.tsv");
    let mut passing = root.join(crate::roundtrip::PASSING_LIST);
    let mut out: Option<PathBuf> = None;
    let mut card: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let value = |i: usize| -> String {
            args.get(i + 1).cloned().unwrap_or_else(|| {
                eprintln!("{} needs a value", args[i]);
                std::process::exit(2)
            })
        };
        match args[i].as_str() {
            "--logs" => {
                logs = PathBuf::from(value(i));
                i += 1;
            }
            "--report" => {
                report = PathBuf::from(value(i));
                i += 1;
            }
            "--passing" => {
                passing = PathBuf::from(value(i));
                i += 1;
            }
            "--write" => out = Some(root.join(TIERS_FILE)),
            "--out" => {
                out = Some(PathBuf::from(value(i)));
                i += 1;
            }
            "--card" => {
                card = Some(value(i).to_lowercase());
                i += 1;
            }
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
            "error: no structure logs in {} (run `MTG_STRUCTURE_LOG={} cargo test --workspace` first)",
            logs.display(),
            logs.display()
        );
        std::process::exit(1);
    }
    let mut names: HashMap<String, String> = HashMap::new();
    for c in mtg_data::cards().iter().filter(|c| listed(c)) {
        let key = c.name.to_lowercase();
        for f in c.card_faces.iter().flatten() {
            names.entry(f.name.to_lowercase()).or_insert(key.clone());
        }
        names.insert(key.clone(), key);
    }
    let fuzz = match std::fs::read_to_string(&report) {
        Ok(t) => {
            let mut f = parse_report(&t);
            charge_named(&t, &names, &mut f);
            f
        }
        Err(e) => {
            eprintln!(
                "error: can't read the every-card report {}: {e} (run `mtg-sim --every-card --report FILE`)",
                report.display()
            );
            std::process::exit(1);
        }
    };
    if fuzz.is_empty() {
        eprintln!(
            "error: {} has no outcome lines (made by an older mtg-sim?)",
            report.display()
        );
        std::process::exit(1);
    }
    let passing: HashSet<String> = std::fs::read_to_string(&passing)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.trim().to_lowercase())
        .collect();
    let mut manual: HashMap<String, Vec<String>> = HashMap::new();
    for (c, p) in mtg_engine::cards::manual_problems(&root.join("crates/mtg-engine/tests/cards")) {
        manual.entry(c.to_lowercase()).or_default().push(p);
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut seen = BTreeSet::new();
    for c in mtg_data::cards().iter() {
        if !listed(c) || !seen.insert(c.name.clone()) {
            continue;
        }
        // `--card` takes the card's name or one of its faces' names.
        if card.as_ref().is_some_and(|n| {
            *n != c.name.to_lowercase()
                && !c
                    .card_faces
                    .iter()
                    .flatten()
                    .any(|f| f.name.to_lowercase() == *n)
        }) {
            continue;
        }
        let def = CardDef::from_scryfall(c);
        let e = evidence(&def, &passing, &manual, &exercised, &fuzz);
        let tier = classify(&e);
        if card.is_some() {
            println!("{}: {}", def.name, tier.tier);
            for f in &tier.failed {
                println!("  fails {f}");
            }
            let n = notes(&e);
            if !n.is_empty() {
                println!("  {n}");
            }
        }
        rows.push(Row {
            name: def.name.to_string(),
            notes: notes(&e),
            tier,
        });
    }
    if card.is_some() {
        if rows.is_empty() {
            eprintln!("no such card among the listed cards");
            std::process::exit(1);
        }
        return;
    }
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    summary(&rows);
    if let Some(path) = out {
        let inputs = format!(
            "{} structure log files, every-card report of {} cards, {} round-trip-passing cards",
            std::fs::read_dir(&logs).map_or(0, |d| d.count()),
            fuzz.len(),
            passing.len()
        );
        std::fs::write(&path, render(&rows, &inputs)).expect("write the tier file");
        println!("wrote {}", path.display());
    }
}

fn evidence(
    def: &CardDef,
    passing: &HashSet<String>,
    manual: &HashMap<String, Vec<String>>,
    exercised: &HashMap<u64, Vec<(String, String)>>,
    fuzz: &HashMap<String, FuzzOutcome>,
) -> Evidence {
    let unsupported: Vec<String> = def
        .unsupported_text()
        .into_iter()
        .map(str::to_string)
        .collect();
    let key = def.name.to_lowercase();
    if !unsupported.is_empty() {
        return Evidence {
            unsupported,
            round_trip: false,
            manual_problems: vec![],
            structures: 0,
            unexercised: 0,
            fuzz: FuzzOutcome::Missing,
        };
    }
    // A card with hand-written abilities isn't in the passing list (`roundtrip` doesn't
    // check those): it passes when all but its hand-written blocks render back.
    let round_trip = (passing.contains(&key) || !def.manual_text().is_empty())
        && mtg_engine::card_tiers::round_trip_passes(def);
    let mut names: Vec<&str> = vec![def.name.as_str()];
    names.extend(def.faces.iter().map(|f| f.chars.name.as_str()));
    let mut hashes: BTreeSet<u64> = BTreeSet::new();
    for f in &def.faces {
        for a in &f.chars.abilities {
            hashes.insert(structure::hash(&structure::fingerprint(a, &names)));
        }
    }
    let unexercised = hashes.iter().filter(|h| !exercised.contains_key(h)).count();
    Evidence {
        unsupported,
        round_trip,
        manual_problems: if def.manual_text().is_empty() {
            vec![]
        } else {
            manual.get(&key).cloned().unwrap_or_default()
        },
        structures: hashes.len(),
        unexercised,
        fuzz: fuzz.get(&key).cloned().unwrap_or(FuzzOutcome::Missing),
    }
}

/// Prints the count in each tier and how many cards fail each check.
pub fn summary(rows: &[Row]) {
    let total = rows.len();
    let pct = |a: usize| 100.0 * a as f64 / total.max(1) as f64;
    println!("{total} cards");
    for t in Tier::ALL.iter().rev() {
        let n = rows.iter().filter(|r| r.tier.tier == *t).count();
        println!("  {:<12} {n:>6} ({:.1}%)", t.name(), pct(n));
    }
    let mut fails: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        for f in &r.tier.failed {
            *fails.entry(check_name(&f.check)).or_default() += 1;
        }
    }
    println!("failed checks:");
    for c in check::ALL {
        println!("  {c:<13} {:>6}", fails.get(c).copied().unwrap_or(0));
    }
}

fn check_name(c: &str) -> &str {
    check::ALL.into_iter().find(|k| *k == c).unwrap_or("other")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_outcome_lines_parse() {
        let text = "index\tcard\tnot used\tability\n\
            3\tA Card\tnever cast: never legal to cast\t\n\
            failure\tB Card (--from 4 --game 1)\tpanic: x\n\
            # outcome<TAB>...\n\
            outcome\t3\tA Card\tnot cast: never cast: never legal to cast\t\n\
            outcome\t4\tB Card\tcast\tpanic,slow\n\
            outcome\t5\tC Card\texempt: no mana cost\t\n";
        let r = parse_report(text);
        assert_eq!(r.len(), 3);
        assert_eq!(
            r["a card"],
            FuzzOutcome::Swept {
                cast: false,
                exempt: None,
                not_cast: Some("never cast: never legal to cast".into()),
                failures: vec![],
            }
        );
        let FuzzOutcome::Swept { cast, failures, .. } = &r["b card"] else {
            panic!()
        };
        assert!(cast);
        assert_eq!(failures, &vec!["panic".to_string(), "slow".to_string()]);
        let FuzzOutcome::Swept { exempt, .. } = &r["c card"] else {
            panic!()
        };
        assert_eq!(exempt.as_deref(), Some("no mana cost"));
    }

    #[test]
    fn a_card_a_failed_game_names_as_an_object_is_not_fuzz_clean() {
        let names: HashMap<String, String> = [
            ("sight beyond sight", "sight beyond sight"),
            ("sight", "sight"),
            ("maelstrom nexus", "maelstrom nexus"),
            (
                "delver of secrets",
                "delver of secrets // insectile aberration",
            ),
            (
                "insectile aberration",
                "delver of secrets // insectile aberration",
            ),
            ("grizzly bears", "grizzly bears"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
        assert_eq!(
            named_in(
                "as Cleanup ended: Insectile Aberration #12 and Sight Beyond Sight #N",
                &names
            ),
            vec![
                "delver of secrets // insectile aberration".to_string(),
                "sight beyond sight".to_string()
            ]
        );
        assert!(named_in("slow: stopped on turn N", &names).is_empty());
        let text = "failure\tMaelstrom Nexus (--from 15350 --game 0)\trules: Sight Beyond Sight #N is listed in Library(PlayerId(N)) but has become a new object\n\
            outcome\t15350\tMaelstrom Nexus\tcast\trules\n\
            outcome\t20000\tSight Beyond Sight\tcast\t\n\
            outcome\t9000\tGrizzly Bears\tcast\t\n";
        let mut fuzz = parse_report(text);
        charge_named(text, &names, &mut fuzz);
        let failures = |k: &str| match &fuzz[k] {
            FuzzOutcome::Swept { failures, .. } => failures.clone(),
            FuzzOutcome::Missing => panic!(),
        };
        assert_eq!(failures("maelstrom nexus"), vec!["rules".to_string()]);
        assert_eq!(
            failures("sight beyond sight"),
            vec!["rules in a game of Maelstrom Nexus".to_string()]
        );
        assert!(failures("grizzly bears").is_empty());
        let t = classify(&Evidence {
            unsupported: vec![],
            round_trip: true,
            manual_problems: vec![],
            structures: 1,
            unexercised: 0,
            fuzz: fuzz["sight beyond sight"].clone(),
        });
        assert_eq!(t.tier, Tier::RoundTrip);
        assert_eq!(
            t.failed_text(),
            "fuzz-clean (rules in a game of Maelstrom Nexus)"
        );
    }

    #[test]
    fn the_tier_file_reads_back() {
        let e = Evidence {
            unsupported: vec![],
            round_trip: true,
            manual_problems: vec![],
            structures: 3,
            unexercised: 1,
            fuzz: FuzzOutcome::Swept {
                cast: false,
                exempt: Some("no mana cost".into()),
                not_cast: None,
                failures: vec![],
            },
        };
        let rows = vec![Row {
            name: "A Card".into(),
            tier: classify(&e),
            notes: notes(&e),
        }];
        let text = render(&rows, "test");
        assert!(text.contains(
            "A Card\tround-trip\tstructure (1 of 3 unexercised)\tfuzz-cast exempt: no mana cost\n"
        ));
        let table = mtg_engine::card_tiers::TierTable::parse(&text);
        assert_eq!(table.recorded("A Card"), Some(&rows[0].tier));
    }
}
