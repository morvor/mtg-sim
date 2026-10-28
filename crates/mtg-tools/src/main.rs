//! Coverage and reporting tools.
//!
//! * `cr-coverage [--write PATH] [--check] [--range LO HI] [--examples]` — which
//!   Comprehensive Rules are cited by tests (via `cr!(...)` or `cr_example!(...)`) or
//!   exempted (`docs/cr-exemptions/*.tsv`), and which of the rules' worked examples are
//!   cited by `cr_example!(rule, substring)` or exempted (`docs/cr-exemptions/examples.tsv`);
//!   `--examples [--range 600-611]` lists each example with its rule and status.
//! * `card-coverage [--write PATH]` — how many cards the oracle compiler fully supports
//!   (and of those, how many round-trip: see `roundtrip`), and the most common
//!   unsupported ability texts.
//! * `roundtrip [--card NAME] [--filter TEXT] [--limit N] [--list] [--clusters N]
//!   [--write PATH] [--check] [--ast]` — renders every fully supported card's compiled
//!   abilities back to Oracle-style text and compares it with its Oracle text.
//! * `manual-check` — checks the hand-written card abilities (`mtg-engine/src/cards/`):
//!   none stale, none the compiler can now parse, none duplicated, every card tested.
//! * `rulings-coverage [--write PATH] [--check] [--card NAME] [--text SUBSTR]` — which
//!   Scryfall rulings are cited by tests (via `ruling!(...)`) or exempted
//!   (`docs/rulings-exemptions/*.tsv`); `--card`/`--text` print per-ruling status.
//! * `unsupported [--limit N] [--filter TEXT] [--card NAME] [--paper] [--single]` — dump
//!   unsupported ability texts with counts (`--card`: only that card's texts, by exact name,
//!   case-insensitive; `--paper`: only the cards `card-coverage` counts; `--single`: only
//!   cards whose sole unsupported text is that one, i.e. the cards it alone blocks; `--raw`:
//!   exact texts, numbers kept, e.g. to write a hand-written ability's `text`; `--by-card`:
//!   one `card<TAB>text` line per card and unsupported text, no counts). Sticker
//!   sheets are deferred and left out unless named with `--card`.
//! * `interpretations [--write [PATH]] [--check]` — the interpretation register
//!   (`docs/interpretations/`): prints its problems (see `mtg_data::interpretations::check`)
//!   and the index; `--write` writes the index to `docs/INTERPRETATIONS.md` (or PATH);
//!   `--check` exits with status 1 on a problem or when that file is out of date.
//! * `structure-coverage [--write PATH] [--limit N] [--logs DIR] [--card NAME]` — groups
//!   supported cards' abilities by compiled structure and reports the structures no test
//!   exercises (see `structure_coverage.rs` for the command sequence).
//! * `verified [--write] [--out PATH] [--logs DIR] [--report FILE] [--passing PATH]
//!   [--fuzz-from TIERS] [--gain N] [--card NAME]` — every card's tier (unsupported < compiled < round-trip < verified)
//!   and the checks it fails, into `docs/verified-cards.tsv` (see `verified.rs` for the
//!   regeneration recipe).

mod structure_coverage;
mod verified;

mod roundtrip;

use mtg_data::rules::RuleKind;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target" || n == ".git") {
                continue;
            }
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// All `cr!("...")` citations in the repository: rule id → files citing it.
pub fn cr_citations() -> BTreeMap<String, BTreeSet<String>> {
    let mut files = Vec::new();
    rust_files(&repo_root().join("crates"), &mut files);
    let macro_re = Regex::new(r#"(?s)cr!\s*\((.*?)\)"#).unwrap();
    let lit_re = Regex::new(r#""(\d{3}(?:\.\d+[a-z]*)?)""#).unwrap();
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let rel = f
            .strip_prefix(repo_root())
            .unwrap_or(&f)
            .display()
            .to_string();
        for m in macro_re.captures_iter(&text) {
            for l in lit_re.captures_iter(&m[1]) {
                out.entry(l[1].to_string()).or_default().insert(rel.clone());
            }
        }
    }
    // `cr_example!("rule", "...")` also cites its rule.
    for (rule, _, file) in cr_example_citations() {
        out.entry(rule).or_default().insert(file);
    }
    out
}

/// All `cr_example!("rule", "substring")` citations in the engine's sources and tests:
/// (rule, substring, file). Only the engine: mtg-data documents the macro.
pub fn cr_example_citations() -> Vec<(String, String, String)> {
    let mut files = Vec::new();
    rust_files(&repo_root().join("crates/mtg-engine"), &mut files);
    let re = Regex::new(
        r#"(?s)cr_example!\s*\(\s*"(\d{3}(?:\.\d+[a-z]*)?)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)"#,
    )
    .unwrap();
    let mut out = Vec::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let rel = f
            .strip_prefix(repo_root())
            .unwrap_or(&f)
            .display()
            .to_string();
        for m in re.captures_iter(&text) {
            out.push((m[1].to_string(), unescape(&m[2]), rel.clone()));
        }
    }
    out
}

/// The file listing exempted CR examples: `rule<TAB>substring<TAB>reason`.
const EXAMPLE_EXEMPTIONS: &str = "docs/cr-exemptions/examples.tsv";

/// Exempted CR examples from [`EXAMPLE_EXEMPTIONS`]: (rule, substring, reason, line).
pub fn example_exemptions() -> Vec<(String, String, String, usize)> {
    let mut out = Vec::new();
    let Ok(text) = std::fs::read_to_string(repo_root().join(EXAMPLE_EXEMPTIONS)) else {
        return out;
    };
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, '\t');
        let rule = parts.next().unwrap_or("").trim().to_string();
        let needle = parts.next().unwrap_or("").trim().to_string();
        let reason = parts.next().unwrap_or("").trim().to_string();
        out.push((rule, needle, reason, i + 1));
    }
    out
}

