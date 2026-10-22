//! Data access for the MTG rules engine.
//!
//! * [`scryfall`] — Scryfall bulk-data "Oracle Cards" (one entry per Oracle ID).
//! * [`rulings`] — Scryfall bulk-data rulings, keyed by Oracle ID.
//! * [`tags`] — Scryfall oracle tags (functional card tags).
//! * [`interpretations`] — the interpretation register (`docs/interpretations/`).
//! * [`rules`] — the Magic: The Gathering Comprehensive Rules, parsed into
//!   numbered rules and glossary entries.
//!
//! All loaders read from the repository `data/` directory by default. Set the
//! `MTG_DATA_DIR` environment variable to point somewhere else.
//!
//! The global accessors ([`cards`], [`rulings`], [`comprehensive_rules`]) load lazily
//! and cache for the lifetime of the process.

pub mod interpretations;
pub mod rules;
pub mod rulings;
pub mod scryfall;
pub mod tags;

use std::path::PathBuf;
use std::sync::OnceLock;

/// Directory containing the bulk data files.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MTG_DATA_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"))
}

static CARDS: OnceLock<scryfall::CardDatabase> = OnceLock::new();
static RULINGS: OnceLock<rulings::RulingsDatabase> = OnceLock::new();
static RULES: OnceLock<rules::ComprehensiveRules> = OnceLock::new();

/// The Scryfall oracle card database (loaded once per process).
pub fn cards() -> &'static scryfall::CardDatabase {
    CARDS.get_or_init(|| {
        scryfall::CardDatabase::load(&data_dir().join("oracle_cards.jsonl.gz"))
            .expect("failed to load data/oracle_cards.jsonl.gz")
    })
}

/// The Scryfall rulings database (loaded once per process).
pub fn rulings() -> &'static rulings::RulingsDatabase {
    RULINGS.get_or_init(|| {
        rulings::RulingsDatabase::load(&data_dir().join("rulings.jsonl.gz"))
            .expect("failed to load data/rulings.jsonl.gz")
    })
}

/// The Comprehensive Rules (loaded once per process).
pub fn comprehensive_rules() -> &'static rules::ComprehensiveRules {
    RULES.get_or_init(|| {
        rules::ComprehensiveRules::load(&data_dir().join("comprehensive-rules.txt"))
            .expect("failed to load data/comprehensive-rules.txt")
    })
}

