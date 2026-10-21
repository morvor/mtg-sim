//! "with exactly three colors", "that's exactly three colors" (Meeting of the Five,
//! Threefold Signal): an object that has exactly that many of the five colors (CR 105.2),
//! as one of the sets of that many colors. (Exactly two colors: see
//! `trigger_grammar_combat_cast.rs`.)

use super::FilterSuffixPattern;
use crate::ability::*;
use crate::types::ColorSet;

fn word_end(r: &str) -> bool {
    r.is_empty() || r.starts_with(' ') || r.starts_with(',')
}

/// Every set of exactly `n` of the five colors.
fn sets_of(n: u32) -> Vec<ColorSet> {
    (0u8..32)
        .map(ColorSet)
        .filter(|s| s.count() == n)
        .collect()
}

fn exactly_n_colors<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    for lead in ["with exactly ", "that's exactly ", "that are exactly "] {
        let Some(r) = t.strip_prefix(lead) else {
            continue;
        };
        for (w, n) in [("three", 3), ("four", 4)] {
            if let Some(r) = r
                .strip_prefix(w)
                .and_then(|r| r.strip_prefix(" colors"))
                .filter(|r| word_end(r))
            {
                let f = Filter::Or(sets_of(n).into_iter().map(Filter::ExactColors).collect());
                return Some((f, r));
            }
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "with exactly three colors", priority: 100, parse: exactly_n_colors } }