/// Whether rule `id` is within `[lo, hi]` in document order, a bound including the rules
/// under it: "611" includes 611.1 and 611.2c, "702.19" includes 702.19a..702.19z.
fn rule_in_range(id: &str, lo: &str, hi: &str) -> bool {
    let key = mtg_data::rules::rule_sort_key;
    let under = |b: &str| {
        id == b
            || id.strip_prefix(b).is_some_and(|rest| {
                rest.starts_with('.') && !b.contains('.')
                    || rest.starts_with(|c: char| c.is_ascii_lowercase())
            })
    };
    (key(id) >= key(lo) || under(lo)) && (key(id) <= key(hi) || under(hi))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExampleStatus {
    Covered,
    Exempt,
    Open,
}

/// Every worked example in the CR, with its coverage status, plus the problems found in
/// citations and exemptions (unknown rule, substring matching none or several examples,
/// missing reason).
struct ExampleCoverage {
    /// (rule id, example text, status, citing files or exemption reason)
    examples: Vec<(String, String, ExampleStatus, String)>,
    problems: Vec<String>,
}

fn example_coverage() -> ExampleCoverage {
    let rules = mtg_data::comprehensive_rules();
    let mut problems = Vec::new();
    // (rule, example index) → citing files / exemption reason
    let mut cited: BTreeMap<(String, usize), BTreeSet<String>> = BTreeMap::new();
    let mut exempt: BTreeMap<(String, usize), String> = BTreeMap::new();
    let resolve = |rule: &str, needle: &str| -> Result<usize, String> {
        let Some(r) = rules.get(rule) else {
            return Err("no such rule".into());
        };
        let n = mtg_data::normalize_text(needle);
        if n.is_empty() {
            return Err("empty substring".into());
        }
        let hits: Vec<usize> = r
            .examples
            .iter()
            .enumerate()
            .filter(|(_, e)| mtg_data::normalize_text(e).contains(&n))
            .map(|(i, _)| i)
            .collect();
        match hits.as_slice() {
            [i] => Ok(*i),
            [] => Err("matches none of the rule's examples".into()),
            _ => Err(format!("matches {} of the rule's examples", hits.len())),
        }
    };
    for (rule, needle, file) in cr_example_citations() {
        match resolve(&rule, &needle) {
            Ok(i) => {
                cited.entry((rule, i)).or_default().insert(file);
            }
            Err(e) => problems.push(format!("citation {rule} {needle:?} ({file}): {e}")),
        }
    }
    for (rule, needle, reason, line) in example_exemptions() {
        if reason.is_empty() {
            problems.push(format!(
                "exemption {rule} {needle:?} ({EXAMPLE_EXEMPTIONS}:{line}): no reason"
            ));
        }
        match resolve(&rule, &needle) {
            Ok(i) => {
                exempt.insert((rule, i), reason);
            }
            Err(e) => problems.push(format!(
                "exemption {rule} {needle:?} ({EXAMPLE_EXEMPTIONS}:{line}): {e}"
            )),
        }
    }
    let mut examples = Vec::new();
    for r in rules.all() {
        for (i, e) in r.examples.iter().enumerate() {
            let key = (r.id.clone(), i);
            let (st, note) = if let Some(files) = cited.get(&key) {
                (
                    ExampleStatus::Covered,
                    files.iter().cloned().collect::<Vec<_>>().join(", "),
                )
            } else if let Some(reason) = exempt.get(&key) {
                (ExampleStatus::Exempt, reason.clone())
            } else {
                (ExampleStatus::Open, String::new())
            };
            examples.push((r.id.clone(), e.clone(), st, note));
        }
    }
    ExampleCoverage { examples, problems }
}

/// Exemptions from `docs/cr-exemptions.tsv` and every `docs/cr-exemptions/*.tsv`.
pub fn exemptions() -> BTreeMap<String, String> {
    let mut paths = vec![repo_root().join("docs/cr-exemptions.tsv")];
    if let Ok(rd) = std::fs::read_dir(repo_root().join("docs/cr-exemptions")) {
        let mut v: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "tsv"))
            .filter(|p| !p.ends_with(EXAMPLE_EXEMPTIONS))
            .collect();
        v.sort();
        paths.extend(v);
    }
    let mut out = BTreeMap::new();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            if let Some((id, reason)) = l.split_once('\t') {
                out.insert(id.trim().to_string(), reason.trim().to_string());
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Cited,
    ViaSubrules,
    Exempt,
    Uncovered,
}

