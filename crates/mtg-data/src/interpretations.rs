//! The interpretation register: `docs/interpretations/<area>.md`.
//!
//! Every rules judgment call the engine makes (an interpretation of ambiguous or silent
//! rules, a known deviation from the Comprehensive Rules, or a policy for something the
//! rules leave to players or procedure) has an entry there:
//!
//! ```text
//! ## <id>: <one-line title>
//! - Kind: interpretation | deviation | policy
//! - Status: decided | provisional
//! - Rules: ...
//! - Question: ...
//! - Decision: ...
//! - Alternatives: ...
//! - Why: ...
//! - Affects: ...
//! - Code: ...
//! - Tests: ...
//! ```
//!
//! Tests that pin an entry cite it with [`interp!`](crate::interp), which fails unless
//! the entry exists. [`check`] (run by `cargo test -p mtg-data`) verifies that ids are
//! unique, that every citation names an entry and that every entry is cited by a test.
//! `mtg-tools interpretations --write` renders the index `docs/INTERPRETATIONS.md`
//! ([`render_index`]).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The fields every entry must have, in order.
pub const FIELDS: &[&str] = &[
    "Kind",
    "Status",
    "Rules",
    "Question",
    "Decision",
    "Alternatives",
    "Why",
    "Affects",
    "Code",
    "Tests",
];

/// The kinds of entry.
pub const KINDS: &[&str] = &["interpretation", "deviation", "policy"];

/// The statuses of an entry.
pub const STATUSES: &[&str] = &["decided", "provisional"];

/// One register entry.
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub title: String,
    /// The file's stem, e.g. `characteristics-layers`.
    pub area: String,
    pub kind: String,
    pub status: String,
    /// The `- Field:` values, by field name (first line only for multi-line values).
    pub fields: BTreeMap<String, String>,
    /// The file the entry is in, and its line (1-based).
    pub file: PathBuf,
    pub line: usize,
}

/// The repository root (`crates/mtg-data/../..`).
pub fn repo_root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// The register directory, `docs/interpretations`.
pub fn register_dir() -> PathBuf {
    repo_root().join("docs/interpretations")
}

/// Parses one register file. Entries start at `## <id>: <title>` headings; the
/// `- Field: value` bullets that follow belong to the entry.
pub fn parse_file(path: &Path, text: &str) -> Vec<Entry> {
    let area = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut out: Vec<Entry> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if let Some(h) = line.strip_prefix("## ") {
            let (id, title) = match h.split_once(':') {
                Some((id, title)) => (id.trim(), title.trim()),
                None => (h.trim(), ""),
            };
            out.push(Entry {
                id: id.to_string(),
                title: title.to_string(),
                area: area.clone(),
                kind: String::new(),
                status: String::new(),
                fields: BTreeMap::new(),
                file: path.to_path_buf(),
                line: i + 1,
            });
            continue;
        }
        let Some(e) = out.last_mut() else { continue };
        let Some(rest) = line.strip_prefix("- ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if !FIELDS.contains(&name) || e.fields.contains_key(name) {
            continue;
        }
        let value = value.trim().to_string();
        match name {
            "Kind" => e.kind = value.to_lowercase(),
            "Status" => e.status = value.to_lowercase(),
            _ => {}
        }
        e.fields.insert(name.to_string(), value);
    }
    out
}

/// Loads every entry under `dir` (every `*.md` file, sorted by name).
pub fn load(dir: &Path) -> Vec<Entry> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "md"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
        .iter()
        .flat_map(|p| {
            let text = std::fs::read_to_string(p).unwrap_or_default();
            parse_file(p, &text)
        })
        .collect()
}

static REGISTER: OnceLock<Vec<Entry>> = OnceLock::new();

/// The register (loaded once per process).
pub fn register() -> &'static [Entry] {
    REGISTER.get_or_init(|| load(&register_dir()))
}

/// Asserts that the register has an entry `id`. Used by the [`interp!`](crate::interp)
/// citation macro.
pub fn assert_interpretation_exists(id: &str) {
    assert!(
        register().iter().any(|e| e.id == id),
        "cited interpretation {id:?} has no entry under docs/interpretations/ \
         (add `## {id}: <title>` to the area's file)"
    );
}

/// Every `interp!("<id>")` citation in the `.rs` files under `dirs` (recursively):
/// (id, file, line).
pub fn citations(dirs: &[PathBuf]) -> Vec<(String, PathBuf, usize)> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, files);
            } else if p.extension().is_some_and(|x| x == "rs") {
                files.push(p);
            }
        }
    }
    let mut files = Vec::new();
    for d in dirs {
        walk(d, &mut files);
    }
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            let mut rest = line;
            while let Some(pos) = rest.find("interp!(\"") {
                rest = &rest[pos + "interp!(\"".len()..];
                if let Some(end) = rest.find('"') {
                    out.push((rest[..end].to_string(), f.clone(), i + 1));
                    rest = &rest[end..];
                }
            }
        }
    }
    out
}

/// The test directories of every crate (`crates/*/tests`).
pub fn test_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(root.join("crates"))
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path().join("tests"))
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

