//! The log of erroneous rulings, `docs/RULING_DISCREPANCIES.md`, and its check.
//!
//! The log records every Wizards ruling the engine doesn't follow in full, or tests on
//! another card than the one Scryfall's data attaches it to, so that a human can check each
//! one and report it to Wizards of the Coast. Each entry is a `### ` heading followed by
//! `- Field: value` lines (one line each):
//!
//! ```text
//! ### <Card>: <what is wrong>
//! - Kind: template error | misattached | inconsistent | outdated | narrower than stated
//! - Status: open for review | reported | confirmed
//! - Ruling, <Card Name> (<published_at>): "<the ruling's text, verbatim>"   (one or more)
//! - Conflicts with: <the Oracle text, CR rule or other rulings, quoted with dates>
//! - Evidence: <the likely origin, with dates>
//! - Engine: <what the engine does>
//! - Recorded: <exemption files, tests and register ids, each in backticks>
//! ```
//!
//! [`check`] (run by `cargo test -p mtg-data` and `mtg-tools rulings-coverage --check`)
//! keeps the log complete and exact:
//! - every rulings exemption (`docs/rulings-exemptions/*.tsv`) whose reason calls the ruling
//!   outdated, superseded, overridden, inconsistent, misattached or a template error, or
//!   cites the policy `ruling-contradicting-oracle-gives-way` ([`needs_log`]), has an entry
//!   quoting that ruling under that card;
//! - every entry has every field, a known kind and status, and at least one ruling; each
//!   quoted ruling is one of that card's rulings, word for word (compared after
//!   [`normalize_text`](crate::normalize_text)), with its `published_at` date;
//! - the files, tests and register ids an entry's `Recorded` field names exist: test
//!   functions named after a file (`` `file.rs` (`fn_a`, `fn_b`) ``) are defined in it, and
//!   an exemption file named there exempts one of the entry's rulings.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The log, relative to the repository root.
pub const LOG: &str = "docs/RULING_DISCREPANCIES.md";

/// The kinds of discrepancy.
pub const KINDS: &[&str] = &[
    "template error",
    "misattached",
    "inconsistent",
    "outdated",
    "narrower than stated",
];

/// The statuses of an entry.
pub const STATUSES: &[&str] = &["open for review", "reported", "confirmed"];

/// The fields every entry must have, besides its `Ruling` lines.
pub const FIELDS: &[&str] = &[
    "Kind",
    "Status",
    "Conflicts with",
    "Evidence",
    "Engine",
    "Recorded",
];

/// Words in a rulings exemption's reason that make it a discrepancy the log must record,
/// as word beginnings (ignoring case): "supersed" matches "superseded" and "supersedes",
/// "overrid" "overridden" and "overrides", "misattach" "misattached" and "misattachment".
pub const LOGGED_REASONS: &[&str] = &[
    "outdated",
    "supersed",
    "overrid",
    "inconsistent",
    "misattach",
    "template error",
    "ruling-contradicting-oracle-gives-way",
];

/// One quoted ruling of an entry: `- Ruling, <card> (<date>): "<text>"`.
#[derive(Clone, Debug)]
pub struct LoggedRuling {
    pub card: String,
    pub date: String,
    pub text: String,
    /// 1-based line in the log.
    pub line: usize,
}

/// One entry of the log.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub title: String,
    /// 1-based line of its heading.
    pub line: usize,
    /// The `- Field: value` lines other than the rulings, in order.
    pub fields: Vec<(String, String)>,
    pub rulings: Vec<LoggedRuling>,
}