fn cr_coverage(args: &[String]) {
    let rules = mtg_data::comprehensive_rules();
    let cites = cr_citations();
    let ex = exemptions();
    // Invalid citations.
    let invalid: Vec<&String> = cites.keys().filter(|id| rules.get(id).is_none()).collect();
    let mut status: HashMap<String, Status> = HashMap::new();
    // Compute bottom-up: process numbered rules in reverse document order.
    let numbered: Vec<&mtg_data::rules::Rule> = rules.numbered_rules().collect();
    for r in numbered.iter().rev() {
        let s = if cites.contains_key(&r.id) {
            Status::Cited
        } else if ex.contains_key(&r.id) {
            Status::Exempt
        } else if !r.children.is_empty()
            && r.children.iter().all(|c| {
                matches!(
                    status.get(c),
                    Some(Status::Cited | Status::ViaSubrules | Status::Exempt)
                )
            })
        {
            Status::ViaSubrules
        } else {
            Status::Uncovered
        };
        status.insert(r.id.clone(), s);
    }
    let total = numbered.len();
    let count = |s: Status| numbered.iter().filter(|r| status[&r.id] == s).count();
    let (cited, via, exempt, unc) = (
        count(Status::Cited),
        count(Status::ViaSubrules),
        count(Status::Exempt),
        count(Status::Uncovered),
    );
    let mut md = String::new();
    md.push_str(&format!(
        "# Comprehensive Rules coverage\n\nRules effective {}. Generated by `cargo run -p mtg-tools -- cr-coverage --write docs/CR_COVERAGE.md`.\n\n",
        rules.effective_date
    ));
    md.push_str("A rule is **cited** when a test calls `cr!(\"<rule>\")` (which also verifies the rule exists), **covered via subrules** when every subrule is covered, or **exempt** with a reason in `docs/cr-exemptions.tsv`.\n\n");
    md.push_str(&format!(
        "| | Rules | % |\n|---|---:|---:|\n| Cited by tests | {cited} | {:.1}% |\n| Covered via subrules | {via} | {:.1}% |\n| Exempt (with reason) | {exempt} | {:.1}% |\n| **Uncovered** | **{unc}** | {:.1}% |\n| Total numbered rules | {total} | |\n\n",
        100.0 * cited as f64 / total as f64,
        100.0 * via as f64 / total as f64,
        100.0 * exempt as f64 / total as f64,
        100.0 * unc as f64 / total as f64
    ));
    // Worked examples.
    let exc = example_coverage();
    let ex_total = exc.examples.len();
    let ex_count = |s: ExampleStatus| exc.examples.iter().filter(|e| e.2 == s).count();
    let (ex_cov, ex_ex, ex_open) = (
        ex_count(ExampleStatus::Covered),
        ex_count(ExampleStatus::Exempt),
        ex_count(ExampleStatus::Open),
    );
    let pct = |n: usize| 100.0 * n as f64 / ex_total.max(1) as f64;
    md.push_str("## Worked examples\n\nThe paragraphs starting \"Example:\" that follow a rule. An example is **covered** when a test cites it with `cr_example!(\"<rule>\", \"<substring>\")` (which fails unless exactly one of that rule's examples contains the substring), or **exempt** with a reason in `docs/cr-exemptions/examples.tsv`. List them with `cr-coverage --examples [--range 600 611]`.\n\n");
    md.push_str(&format!(
        "| | Examples | % |\n|---|---:|---:|\n| Covered by tests | {ex_cov} | {:.1}% |\n| Exempt (with reason) | {ex_ex} | {:.1}% |\n| **Open** | **{ex_open}** | {:.1}% |\n| Total examples | {ex_total} | |\n\n",
        pct(ex_cov),
        pct(ex_ex),
        pct(ex_open)
    ));
    // Per section.
    md.push_str("## By section\n\n| Section | Title | Rules | Covered | Uncovered |\n|---|---|---:|---:|---:|\n");
    for sec in rules.all().filter(|r| r.kind == RuleKind::Section) {
        let rs: Vec<&&mtg_data::rules::Rule> = numbered
            .iter()
            .filter(|r| r.id.starts_with(&format!("{}.", sec.id)))
            .collect();
        let covered = rs
            .iter()
            .filter(|r| status[&r.id] != Status::Uncovered)
            .count();
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            sec.id,
            sec.text,
            rs.len(),
            covered,
            rs.len() - covered
        ));
    }
    md.push_str("\n## Uncovered rules\n\n");
    for r in &numbered {
        if status[&r.id] == Status::Uncovered {
            let first = r.text.lines().next().unwrap_or("");
            let short: String = first.chars().take(140).collect();
            md.push_str(&format!("- **{}** {}\n", r.id, short));
        }
    }
    if !ex.is_empty() {
        md.push_str("\n## Exemptions\n\n");
        for (id, reason) in &ex {
            md.push_str(&format!("- **{id}**: {reason}\n"));
        }
    }
    let mut write = None;
    let mut check = false;
    let mut examples_mode = false;
    let mut range: Option<(String, String)> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write" => {
                write = Some(args[i + 1].clone());
                i += 1;
            }
            "--check" => check = true,
            "--examples" => examples_mode = true,
            "--range" => {
                // `--range 600 611` or `--range 600-611`.
                let a = args[i + 1].clone();
                if let Some((lo, hi)) = a.split_once('-') {
                    range = Some((lo.to_string(), hi.to_string()));
                    i += 1;
                } else {
                    range = Some((a, args[i + 2].clone()));
                    i += 2;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if examples_mode {
        // Print every example (in the range, if given) with its rule and status.
        let in_range = |id: &str| match &range {
            None => true,
            Some((lo, hi)) => rule_in_range(id, lo, hi),
        };
        let (mut n, mut cov, mut exm, mut open) = (0, 0, 0, 0);
        for (id, text, st, note) in &exc.examples {
            if !in_range(id) {
                continue;
            }
            n += 1;
            let label = match st {
                ExampleStatus::Covered => {
                    cov += 1;
                    "covered"
                }
                ExampleStatus::Exempt => {
                    exm += 1;
                    "exempt"
                }
                ExampleStatus::Open => {
                    open += 1;
                    "OPEN"
                }
            };
            println!("{label:>8}  {id:<9} {}", text.replace('\n', " / "));
            if !note.is_empty() {
                println!("{:>8}  {:<9}   -> {note}", "", "");
            }
        }
        match &range {
            Some((lo, hi)) => print!("range {lo}..{hi}: "),
            None => print!("all rules: "),
        }
        println!("{n} examples, {cov} covered, {exm} exempt, {open} open");
        for p in &exc.problems {
            println!("INVALID example {p}");
        }
        if check && (open > 0 || !exc.problems.is_empty()) {
            std::process::exit(1);
        }
        return;
    }
    if let Some((lo, hi)) = &range {
        // Print the status of every rule whose id is within [lo, hi] (inclusive, by document
        // order prefix: "702.19" includes 702.19a..702.19z).
        let in_range = |id: &str| rule_in_range(id, lo, hi);
        let mut n = 0;
        let mut unc = 0;
        for r in &numbered {
            if !in_range(&r.id) {
                continue;
            }
            n += 1;
            let st = match status[&r.id] {
                Status::Cited => "cited",
                Status::ViaSubrules => "via-subrules",
                Status::Exempt => "exempt",
                Status::Uncovered => {
                    unc += 1;
                    "UNCOVERED"
                }
            };
            let first: String = r
                .text
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(110)
                .collect();
            println!("{st:>12}  {:<9} {first}", r.id);
        }
        println!("range {lo}..{hi}: {n} rules, {unc} uncovered");
        return;
    }
    println!(
        "CR coverage: {} cited, {} via subrules, {} exempt, {} uncovered of {} ({:.1}% covered)",
        cited,
        via,
        exempt,
        unc,
        total,
        100.0 * (total - unc) as f64 / total as f64
    );
    println!(
        "CR examples: {ex_cov} covered, {ex_ex} exempt, {ex_open} open of {ex_total} ({:.1}% covered or exempt)",
        pct(ex_cov + ex_ex)
    );
    if !invalid.is_empty() {
        println!("INVALID citations (no such rule): {invalid:?}");
    }
    for p in &exc.problems {
        println!("INVALID example {p}");
    }
    for (id, _) in &ex {
        if rules.get(id).is_none() {
            println!("INVALID exemption (no such rule): {id}");
        }
    }
    if let Some(p) = write {
        std::fs::write(repo_root().join(&p), md).expect("write report");
        println!("wrote {p}");
    }
    if check && (unc > 0 || !invalid.is_empty() || !exc.problems.is_empty()) {
        std::process::exit(1);
    }
}

/// Normalizes ability text into a pattern: numbers → N, mana symbols kept.
fn pattern_of(text: &str) -> String {
    let re = Regex::new(r"\b\d+\b").unwrap();
    re.replace_all(text, "N").to_string()
}

/// How to regenerate the tier file and this report, for CARD_COVERAGE.md.
const TIER_RECIPE: &str = "```sh\nexport CARGO_BUILD_JOBS=2\nrm -rf target/structlog\nMTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace          # structure log\n# The every-card sweep: about 2.3 hours with 3 threads, so three chunks of ~46 min each\ncargo run --release -p mtg-sim -- --every-card --threads 3 --from 0 --count 10100 --report target/every-card-0.tsv\ncargo run --release -p mtg-sim -- --every-card --threads 3 --from 10100 --count 10100 --report target/every-card-1.tsv\ncargo run --release -p mtg-sim -- --every-card --threads 3 --from 20200 --count 10100 --report target/every-card-2.tsv\ncat target/every-card-[012].tsv > target/every-card.tsv\ncargo run --release -p mtg-tools -- roundtrip --write docs/ROUNDTRIP.md     # if round trips changed\ncargo run --release -p mtg-tools -- verified --logs target/structlog --report target/every-card.tsv --write\ncargo run --release -p mtg-tools -- card-coverage --write docs/CARD_COVERAGE.md\n```\n";

fn card_coverage(args: &[String]) {
    use mtg_engine::card::CardDef;
    use mtg_engine::card_tiers::{check, Tier, TierTable, TIERS_FILE};
    let db = mtg_data::cards();
    let table = TierTable::load(&repo_root().join(TIERS_FILE));
    if table.is_empty() {
        eprintln!("warning: {TIERS_FILE} is missing or empty: every card counts as at most compiled (run `mtg-tools verified --write`)");
    }
    let mut total = 0usize;
    let mut full = 0usize;
    let mut tiers: BTreeMap<Tier, usize> = BTreeMap::new();
    let mut fails: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_abilities = 0usize;
    let mut supported_abilities = 0usize;
    let mut patterns: HashMap<String, usize> = HashMap::new();
    // Format → (cards, count at least each tier, indexed by Tier as usize).
    let mut by_format: BTreeMap<&str, (usize, [usize; 4])> = BTreeMap::new();
    let mut manual_cards: Vec<String> = Vec::new();
    let formats = [
        "standard",
        "pioneer",
        "modern",
        "legacy",
        "vintage",
        "commander",
        "pauper",
    ];
    // Sticker sheets are deferred (docs/DEFERRED.md): counted separately.
    let mut sticker_sheets = 0usize;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for c in db.iter() {
        if !c.is_playable_card() || !c.is_legal_somewhere() {
            continue;
        }
        if !c.is_paper() {
            continue;
        }
        if c.is_sticker_sheet() {
            sticker_sheets += 1;
            continue;
        }
        total += 1;
        let def = CardDef::from_scryfall(c);
        let unsupported = def.unsupported_text();
        let n_abilities: usize = def.faces.iter().map(|f| f.chars.abilities.len()).sum();
        total_abilities += n_abilities;
        supported_abilities += n_abilities.saturating_sub(unsupported.len());
        let ok = unsupported.is_empty();
        let tier = table.tier_of(&def);
        *tiers.entry(tier.tier).or_default() += 1;
        if ok && seen.insert(c.name.clone()) {
            for f in &tier.failed {
                *fails.entry(f.check.clone()).or_default() += 1;
            }
        }
        if ok {
            full += 1;
            if !def.manual_text().is_empty() {
                manual_cards.push(c.name.clone());
            }
        }
        for f in formats {
            if c.legality(f)
                .is_some_and(|l| l == "legal" || l == "restricted")
            {
                let e = by_format.entry(f).or_default();
                e.0 += 1;
                for t in Tier::ALL {
                    if tier.tier >= t {
                        e.1[t as usize] += 1;
                    }
                }
            }
        }
        for u in unsupported {
            *patterns.entry(pattern_of(u)).or_default() += 1;
        }
    }
    let pct = |a: usize, b: usize| 100.0 * a as f64 / b.max(1) as f64;
    let at_least =
        |t: Tier| -> usize { tiers.iter().filter(|(k, _)| **k >= t).map(|(_, n)| n).sum() };
    let verified = at_least(Tier::Verified);
    let mut pats: Vec<(String, usize)> = patterns.into_iter().collect();
    pats.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!(
        "Verified: {verified}/{total} ({:.1}%); round-trip or better: {} ({:.1}%); compiled (fully supported): {full} ({:.1}%)",
        pct(verified, total),
        at_least(Tier::RoundTrip),
        pct(at_least(Tier::RoundTrip), total),
        pct(full, total),
    );
    println!(
        "Cards fully supported: {full}/{total} ({:.1}%); abilities supported: {supported_abilities}/{total_abilities} ({:.1}%)",
        pct(full, total),
        pct(supported_abilities, total_abilities)
    );
    let n_manual = manual_cards.len();
    println!(
        "Fully supported: {full} (compiled: {}, with hand-written abilities: {n_manual})",
        full - n_manual
    );
    println!("Deferred, not counted: {sticker_sheets} sticker sheets (see docs/DEFERRED.md)");
    let mut md = String::new();
    md.push_str("# Card coverage\n\nGenerated by `cargo run --release -p mtg-tools -- card-coverage --write docs/CARD_COVERAGE.md` from `docs/verified-cards.tsv` (`mtg-tools verified --write`) and the live compiler.\n\n");
    md.push_str(&format!(
        "**Verified: {verified} of {total} cards ({:.1}%)** (paper cards legal in at least one format).\n\n",
        pct(verified, total)
    ));
    md.push_str("A card is *verified* when it passes every check we have: every ability compiled (or hand-written and tested), the compiled abilities render back to the Oracle text (the round trip, `docs/ROUNDTRIP.md`), every ability structure on it is exercised by some test (`docs/STRUCTURE_COVERAGE.md`), and the latest every-card sweep cast or played it with no panic, hang, slow game or rules-check violation involving it (in the games built around it, or naming it; `docs/EVERY_CARD.md`; a card the sweep can't cast, with no mana cost or in a supplementary deck, is exempt from being cast). Simulations use only verified cards by default (`mtg-sim --cards`, `mtg_api::ProtocolOptions::min_card_tier`).\n\n");
    md.push_str(
        "| Tier | Means | Cards | % | At least this tier | % |\n|---|---|---:|---:|---:|---:|\n",
    );
    for (t, means) in [
        (
            Tier::Verified,
            "round-trip, structure-verified and fuzz-clean",
        ),
        (
            Tier::RoundTrip,
            "compiled, and the Oracle round trip passes (for a card with hand-written abilities: everything but those, once `manual-check` passes)",
        ),
        (
            Tier::Compiled,
            "every ability compiled or hand-written (fully supported)",
        ),
        (Tier::Unsupported, "some ability not compiled"),
    ] {
        let n = tiers.get(&t).copied().unwrap_or(0);
        md.push_str(&format!(
            "| {t} | {means} | {n} | {:.1}% | {} | {:.1}% |\n",
            pct(n, total),
            at_least(t),
            pct(at_least(t), total)
        ));
    }
    md.push_str("\nChecks failed by compiled cards (a card can fail several):\n\n| Check | Cards failing | What it is |\n|---|---:|---|\n");
    for (c, what) in [
        (
            check::ROUND_TRIP,
            "the compiled abilities don't render back to the Oracle text",
        ),
        (check::MANUAL, "a hand-written ability fails `manual-check`"),
        (check::STRUCTURE, "an ability structure no test exercises"),
        (check::FUZZ_SWEPT, "not in the every-card report"),
        (check::FUZZ_CAST, "the sweep never cast or played it"),
        (
            check::FUZZ_CLEAN,
            "a panic, hang, slow game or rules violation in its games, or in another game whose failure names it",
        ),
        (
            check::LISTED,
            "not in `docs/verified-cards.tsv` (newer card data: regenerate)",
        ),
    ] {
        let n = fails.get(c).copied().unwrap_or(0);
        if n > 0 || c != check::LISTED {
            md.push_str(&format!("| {c} | {n} | {what} |\n"));
        }
    }
    md.push_str("\n| Format | Cards | Verified | % | Round-trip or better | % | Compiled (fully supported) | % |\n|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for (f, (t, at)) in &by_format {
        let (v, r, c) = (
            at[Tier::Verified as usize],
            at[Tier::RoundTrip as usize],
            at[Tier::Compiled as usize],
        );
        md.push_str(&format!(
            "| {f} | {t} | {v} | {:.1}% | {r} | {:.1}% | {c} | {:.1}% |\n",
            pct(v, *t),
            pct(r, *t),
            pct(c, *t)
        ));
    }
    md.push_str(&format!(
        "\n## Compiled cards\n\n- Fully supported (the compiled tier and above): **{full}** ({:.1}%): compiled {}, with hand-written abilities {n_manual}\n- Abilities supported: {supported_abilities}/{total_abilities} ({:.1}%)\n- Deferred, not counted: {sticker_sheets} sticker sheets (see `docs/DEFERRED.md`)\n\n",
        pct(full, total),
        full - n_manual,
        pct(supported_abilities, total_abilities)
    ));
    let tiers_text = std::fs::read_to_string(repo_root().join(TIERS_FILE)).unwrap_or_default();
    let swept: Vec<&str> = tiers_text
        .lines()
        .filter_map(|l| l.strip_prefix(crate::verified::SWEPT))
        .collect();
    if !swept.is_empty() {
        md.push_str("## Sweep results\n\nThe tier file's every-card results come from these sweeps (a carried-over result is from a sweep of the code as it was then):\n\n");
        for w in &swept {
            md.push_str(&format!("- {w}\n"));
        }
        md.push('\n');
    }
    md.push_str("## Regenerating\n\nRegenerate the tier file and this report before each milestone, and after a new card set or a CR update (other work changes round trips and structure coverage all the time, so the numbers drift between regenerations; a card newer than the tier file counts as at most compiled, and a card that stops compiling or round-tripping drops at once). Run nothing else heavy alongside the sweep:\n\n");
    md.push_str(TIER_RECIPE);
    md.push_str("\nEach chunk is a separate command because one run of the whole sweep takes longer than a command may run here (a pool larger than 30,300 cards needs a fourth chunk). `verified` reads the concatenated reports' `outcome` lines (whether each card was cast, and its failed games) and `failure` lines (a card a failed game's message names as an object, like a rules-check violation's, isn't fuzz-clean either).\n");
    md.push_str(
        "\n`mtg-tools verified --card \"Name\"` prints one card's tier and failed checks.\n",
    );
    md.push_str("\nWhen only tests changed since the last sweep (tests can't change what the sweep finds), skip the sweep and carry each card's sweep results over from the committed tier file: regenerate the structure log, then run `verified --logs target/structlog --fuzz-from docs/verified-cards.tsv --write` and `card-coverage`. A card the old file listed as unsupported has no recorded sweep result and counts as not swept. `verified --gain 60` lists the unexercised structures with the highest verified gain (the cards whose only failed check is that structure).\n");
    md.push_str("\nTo sweep only some cards (say, the ones a change newly counts or newly compiles) and keep everyone else's results, sweep a list of names and merge it over the carried-over results; card indices (and repro commands) stay the full pool's, and a failed game in the new report also charges the carried-over cards it names:\n\n```sh\ncargo run --release -p mtg-sim -- --every-card --threads 3 --names target/names.txt --report target/every-card-partial.tsv\ncargo run --release -p mtg-tools -- verified --logs target/structlog --fuzz-from docs/verified-cards.tsv --report target/every-card-partial.tsv --write\ncargo run --release -p mtg-tools -- card-coverage --write docs/CARD_COVERAGE.md\n```\n");
    md.push_str("\n## Cards with hand-written abilities\n\nGenuinely unique abilities written directly in the ability AST (`crates/mtg-engine/src/cards/`), each tested in `crates/mtg-engine/tests/cards/m_<card>.rs`; see the policy in `cards/mod.rs`.\n\n| Card | Ability | Why hand-written |\n|---|---|---|\n");
    let mut entries: Vec<&mtg_engine::cards::ManualAbility> = mtg_engine::cards::all().collect();
    entries.sort_by(|a, b| (a.card, a.face, a.text).cmp(&(b.card, b.face, b.text)));
    for m in entries {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            m.card,
            m.text.replace('|', "\\|").replace('\n', " / "),
            m.reason.replace('|', "\\|")
        ));
    }
    md.push_str("\n## Most common unsupported ability texts\n\n| Count | Text |\n|---:|---|\n");
    for (p, n) in pats.iter().take(200) {
        md.push_str(&format!(
            "| {n} | {} |\n",
            p.replace('|', "\\|").replace('\n', " / ")
        ));
    }
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--write" {
            std::fs::write(repo_root().join(&args[i + 1]), &md).expect("write report");
            println!("wrote {}", args[i + 1]);
            i += 1;
        }
        i += 1;
    }
}

