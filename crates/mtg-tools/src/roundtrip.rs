//! `roundtrip [--card NAME] [--filter TEXT] [--limit N] [--write PATH] [--check] [--ast]
//! [--list] [--clusters N]`: renders every fully supported card's compiled abilities back
//! to Oracle-style text ([`mtg_engine::oracle::render`]) and compares it with the card's
//! Oracle text.
//!
//! * `--card NAME`: only that card (prints both sides).
//! * `--filter TEXT`: only cards whose Oracle text contains TEXT (case-insensitive).
//! * `--limit N`: with `--list`, print at most N mismatching cards (default 20).
//! * `--list`: print mismatching cards with both sides.
//! * `--write PATH`: write the Markdown report (and `docs/roundtrip-passing.txt`, the
//!   checked-in list of passing cards that must stay passing).
//! * `--check`: exit 1 if a card in `docs/roundtrip-passing.txt` no longer passes.
//! * `--ast`: print the compiled abilities' AST of `--card`.
//! * `--clusters N`: print the N largest mismatch clusters.

use mtg_engine::card::CardDef;
use mtg_engine::oracle::render::compare::{check_card, normalize_unit, CardCheck};
use std::collections::{BTreeMap, BTreeSet};

pub const PASSING_LIST: &str = "docs/roundtrip-passing.txt";

/// Cards the coverage reports count: playable paper cards legal somewhere.
pub fn counted(c: &mtg_data::scryfall::ScryfallCard) -> bool {
    c.is_playable_card()
        && c.is_legal_somewhere()
        && !(c.games.iter().all(|g| g != "paper") && !c.games.is_empty())
}

/// A short description of how a mismatching card differs, used to group mismatches.
pub fn cluster_key(r: &CardCheck) -> String {
    if let Some(g) = r.gaps.first() {
        let g = g.split(['(', '{', ':']).next().unwrap_or(g).trim();
        return format!("gap: {g}");
    }
    let o: Vec<String> = r
        .unmatched_oracle
        .iter()
        .flat_map(|u| normalize_unit(u))
        .collect();
    let m: Vec<String> = r
        .unmatched_rendered
        .iter()
        .flat_map(|u| normalize_unit(u))
        .collect();
    if o.is_empty() {
        return "extra rendered ability".into();
    }
    if m.is_empty() {
        return "missing rendered ability".into();
    }
    let (a, b) = first_diff(&o, &m);
    let a = if a.is_empty() {
        "∅".to_string()
    } else {
        a.join(" ")
    };
    let b = if b.is_empty() {
        "∅".to_string()
    } else {
        b.join(" ")
    };
    format!("oracle `{a}` vs rendered `{b}`")
}

/// The first differing region of two token sequences (after their common prefix and
/// before their common suffix), each cut to a few tokens.
fn first_diff(a: &[String], b: &[String]) -> (Vec<String>, Vec<String>) {
    let p = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let (a2, b2) = (&a[p..], &b[p..]);
    let s = a2
        .iter()
        .rev()
        .zip(b2.iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let a3 = &a2[..a2.len() - s];
    let b3 = &b2[..b2.len() - s];
    let cut = |v: &[String]| v.iter().take(5).cloned().collect::<Vec<_>>();
    (cut(a3), cut(b3))
}

struct Options {
    card: Option<String>,
    filter: Option<String>,
    limit: usize,
    write: Option<String>,
    check: bool,
    ast: bool,
    clusters: usize,
    list: bool,
}

fn parse(args: &[String]) -> Options {
    let mut o = Options {
        card: None,
        filter: None,
        limit: 20,
        write: None,
        check: false,
        ast: false,
        clusters: 0,
        list: false,
    };
    let mut i = 0;
    while i < args.len() {
        let next = args.get(i + 1).cloned();
        match args[i].as_str() {
            "--card" => {
                o.card = next.map(|s| s.to_lowercase());
                i += 1;
            }
            "--filter" => {
                o.filter = next.map(|s| s.to_lowercase());
                i += 1;
            }
            "--limit" => {
                o.limit = next.and_then(|s| s.parse().ok()).unwrap_or(20);
                i += 1;
            }
            "--write" => {
                o.write = next;
                i += 1;
            }
            "--clusters" => {
                o.clusters = next.and_then(|s| s.parse().ok()).unwrap_or(30);
                i += 1;
            }
            "--check" => o.check = true,
            "--ast" => o.ast = true,
            "--list" => o.list = true,
            _ => {}
        }
        i += 1;
    }
    o
}

/// The round-trip results for every counted, fully supported card.
pub fn check_all(filter: impl Fn(&mtg_data::scryfall::ScryfallCard) -> bool) -> Vec<CardCheck> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for c in mtg_data::cards().iter() {
        if !counted(c) || !filter(c) || !seen.insert(c.name.clone()) {
            continue;
        }
        let def = CardDef::from_scryfall(c);
        if !def.is_fully_supported() {
            continue;
        }
        out.push(check_card(&def));
    }
    out
}