impl LogEntry {
    /// The first value of a field.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Parses the log: entries start at `### ` headings, and the `- Field: value` and
/// `- Ruling, <card> (<date>): "<text>"` lines that follow belong to the entry. Returns the
/// entries and the malformed ruling lines (line, text).
pub fn parse(text: &str) -> (Vec<LogEntry>, Vec<(usize, String)>) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut entries: Vec<LogEntry> = Vec::new();
    let mut current: Option<LogEntry> = None;
    let mut malformed = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        if line.starts_with('#') {
            // Any heading ends the entry before it; a `### ` heading starts one.
            entries.extend(current.take());
            if let Some(h) = line.strip_prefix("### ") {
                current = Some(LogEntry {
                    title: h.trim().to_string(),
                    line: i + 1,
                    fields: Vec::new(),
                    rulings: Vec::new(),
                });
            }
            continue;
        }
        let Some(e) = current.as_mut() else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("- Ruling, ") {
            match parse_ruling(rest) {
                Some((card, date, text)) => e.rulings.push(LoggedRuling {
                    card,
                    date,
                    text,
                    line: i + 1,
                }),
                None => malformed.push((i + 1, line.to_string())),
            }
            continue;
        }
        let Some((name, value)) = line.strip_prefix("- ").and_then(|r| r.split_once(':')) else {
            continue;
        };
        if FIELDS.contains(&name.trim()) {
            e.fields
                .push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    entries.extend(current);
    (entries, malformed)
}

/// `<card> (<YYYY-MM-DD>): "<text>"` → (card, date, text).
fn parse_ruling(rest: &str) -> Option<(String, String, String)> {
    let (head, quoted) = rest.split_once("): \"")?;
    let (card, date) = head.rsplit_once(" (")?;
    let text = quoted.strip_suffix('"')?;
    let date_ok = date.len() == 10
        && date.chars().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    (date_ok && !card.trim().is_empty() && !text.is_empty())
        .then(|| (card.trim().to_string(), date.to_string(), text.to_string()))
}

/// One rulings exemption: `Card<TAB>needle<TAB>reason` in `docs/rulings-exemptions/*.tsv`.
#[derive(Clone, Debug)]
pub struct Exemption {
    pub card: String,
    pub needle: String,
    pub reason: String,
    /// The file, relative to the repository root.
    pub file: String,
    /// 1-based line.
    pub line: usize,
}

/// Every rulings exemption under `root/docs/rulings-exemptions` (files sorted by name).
pub fn exemptions(root: &Path) -> Vec<Exemption> {
    let dir = root.join("docs/rulings-exemptions");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "tsv"))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let file = format!(
            "docs/rulings-exemptions/{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let mut parts = line.splitn(3, '\t');
            out.push(Exemption {
                card: parts.next().unwrap_or("").trim().to_string(),
                needle: parts.next().unwrap_or("").trim().to_string(),
                reason: parts.next().unwrap_or("").trim().to_string(),
                file: file.clone(),
                line: i + 1,
            });
        }
    }
    out
}

/// Whether an exemption's reason makes it a discrepancy the log must record: a word in it
/// begins with one of [`LOGGED_REASONS`] (ignoring case).
pub fn needs_log(reason: &str) -> bool {
    let r = reason.to_lowercase();
    LOGGED_REASONS.iter().any(|w| starts_a_word(&r, w))
}

/// Whether `stem` occurs in `hay` at the start of a word.
fn starts_a_word(hay: &str, stem: &str) -> bool {
    hay.match_indices(stem).any(|(i, _)| {
        !hay[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '-')
    })
}

/// The text between each pair of backticks in `s`.
fn backticked(s: &str) -> Vec<&str> {
    s.split('`').skip(1).step_by(2).collect()
}

/// Whether two card names are the same card (the same Oracle id).
fn same_card(a: &str, b: &str) -> bool {
    let cards = crate::cards();
    match (cards.by_name(a), cards.by_name(b)) {
        (Some(x), Some(y)) => x.oracle_id == y.oracle_id,
        _ => false,
    }
}