fn unsupported(args: &[String]) {
    use mtg_engine::card::CardDef;
    let mut limit = 100usize;
    let mut filter: Option<String> = None;
    let mut card: Option<String> = None;
    let mut paper = false;
    let mut single = false;
    let mut raw = false;
    let mut by_card = false;
    let mut text: Option<String> = None;
    let mut type_line = "Sorcery".to_string();
    let mut name = "Probe".to_string();
    let mut ast = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--text" => {
                text = Some(args[i + 1].clone());
                i += 1;
            }
            "--type" => {
                type_line = args[i + 1].clone();
                i += 1;
            }
            "--name" => {
                name = args[i + 1].clone();
                i += 1;
            }
            "--ast" => ast = true,
            "--paper" => paper = true,
            "--by-card" => by_card = true,
            "--single" => single = true,
            "--raw" => raw = true,
            "--limit" => {
                limit = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--filter" => {
                filter = Some(args[i + 1].to_lowercase());
                i += 1;
            }
            "--card" => {
                card = Some(args[i + 1].to_lowercase());
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    // `--text "ORACLE" [--type "TYPE LINE"] [--name NAME] [--ast]`: compiles one text (a
    // probe for grammar work) and prints the blocks it doesn't support.
    if let Some(text) = text {
        let tl = mtg_engine::types::TypeLine::parse(&type_line);
        let ctx = mtg_engine::oracle::CompileContext {
            card_name: &name,
            full_name: &name,
            type_line: &tl,
            layout: mtg_engine::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: Some("1"),
            toughness: Some("1"),
        };
        let c = mtg_engine::oracle::compile(&text, &ctx);
        if ast {
            println!("{:#?}", c.abilities);
        }
        if c.unsupported.is_empty() {
            println!("OK");
        }
        for u in c.unsupported {
            println!("UNSUPPORTED\t{u}");
        }
        return;
    }
    let mut patterns: HashMap<String, (usize, String)> = HashMap::new();
    for c in mtg_data::cards().iter() {
        if !c.is_playable_card() || !c.is_legal_somewhere() {
            continue;
        }
        // Sticker sheets are deferred (docs/DEFERRED.md), unless asked for by name.
        if c.is_sticker_sheet() && card.is_none() {
            continue;
        }
        if card.as_ref().is_some_and(|n| c.name.to_lowercase() != *n) {
            continue;
        }
        if paper && !c.is_paper() {
            continue;
        }
        let def = CardDef::from_scryfall(c);
        let texts = def.unsupported_text();
        if single && texts.len() != 1 {
            continue;
        }
        for u in texts {
            if filter
                .as_ref()
                .is_some_and(|f| !u.to_lowercase().contains(f))
            {
                continue;
            }
            let key = if raw { u.to_string() } else { pattern_of(u) };
            if by_card {
                // One line per (card, text): the card-to-text map the planning scripts use.
                println!("{}\t{}", c.name, key.replace('\n', " / "));
                continue;
            }
            let e = patterns.entry(key).or_insert((0, c.name.clone()));
            e.0 += 1;
        }
    }
    if by_card {
        return;
    }
    let mut pats: Vec<(String, (usize, String))> = patterns.into_iter().collect();
    pats.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then(a.0.cmp(&b.0)));
    for (p, (n, example)) in pats.into_iter().take(limit) {
        println!("{n}\t{}\t[{example}]", p.replace('\n', " / "));
    }
}