/// Checks the register against the tests under `root`: every entry has every field, a
/// known kind and status, and a unique id that is a slug; every `interp!` citation names
/// an entry; every entry is cited by at least one test. Returns the problems found.
pub fn check(root: &Path) -> Vec<String> {
    let entries = load(&root.join("docs/interpretations"));
    let cites = citations(&test_dirs(root));
    let mut problems = Vec::new();
    if entries.is_empty() {
        problems.push("no entries under docs/interpretations/".to_string());
    }
    let mut seen: BTreeMap<&str, &Entry> = BTreeMap::new();
    for e in &entries {
        let at = format!("{}:{}", e.file.display(), e.line);
        if let Some(prev) = seen.insert(&e.id, e) {
            problems.push(format!(
                "{at}: duplicate id {:?} (also {}:{})",
                e.id,
                prev.file.display(),
                prev.line
            ));
        }
        let slug = !e.id.is_empty()
            && e.id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !slug {
            problems.push(format!("{at}: id {:?} is not a slug (a-z, 0-9, -)", e.id));
        }
        if e.title.is_empty() {
            problems.push(format!("{at}: {:?} has no title", e.id));
        }
        for f in FIELDS {
            if !e.fields.get(*f).is_some_and(|v| !v.is_empty()) {
                problems.push(format!("{at}: {:?} has no `- {f}:` line", e.id));
            }
        }
        if !KINDS.contains(&e.kind.as_str()) {
            problems.push(format!(
                "{at}: {:?} has kind {:?} (expected one of {KINDS:?})",
                e.id, e.kind
            ));
        }
        if !STATUSES.contains(&e.status.as_str()) {
            problems.push(format!(
                "{at}: {:?} has status {:?} (expected one of {STATUSES:?})",
                e.id, e.status
            ));
        }
    }
    let cited: BTreeSet<&str> = cites.iter().map(|(id, _, _)| id.as_str()).collect();
    for (id, f, line) in &cites {
        if !seen.contains_key(id.as_str()) {
            problems.push(format!(
                "{}:{line}: interp!({id:?}) names no register entry",
                f.display()
            ));
        }
    }
    for e in &entries {
        if !cited.contains(e.id.as_str()) {
            problems.push(format!(
                "{}:{}: {:?} is cited by no test (add interp!({:?}) to the tests that pin it)",
                e.file.display(),
                e.line,
                e.id,
                e.id
            ));
        }
    }
    problems
}

/// Renders `docs/INTERPRETATIONS.md`: counts, the provisional entries first (with their
/// question and decision), then an index of every entry.
pub fn render_index(entries: &[Entry]) -> String {
    let mut s = String::new();
    s.push_str("# Interpretation register\n\n");
    s.push_str(
        "Generated by `cargo run --release -p mtg-tools -- interpretations --write` from \
         `docs/interpretations/*.md`; don't edit by hand.\n\n",
    );
    s.push_str(
        "Every rules judgment call the engine makes has an entry: an **interpretation** (the \
         rules are ambiguous or silent and we chose a reading), a **deviation** (the engine \
         knowingly differs from the Comprehensive Rules) or a **policy** (the rules leave it \
         to a player or to procedure and the engine decides). **Provisional** entries are \
         calls a reasonable judge could make the other way; they await review. Tests that pin \
         an entry cite it with `interp!(\"<id>\")`. What the engine doesn't support at all \
         is a scope decision, listed in `docs/DEFERRED.md` instead.\n\n",
    );
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for e in entries {
        *counts.entry((&e.kind, &e.status)).or_default() += 1;
    }
    s.push_str(&format!("{} entries.\n\n", entries.len()));
    s.push_str("| Kind | Decided | Provisional |\n|---|---:|---:|\n");
    for k in KINDS {
        s.push_str(&format!(
            "| {k} | {} | {} |\n",
            counts.get(&(k, "decided")).copied().unwrap_or(0),
            counts.get(&(k, "provisional")).copied().unwrap_or(0)
        ));
    }
    s.push_str("\n## Provisional entries\n\n");
    let prov: Vec<&Entry> = entries
        .iter()
        .filter(|e| e.status == "provisional")
        .collect();
    if prov.is_empty() {
        s.push_str("None.\n");
    }
    for e in prov {
        s.push_str(&format!(
            "- **{}** ({}, {}): {}\n  - Question: {}\n  - Decision: {}\n",
            e.id,
            e.kind,
            link(e),
            e.title,
            e.fields.get("Question").map(String::as_str).unwrap_or(""),
            e.fields.get("Decision").map(String::as_str).unwrap_or("")
        ));
    }
    s.push_str(
        "\n## All entries\n\n| Id | Title | Kind | Status | Area |\n|---|---|---|---|---|\n",
    );
    let mut all: Vec<&Entry> = entries.iter().collect();
    all.sort_by(|a, b| (&a.area, &a.id).cmp(&(&b.area, &b.id)));
    for e in all {
        s.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            e.id,
            e.title.replace('|', "\\|"),
            e.kind,
            e.status,
            link(e)
        ));
    }
    s
}

fn link(e: &Entry) -> String {
    format!("[{}](interpretations/{}.md)", e.area, e.area)
}