/// Normalizes a string for loose comparisons: lowercases, folds curly quotes and
/// dashes to ASCII, and collapses whitespace.
pub fn normalize_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for ch in s.chars() {
        let ch = match ch {
            '\u{2018}' | '\u{2019}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            c => c,
        };
        if ch.is_whitespace() {
            if !last_space && !out.is_empty() {
                out.push(' ');
            }
            last_space = true;
        } else {
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
            last_space = false;
        }
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Asserts that the Comprehensive Rules contain the given rule number.
/// Used by the `cr!` citation macro in tests so rule citations cannot drift from
/// the real document.
pub fn assert_rule_exists(id: &str) {
    let rules = comprehensive_rules();
    assert!(
        rules.get(id).is_some(),
        "cited Comprehensive Rule {id:?} does not exist in the loaded rules (effective {})",
        rules.effective_date
    );
}

/// Asserts that the Comprehensive Rules contain rule `id` and that one of its worked
/// examples (the paragraphs starting "Example:" that follow the rule) contains `needle`
/// (compared after [`normalize_text`]). Returns the full example text. Used by the
/// `cr_example!` citation macro.
pub fn assert_rule_example_exists(id: &str, needle: &str) -> String {
    assert_rule_exists(id);
    let rules = comprehensive_rules();
    let rule = rules.get(id).expect("rule exists");
    let n = normalize_text(needle);
    assert!(
        !n.is_empty(),
        "cr_example! citation of {id:?}: the substring must not be empty"
    );
    let hits: Vec<&String> = rule
        .examples
        .iter()
        .filter(|e| normalize_text(e).contains(&n))
        .collect();
    match hits.as_slice() {
        [one] => (*one).clone(),
        [] => panic!(
            "cr_example! citation: no example of rule {id:?} contains {needle:?}.\nExamples of {id}:\n{}",
            if rule.examples.is_empty() {
                "  (none)".to_string()
            } else {
                rule.examples
                    .iter()
                    .map(|e| format!("  - {e}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        ),
        _ => panic!(
            "cr_example! citation: {needle:?} matches {} examples of rule {id:?}; use a more distinctive substring",
            hits.len()
        ),
    }
}

/// Asserts that the named card has a Scryfall ruling containing `needle`
/// (compared after [`normalize_text`]). Returns the full ruling text.
pub fn assert_ruling_exists(card_name: &str, needle: &str) -> String {
    let card = cards()
        .by_name(card_name)
        .unwrap_or_else(|| panic!("ruling citation: unknown card {card_name:?}"));
    let norm_needle = normalize_text(needle);
    let found = rulings()
        .for_oracle_id(&card.oracle_id)
        .into_iter()
        .find(|r| normalize_text(&r.comment).contains(&norm_needle));
    match found {
        Some(r) => r.comment.clone(),
        None => panic!(
            "ruling citation: no ruling for {card_name:?} contains {needle:?}.\nAvailable rulings:\n{}",
            rulings()
                .for_oracle_id(&card.oracle_id)
                .iter()
                .map(|r| format!("  - {}", r.comment))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

/// Cite one or more Comprehensive Rules from a test. Panics (failing the test) if a
/// cited rule number doesn't exist. The `mtg-tools cr-coverage` report scans for
/// these citations to compute rule coverage.
///
/// ```ignore
/// cr!("704.5a", "704.5b");
/// ```
#[macro_export]
macro_rules! cr {
    ($($id:literal),+ $(,)?) => {
        $( $crate::assert_rule_exists($id); )+
    };
}

/// Cite a Scryfall ruling from a test: `ruling!("Card Name", "distinctive substring")`.
/// Panics if the card has no ruling containing the substring, so ruling citations are
/// verified against the real bulk data.
#[macro_export]
macro_rules! ruling {
    ($card:literal, $needle:literal $(,)?) => {
        $crate::assert_ruling_exists($card, $needle)
    };
}

/// Cite an interpretation-register entry from a test that pins it:
/// `interp!("changeling-cda-only")`. Panics (failing the test) unless
/// `docs/interpretations/` has an entry with that id; `cargo test -p mtg-data` checks that
/// every entry is cited by some test. See [`interpretations`].
#[macro_export]
macro_rules! interp {
    ($($id:literal),+ $(,)?) => {
        $( $crate::interpretations::assert_interpretation_exists($id); )+
    };
}

/// Cite one worked example of a Comprehensive Rule from a test:
/// `cr_example!("613.10", "distinctive substring")`. Panics (failing the test) unless the
/// rule exists and exactly one of its "Example:" paragraphs contains the substring. Like
/// `ruling!`, it cites by substring so citations survive the CR adding or reordering
/// examples. It also counts as citing the rule itself (as `cr!` does) in
/// `mtg-tools cr-coverage`, which reports example coverage (`--examples`).
#[macro_export]
macro_rules! cr_example {
    ($id:literal, $needle:literal $(,)?) => {
        $crate::assert_rule_example_exists($id, $needle)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn cr_example_citation_matches_one_example() {
        let text = crate::cr_example!("120.5", "neither lightning bolt nor the damage");
        assert!(text.starts_with("Example: A player casts Lightning Bolt"));
        crate::assert_rule_exists("120.5");
    }

    #[test]
    #[should_panic(expected = "no example of rule")]
    fn cr_example_citation_rejects_unknown_substring() {
        crate::cr_example!("120.5", "no such example text");
    }

    #[test]
    #[should_panic(expected = "matches 3 examples")]
    fn cr_example_citation_rejects_ambiguous_substring() {
        crate::cr_example!("107.1b", "example:");
    }
}