/// All `ruling!("Card", "text")` citations.
pub fn ruling_citations() -> Vec<(String, String, String)> {
    let mut files = Vec::new();
    // Only the engine and the API's tests (hidden information): mtg-data and mtg-tools
    // mention the macro in docs and help text.
    rust_files(&repo_root().join("crates/mtg-engine"), &mut files);
    rust_files(&repo_root().join("crates/mtg-api/tests"), &mut files);
    let re =
        Regex::new(r#"(?s)ruling!\s*\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)"#)
            .unwrap();
    let mut out = Vec::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let rel = f
            .strip_prefix(repo_root())
            .unwrap_or(&f)
            .display()
            .to_string();
        for m in re.captures_iter(&text) {
            out.push((unescape(&m[1]), unescape(&m[2]), rel.clone()));
        }
    }
    out
}

/// The value of a Rust string literal's contents: `\"`, `\\`, `\n`, `\u{201c}`, and line
/// continuations, as the `ruling!` macro sees them.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('u') if it.peek() == Some(&'{') => {
                it.next();
                let hex: String = it.by_ref().take_while(|&c| c != '}').collect();
                if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(ch);
                }
            }
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\n') => {
                while it.peek().is_some_and(|c| c.is_whitespace()) {
                    it.next();
                }
            }
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

