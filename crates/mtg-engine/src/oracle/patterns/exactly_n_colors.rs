//! Object qualities (cards3-cast-zones):
//!
//! * "with exactly three colors", "that's exactly three colors" (Meeting of the Five,
//!   Threefold Signal): an object that has exactly that many of the five colors
//!   (CR 105.2), as one of the sets of that many colors (exactly two colors: see
//!   `trigger_grammar_combat_cast.rs`);
//! * "with mana value less than or equal to that Aura", "with a different name than each
//!   Aura you control" (Light-Paws, Emperor's Voice).

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

/// "with mana value less than or equal to that Aura" (Light-Paws, Emperor's Voice): that
/// object's mana value (the referent, resolved by `filters_relational`).
fn mana_value_at_most_that<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    use super::filters_relational::{stat_filter, stat_of, Stat, REFERENT_THAT};
    let r = t
        .strip_prefix("with mana value less than or equal to that ")
        .or_else(|| t.strip_prefix("and with mana value less than or equal to that "))?;
    let noun = r.split([' ', ',']).next()?;
    if crate::oracle::phrases::head_noun(noun).is_none() {
        return None;
    }
    let tail = &r[noun.len()..];
    let f = stat_filter(
        Stat::ManaValue,
        Cmp::Le,
        stat_of(Stat::ManaValue, Sel::Var(REFERENT_THAT)),
    );
    Some((f, tail))
}

/// "with a different name than each Aura you control" (Light-Paws, Emperor's Voice):
/// unlike the name of every such object (CR 201.2).
fn different_name_than_each<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t
        .strip_prefix("with a different name than each ")
        .or_else(|| t.strip_prefix("and with a different name than each "))?;
    let (f, _, rest) = crate::oracle::phrases::parse_object_phrase(r)?;
    Some((Filter::DifferentNameFrom(Box::new(Sel::All(f))), rest))
}

inventory::submit! { FilterSuffixPattern { name: "with mana value less than or equal to that [object]", priority: 100, parse: mana_value_at_most_that } }
inventory::submit! { FilterSuffixPattern { name: "with a different name than each [object]", priority: 100, parse: different_name_than_each } }