fn print_card(r: &CardCheck) {
    println!(
        "== {} — {}",
        r.name,
        if r.pass { "PASS" } else { "MISMATCH" }
    );
    for (i, (o, m)) in r.faces.iter().enumerate() {
        if r.faces.len() > 1 {
            println!("  -- face {i}");
        }
        for u in o {
            println!("  oracle:   {}", u.replace('\n', " / "));
        }
        for u in m {
            println!("  rendered: {}", u.replace('\n', " / "));
        }
    }
    for g in &r.gaps {
        println!("  gap: {g}");
    }
    if !r.pass {
        for u in &r.unmatched_oracle {
            println!("  - {}", normalize_unit(u).join(" "));
        }
        for u in &r.unmatched_rendered {
            println!("  + {}", normalize_unit(u).join(" "));
        }
    }
}

pub fn run(args: &[String]) {
    let o = parse(args);
    if o.ast {
        for c in mtg_data::cards().iter() {
            if o.card.as_ref().is_some_and(|n| c.name.to_lowercase() != *n) {
                continue;
            }
            let def = CardDef::from_scryfall(c);
            for f in &def.faces {
                println!("== {} ==\n{}", f.chars.name, f.chars.rules_text);
                for a in &f.chars.abilities {
                    println!("{:#?}", a.kind);
                }
            }
            break;
        }
        return;
    }
    let card = o.card.clone();
    let filter = o.filter.clone();
    let results = check_all(|c| {
        if card.as_ref().is_some_and(|n| c.name.to_lowercase() != *n) {
            return false;
        }
        if let Some(f) = &filter {
            let text = c
                .faces()
                .iter()
                .map(|f| f.oracle_text.clone().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();
            if !text.contains(f.as_str()) {
                return false;
            }
        }
        true
    });
    if o.card.is_some() {
        for r in &results {
            print_card(r);
        }
        if results.is_empty() {
            println!("no fully supported card by that name");
        }
        return;
    }
    let total = results.len();
    let passing: Vec<&CardCheck> = results.iter().filter(|r| r.pass).collect();
    let failing: Vec<&CardCheck> = results.iter().filter(|r| !r.pass).collect();
    let mut clusters: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for r in &failing {
        clusters.entry(cluster_key(r)).or_default().push(&r.name);
    }
    let mut cl: Vec<(&String, &Vec<&str>)> = clusters.iter().collect();
    cl.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    println!(
        "Round trip: {} of {total} fully supported cards pass ({:.1}%); {} mismatch",
        passing.len(),
        100.0 * passing.len() as f64 / total.max(1) as f64,
        failing.len()
    );
    if o.list {
        for r in failing.iter().take(o.limit) {
            print_card(r);
        }
    }
    if o.clusters > 0 {
        for (k, v) in cl.iter().take(o.clusters) {
            println!(
                "{:5}  {k}  [{}]",
                v.len(),
                v.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
            );
        }
    }
    if o.check {
        let path = super::repo_root().join(PASSING_LIST);
        let list = std::fs::read_to_string(&path).unwrap_or_default();
        let now: BTreeSet<&str> = passing.iter().map(|r| r.name.as_str()).collect();
        let checked: BTreeSet<&str> = results.iter().map(|r| r.name.as_str()).collect();
        let mut regressions = Vec::new();
        for name in list
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        {
            if !now.contains(name) && (o.filter.is_none() || checked.contains(name)) {
                regressions.push(name);
            }
        }
        if !regressions.is_empty() {
            println!("{} cards no longer pass the round trip:", regressions.len());
            for r in &regressions {
                println!("  {r}");
            }
            std::process::exit(1);
        }
        println!("all cards in {PASSING_LIST} still pass");
    }
    if let Some(path) = &o.write {
        let md = report(total, &passing, &failing, &cl);
        std::fs::write(super::repo_root().join(path), md).expect("write report");
        let mut names: Vec<&str> = passing.iter().map(|r| r.name.as_str()).collect();
        names.sort();
        let mut list = String::from(
            "# Cards whose compiled abilities render back to their Oracle text.\n# Generated by `mtg-tools roundtrip --write docs/ROUNDTRIP.md`; `roundtrip --check` and\n# the engine test `oracle_roundtrip` fail if one of them stops passing.\n",
        );
        for n in names {
            list.push_str(n);
            list.push('\n');
        }
        std::fs::write(super::repo_root().join(PASSING_LIST), list).expect("write list");
        println!("wrote {path} and {PASSING_LIST}");
    }
}

fn report(
    total: usize,
    passing: &[&CardCheck],
    failing: &[&CardCheck],
    clusters: &[(&String, &Vec<&str>)],
) -> String {
    use mtg_engine::oracle::render::compare::{EQUIVALENCES, IGNORED_WORDS};
    let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut md = String::new();
    md.push_str("# Oracle round trip\n\n");
    md.push_str("Generated by `cargo run --release -p mtg-tools -- roundtrip --write docs/ROUNDTRIP.md`.\n\n");
    md.push_str("Every fully supported card's compiled abilities are rendered back into Oracle-style text (`crates/mtg-engine/src/oracle/render/`, written independently of the parser) and compared with the card's Oracle text after the same normalization on both sides (reminder text, self-references, case, punctuation, number words, grammatical number). A card *passes* when every ability matches. `docs/roundtrip-passing.txt` lists the passing cards; they must stay passing (`roundtrip --check`, engine test `oracle_roundtrip`).\n\n");
    md.push_str(&format!(
        "- Fully supported cards checked: **{total}**\n- Round-trip verified: **{}** ({:.1}%)\n- Mismatches: **{}**\n\n",
        passing.len(),
        100.0 * passing.len() as f64 / total.max(1) as f64,
        failing.len()
    ));
    md.push_str("## Allowed equivalences\n\nApplied to both sides (`compare.rs`, `EQUIVALENCES`):\n\n| Pattern | Replacement | Why |\n|---|---|---|\n");
    for e in EQUIVALENCES {
        md.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            e.pattern.replace('|', "\\|"),
            e.replacement,
            squash(e.why).replace('|', "\\|")
        ));
    }
    md.push_str("\nIgnored words (`IGNORED_WORDS`):\n\n");
    for (w, why) in IGNORED_WORDS {
        md.push_str(&format!("- `{w}`: {}\n", squash(why)));
    }
    md.push_str("\n## Mismatch clusters\n\n| Cards | Difference | Examples |\n|---:|---|---|\n");
    for (k, v) in clusters.iter().take(200) {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            v.len(),
            k.replace('|', "\\|"),
            v.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
        ));
    }
    md.push_str("\n## Sample mismatches\n\n");
    for r in failing.iter().take(80) {
        md.push_str(&format!("### {}\n\n", r.name));
        for u in &r.unmatched_oracle {
            md.push_str(&format!("- Oracle: {}\n", u.replace('\n', " / ")));
        }
        for u in &r.unmatched_rendered {
            md.push_str(&format!("- Rendered: {}\n", u.replace('\n', " / ")));
        }
        for g in &r.gaps {
            md.push_str(&format!("- Gap: {g}\n"));
        }
        md.push('\n');
    }
    md
}