/// Rulings exempted from test coverage, from every `docs/rulings-exemptions/*.tsv`:
/// `Card Name<TAB>distinctive substring<TAB>reason` → (card, substring, reason, file).
pub fn rulings_exemptions() -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(repo_root().join("docs/rulings-exemptions")) else {
        return out;
    };
    let mut paths: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "tsv"))
        .collect();
    paths.sort();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file = path
            .strip_prefix(repo_root())
            .unwrap_or(&path)
            .display()
            .to_string();
        for line in text.lines() {
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let mut parts = line.splitn(3, '\t');
            let card = parts.next().unwrap_or("").trim().to_string();
            let needle = parts.next().unwrap_or("").trim().to_string();
            let reason = parts.next().unwrap_or("").trim().to_string();
            out.push((card, needle, reason, file.clone()));
        }
    }
    out
}

/// Resolves `(card, needle)` references against the rulings: every matching
/// (oracle_id, comment) goes into `hit`; references matching nothing are returned.
fn match_ruling_refs<'a>(
    refs: impl Iterator<Item = (&'a str, &'a str, &'a str)>,
    hit: &mut BTreeSet<(String, String)>,
) -> Vec<String> {
    let rulings = mtg_data::rulings();
    let cards = mtg_data::cards();
    let mut unmatched = Vec::new();
    for (card, needle, file) in refs {
        let Some(c) = cards.by_name(card) else {
            unmatched.push(format!("{card}: unknown card ({file})"));
            continue;
        };
        let n = mtg_data::normalize_text(needle);
        let mut found = false;
        if !n.is_empty() {
            for r in rulings.for_oracle_id(&c.oracle_id) {
                if mtg_data::normalize_text(&r.comment).contains(&n) {
                    hit.insert((c.oracle_id.clone(), r.comment.clone()));
                    found = true;
                }
            }
        }
        if !found {
            unmatched.push(format!("{card}: {needle} ({file})"));
        }
    }
    unmatched
}