/// A register id: a slug with at least one hyphen.
fn is_register_id(s: &str) -> bool {
    s.contains('-')
        && !s.starts_with('-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The test functions named in parentheses right after a backticked `.rs` file in `s`:
/// `` `file.rs` (`fn_a`, `fn_b`) `` → [("file.rs", ["fn_a", "fn_b"])].
fn named_tests(s: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('`') else { break };
        let token = &after[..end];
        rest = &after[end + 1..];
        if !token.ends_with(".rs") {
            continue;
        }
        let Some(inner) = rest.strip_prefix(" (") else {
            continue;
        };
        let Some(close) = inner.find(')') else {
            continue;
        };
        let fns: Vec<String> = backticked(&inner[..close])
            .into_iter()
            .map(str::to_string)
            .collect();
        out.push((token.to_string(), fns));
    }
    out
}

/// Where a path named in the log is: under the repository root, or (for `tests/...`)
/// under the engine crate.
fn resolve(root: &Path, p: &str) -> Option<PathBuf> {
    [root.join(p), root.join("crates/mtg-engine").join(p)]
        .into_iter()
        .find(|c| c.exists())
}

/// Checks the log against the rulings data, the rulings exemptions, the register and the
/// files it names. Returns the problems found.
pub fn check(root: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let log_path = root.join(LOG);
    let Ok(text) = std::fs::read_to_string(&log_path) else {
        return vec![format!("{LOG} is missing")];
    };
    let (entries, malformed) = parse(&text);
    for (line, l) in &malformed {
        problems.push(format!(
            "{LOG}:{line}: malformed ruling line (expected `- Ruling, <Card> (<YYYY-MM-DD>): \"<text>\"`): {l}"
        ));
    }
    if entries.is_empty() {
        problems.push(format!("{LOG} has no entries"));
    }
    let cards = crate::cards();
    let rulings = crate::rulings();
    let register: BTreeSet<String> =
        crate::interpretations::load(&root.join("docs/interpretations"))
            .into_iter()
            .map(|e| e.id)
            .collect();
    let all_exemptions = exemptions(root);
    // The entries' rulings: (oracle id, normalized text).
    let mut logged: Vec<(String, String)> = Vec::new();
    for e in &entries {
        let at = format!("{LOG}:{}", e.line);
        for f in FIELDS {
            if !e.field(f).is_some_and(|v| !v.is_empty()) {
                problems.push(format!("{at}: {:?} has no `- {f}:` line", e.title));
            }
        }
        if let Some(k) = e.field("Kind") {
            if !KINDS.contains(&k) {
                problems.push(format!(
                    "{at}: {:?} has kind {k:?} (expected one of {KINDS:?})",
                    e.title
                ));
            }
        }
        if let Some(s) = e.field("Status") {
            if !STATUSES.contains(&s) {
                problems.push(format!(
                    "{at}: {:?} has status {s:?} (expected one of {STATUSES:?})",
                    e.title
                ));
            }
        }
        if e.rulings.is_empty() {
            problems.push(format!("{at}: {:?} quotes no ruling", e.title));
        }
        for r in &e.rulings {
            let at = format!("{LOG}:{}", r.line);
            let Some(card) = cards.by_name(&r.card) else {
                problems.push(format!("{at}: unknown card {:?}", r.card));
                continue;
            };
            let n = crate::normalize_text(&r.text);
            let same: Vec<_> = rulings
                .for_oracle_id(&card.oracle_id)
                .into_iter()
                .filter(|x| crate::normalize_text(&x.comment) == n)
                .collect();
            if same.is_empty() {
                problems.push(format!(
                    "{at}: {:?} has no ruling with exactly this text (quote it verbatim): {:?}",
                    r.card, r.text
                ));
            } else if !same.iter().any(|x| x.published_at == r.date) {
                problems.push(format!(
                    "{at}: {:?}'s ruling is dated {}, not {}",
                    r.card, same[0].published_at, r.date
                ));
            }
            logged.push((card.oracle_id.clone(), n));
        }
        let recorded = e.field("Recorded").unwrap_or("");
        for token in backticked(recorded) {
            if token.contains('/') {
                if resolve(root, token).is_none() {
                    problems.push(format!(
                        "{at}: {:?} names a missing file {token:?}",
                        e.title
                    ));
                } else if token.starts_with("docs/rulings-exemptions/") {
                    // That file exempts (part of) one of the entry's rulings.
                    let exempts_one = all_exemptions.iter().any(|x| {
                        x.file == token
                            && e.rulings.iter().any(|r| {
                                same_card(&r.card, &x.card)
                                    && crate::normalize_text(&r.text)
                                        .contains(&crate::normalize_text(&x.needle))
                            })
                    });
                    if !exempts_one {
                        problems.push(format!(
                            "{at}: {:?} names {token} but no line there exempts one of its rulings",
                            e.title
                        ));
                    }
                }
            } else if is_register_id(token) && !register.contains(token) {
                problems.push(format!(
                    "{at}: {:?} names {token:?}, which is no register entry",
                    e.title
                ));
            }
        }
        for (file, fns) in named_tests(recorded) {
            let Some(path) = resolve(root, &file) else {
                continue;
            };
            let src = std::fs::read_to_string(path).unwrap_or_default();
            for f in fns {
                if !src.contains(&format!("fn {f}(")) {
                    problems.push(format!("{at}: {:?}: {file} has no test {f}", e.title));
                }
            }
        }
    }
    for x in &all_exemptions {
        if !needs_log(&x.reason) {
            continue;
        }
        let Some(card) = cards.by_name(&x.card) else {
            continue; // `mtg-tools rulings-coverage --check` reports unknown cards
        };
        let needle = crate::normalize_text(&x.needle);
        if !logged
            .iter()
            .any(|(id, t)| *id == card.oracle_id && t.contains(&needle))
        {
            problems.push(format!(
                "{}:{}: {} ({:?}): its reason makes it a ruling discrepancy, but {LOG} quotes no ruling of that card containing the needle (add an entry)",
                x.file, x.line, x.card, x.needle
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entries_rulings_and_fields() {
        let text = "# Log\n\n## Kind\n\n### Card: wrong\n- Kind: outdated\n- Status: reported\n- Ruling, Gut, True Soul Zealot (2022-06-10): \"Gut doesn't have to be among the attacking creatures.\"\n- Ruling, Bad line\n- Recorded: `tests/x.rs` (`a_test`, `b_test`); register `some-id`\n\n## Next\n- Kind: stray\n";
        let (entries, malformed) = parse(text);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.title, "Card: wrong");
        assert_eq!(e.line, 5);
        assert_eq!(e.field("Kind"), Some("outdated"));
        assert_eq!(e.field("Status"), Some("reported"));
        assert_eq!(e.rulings.len(), 1);
        assert_eq!(e.rulings[0].card, "Gut, True Soul Zealot");
        assert_eq!(e.rulings[0].date, "2022-06-10");
        assert_eq!(
            e.rulings[0].text,
            "Gut doesn't have to be among the attacking creatures."
        );
        assert_eq!(malformed.len(), 1);
        assert_eq!(
            named_tests(e.field("Recorded").unwrap()),
            vec![(
                "tests/x.rs".to_string(),
                vec!["a_test".to_string(), "b_test".to_string()]
            )]
        );
    }

    #[test]
    fn reasons_needing_a_log_entry() {
        assert!(needs_log("Outdated (conclusion B): ..."));
        assert!(needs_log("Inconsistent with the other rulings"));
        assert!(needs_log(
            "policy ruling-contradicting-oracle-gives-way, ..."
        ));
        assert!(needs_log("Contradicts the card (template error; ...)"));
        assert!(needs_log("the CR 605.1a update supersedes it"));
        assert!(needs_log("Misattachment in Scryfall data"));
        assert!(!needs_log("a non-outdated note"));
        assert!(!needs_log(
            "Oracle-wording history: an errata note, no game behavior"
        ));
        assert!(!needs_log("flavor joke; no game rules content"));
        assert!(is_register_id("history-counts-players-who-left"));
        assert!(!is_register_id("fn_name"));
        assert!(!is_register_id("rulings-P005.tsv"));
    }
}