fn rulings_coverage(args: &[String]) {
    let cites = ruling_citations();
    let exempts = rulings_exemptions();
    let rulings = mtg_data::rulings();
    let cards = mtg_data::cards();
    let mut matched: BTreeSet<(String, String)> = BTreeSet::new(); // (oracle_id, comment)
    let unmatched = match_ruling_refs(
        cites
            .iter()
            .map(|(c, n, f)| (c.as_str(), n.as_str(), f.as_str())),
        &mut matched,
    );
    let mut exempted: BTreeSet<(String, String)> = BTreeSet::new();
    let unmatched_ex = match_ruling_refs(
        exempts
            .iter()
            .map(|(c, n, _, f)| (c.as_str(), n.as_str(), f.as_str())),
        &mut exempted,
    );
    let missing_reason: Vec<String> = exempts
        .iter()
        .filter(|(_, _, r, _)| r.is_empty())
        .map(|(c, n, _, f)| format!("{c}: {n} ({f})"))
        .collect();
    let unique_comments: BTreeSet<&str> = rulings.iter().map(|r| r.comment.as_str()).collect();
    let cited_comments: BTreeSet<&str> = matched.iter().map(|(_, c)| c.as_str()).collect();
    let exempt_comments: BTreeSet<&str> = exempted
        .iter()
        .map(|(_, c)| c.as_str())
        .filter(|c| !cited_comments.contains(c))
        .collect();
    // A ruling text shared by many cards counts as covered once for "unique" coverage.
    let covered_unique = unique_comments
        .iter()
        .filter(|c| cited_comments.contains(*c))
        .count();
    let exempt_unique = unique_comments
        .iter()
        .filter(|c| exempt_comments.contains(*c))
        .count();
    let pct = |n: usize| 100.0 * n as f64 / unique_comments.len() as f64;
    println!(
        "Rulings cited: {} (card, ruling) pairs; {} of {} unique ruling texts cited ({:.2}%), {} exempt ({:.2}%); {} citations and {} exemptions unmatched",
        matched.len(),
        covered_unique,
        unique_comments.len(),
        pct(covered_unique),
        exempt_unique,
        pct(exempt_unique),
        unmatched.len(),
        unmatched_ex.len()
    );
    for u in &unmatched {
        println!("  UNMATCHED citation {u}");
    }
    for u in &unmatched_ex {
        println!("  UNMATCHED exemption {u}");
    }
    for u in &missing_reason {
        println!("  EXEMPTION WITHOUT REASON {u}");
    }
    let status = |comment: &str, oracle_id: &str| -> &'static str {
        let key = (oracle_id.to_string(), comment.to_string());
        if matched.contains(&key) {
            "CITED    "
        } else if cited_comments.contains(comment) {
            "CITED*   "
        } else if exempted.contains(&key) || exempt_comments.contains(comment) {
            "EXEMPT   "
        } else {
            "UNCOVERED"
        }
    };
    let mut write = None;
    let mut check = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write" => {
                write = args.get(i + 1).cloned();
                i += 1;
            }
            "--check" => check = true,
            // Status of every ruling of one card.
            "--card" => {
                let name = args.get(i + 1).cloned().unwrap_or_default();
                i += 1;
                match cards.by_name(&name) {
                    None => println!("unknown card: {name}"),
                    Some(c) => {
                        println!(
                            "{} ({} rulings):",
                            c.name,
                            rulings.for_oracle_id(&c.oracle_id).len()
                        );
                        for r in rulings.for_oracle_id(&c.oracle_id) {
                            println!("  {} {}", status(&r.comment, &c.oracle_id), r.comment);
                        }
                    }
                }
            }
            // Status of every unique ruling text containing a substring.
            "--text" => {
                let needle = mtg_data::normalize_text(args.get(i + 1).map_or("", |s| s.as_str()));
                i += 1;
                let mut by_text: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
                for r in rulings.iter() {
                    if mtg_data::normalize_text(&r.comment).contains(&needle) {
                        by_text
                            .entry(r.comment.as_str())
                            .or_default()
                            .push(r.oracle_id.as_str());
                    }
                }
                for (text, ids) in &by_text {
                    let st = ids
                        .iter()
                        .map(|id| status(text, id))
                        .min_by_key(|s| {
                            if s.starts_with("CITED") {
                                0
                            } else if s.starts_with("EXEMPT") {
                                1
                            } else {
                                2
                            }
                        })
                        .unwrap_or("UNCOVERED");
                    let names: Vec<String> = ids
                        .iter()
                        .take(3)
                        .filter_map(|id| cards.by_oracle_id(id).map(|c| c.name.clone()))
                        .collect();
                    println!(
                        "  {st} [{} cards, e.g. {}] {text}",
                        ids.len(),
                        names.join("; ")
                    );
                }
            }
            _ => {}
        }
        i += 1;
    }
    let mut md = String::new();
    md.push_str("# Scryfall rulings coverage\n\nGenerated by `cargo run -p mtg-tools -- rulings-coverage --write docs/RULINGS_COVERAGE.md`. Tests cite rulings with `ruling!(\"Card Name\", \"distinctive text\")`, which fails if no such ruling exists. Rulings with no engine-testable content are listed with a reason in `docs/rulings-exemptions/*.tsv`.\n\n");
    md.push_str(&format!(
        "- Total rulings: {} ({} unique texts across {} cards)\n- (card, ruling) pairs cited by tests: **{}**\n- Unique ruling texts covered by tests: **{}** ({:.2}%)\n- Unique ruling texts exempt: **{}** ({:.2}%)\n\n",
        rulings.len(),
        unique_comments.len(),
        rulings.iter().map(|r| r.oracle_id.as_str()).collect::<BTreeSet<_>>().len(),
        matched.len(),
        covered_unique,
        pct(covered_unique),
        exempt_unique,
        pct(exempt_unique),
    ));
    let mut by_card: BTreeMap<String, usize> = BTreeMap::new();
    for (card, _, _) in &cites {
        *by_card.entry(card.clone()).or_default() += 1;
    }
    md.push_str(&format!(
        "## Cards with cited rulings ({})\n\n",
        by_card.len()
    ));
    for (c, n) in &by_card {
        md.push_str(&format!("- {c} ({n})\n"));
    }
    if let Some(path) = write {
        std::fs::write(repo_root().join(&path), &md).expect("write report");
        println!("wrote {path}");
    }
    if check && !(unmatched.is_empty() && unmatched_ex.is_empty() && missing_reason.is_empty()) {
        std::process::exit(1);
    }
}

/// Checks the hand-written card definitions (stale, now compilable, untested, duplicate);
/// exits with status 1 on a problem.
fn manual_check() {
    let dir = repo_root().join("crates/mtg-engine/tests/cards");
    let problems = mtg_engine::cards::manual_check(&dir);
    let n = mtg_engine::cards::all().count();
    for p in &problems {
        println!("{p}");
    }
    println!("{n} hand-written abilities, {} problems", problems.len());
    if !problems.is_empty() {
        std::process::exit(1);
    }
}

/// The interpretation register: problems, and the index `docs/INTERPRETATIONS.md`.
fn interpretations(args: &[String]) {
    use mtg_data::interpretations as reg;
    let root = repo_root();
    let mut write: Option<PathBuf> = None;
    let mut check = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write" => {
                let path = args.get(i + 1).filter(|a| !a.starts_with("--"));
                write = Some(match path {
                    Some(p) => {
                        i += 1;
                        PathBuf::from(p)
                    }
                    None => root.join("docs/INTERPRETATIONS.md"),
                });
            }
            "--check" => check = true,
            other => {
                eprintln!("interpretations: unknown argument {other:?}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let entries = reg::load(&root.join("docs/interpretations"));
    let problems = reg::check(&root);
    for p in &problems {
        println!("{p}");
    }
    let index = reg::render_index(&entries);
    let provisional = entries.iter().filter(|e| e.status == "provisional").count();
    println!(
        "{} entries ({provisional} provisional), {} problems",
        entries.len(),
        problems.len()
    );
    let mut stale = false;
    if let Some(path) = &write {
        std::fs::write(path, &index).expect("write index");
        println!("wrote {}", path.display());
    } else {
        let current =
            std::fs::read_to_string(root.join("docs/INTERPRETATIONS.md")).unwrap_or_default();
        if current != index {
            stale = true;
            println!(
                "docs/INTERPRETATIONS.md is out of date: run `mtg-tools interpretations --write`"
            );
        }
    }
    if check && (!problems.is_empty() || stale) {
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("cr-coverage") => cr_coverage(&args[1..]),
        Some("card-coverage") => card_coverage(&args[1..]),
        Some("rulings-coverage") => rulings_coverage(&args[1..]),
        Some("unsupported") => unsupported(&args[1..]),
        Some("roundtrip") => roundtrip::run(&args[1..]),
        Some("manual-check") => manual_check(),
        Some("interpretations") => interpretations(&args[1..]),
        Some("structure-coverage") => structure_coverage::run(&args[1..], repo_root()),
        Some("verified") => verified::run(&args[1..], repo_root()),
        _ => {
            eprintln!("usage: mtg-tools <cr-coverage|card-coverage|rulings-coverage|unsupported|structure-coverage|manual-check|interpretations|roundtrip|verified> [--write PATH] [--check]");
            std::process::exit(2);
        }
    }
}
