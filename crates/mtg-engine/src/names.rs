//! Names (CR 201): comparing the names of groups of objects (CR 201.2b, 201.2c) and
//! names "originally printed in" an expansion (CR 206.3).
//!
//! An object's own names are given by [`Characteristics::names`] (a split card has two,
//! CR 709.4a; interchangeable names count, CR 201.3a); [`Characteristics::has_name`] and
//! [`Characteristics::shares_name_with`] implement "has the same name" (CR 201.2a).

use crate::object::Characteristics;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// The largest number of the given objects that have different names (CR 201.2b): each
/// has at least one name and no two of them have a name in common. Objects with no name
/// (e.g. face-down permanents) never count.
pub fn distinct_name_count<'a>(objs: impl IntoIterator<Item = &'a Characteristics>) -> usize {
    let mut named: Vec<&Characteristics> = objs.into_iter().filter(|c| c.has_a_name()).collect();
    // Objects with fewer names are less likely to clash with others: pick them first.
    // (An object with the names of all nonlegendary creature cards clashes with most.)
    named.sort_by_key(|c| (c.all_creature_names, c.names().count()));
    let mut picked: Vec<&Characteristics> = Vec::new();
    for c in named {
        if picked.iter().all(|p| !p.shares_name_with(c)) {
            picked.push(c);
        }
    }
    picked.len()
}

/// Whether `first` has a different name than each of `others` (CR 201.2c): it has at
/// least one name and no names in common with any of them, even if some of them have no
/// name. An object with no name doesn't have a different name than anything.
pub fn has_different_name<'a>(
    first: &Characteristics,
    others: impl IntoIterator<Item = &'a Characteristics>,
) -> bool {
    first.has_a_name() && others.into_iter().all(|o| !first.shares_name_with(o))
}

/// Normalizes a card name for comparison with the lists printed in the Comprehensive
/// Rules (which use typographic apostrophes).
fn norm(n: &str) -> String {
    n.trim().replace('\u{2019}', "'").to_lowercase()
}

/// The names the Comprehensive Rules list as "originally printed in" each expansion
/// (CR 206.3a-c), keyed by the expansion's name ("arabian nights", "antiquities", ...).
fn originally_printed_lists() -> &'static HashMap<String, HashSet<String>> {
    static LISTS: OnceLock<HashMap<String, HashSet<String>>> = OnceLock::new();
    LISTS.get_or_init(|| {
        let cr = mtg_data::comprehensive_rules();
        let mut out = HashMap::new();
        let Some(parent) = cr.get("206.3") else {
            return out;
        };
        for id in &parent.children {
            let Some(rule) = cr.get(id) else { continue };
            let text = rule.text.replace('™', "");
            // "... with a name originally printed in the Arabian Nights expansion. Those
            // names are Abu Ja'far, Aladdin, ..., and Ydwen Efreet."
            let Some(set) = text
                .split_once("originally printed in the ")
                .and_then(|(_, r)| r.split_once(" expansion"))
                .map(|(s, _)| norm(s))
            else {
                continue;
            };
            let Some((_, list)) = text.split_once("Those names are ") else {
                continue;
            };
            let list = list.trim().trim_end_matches('.');
            // Names containing commas ("Reveka, Wizard Savant") are separated by semicolons.
            let sep = if list.contains(';') { ';' } else { ',' };
            let names: HashSet<String> = list
                .split(sep)
                .map(|n| {
                    let n = n.trim();
                    norm(n.strip_prefix("and ").unwrap_or(n))
                })
                .filter(|n| !n.is_empty())
                .collect();
            out.insert(set, names);
        }
        out
    })
}

/// Whether `name` is a name originally printed in the named expansion (CR 206.3). Only
/// the expansions the Comprehensive Rules list are known (CR 206.3a-c).
pub fn originally_printed_in(set: &str, name: &str) -> bool {
    originally_printed_lists()
        .get(&norm(set))
        .is_some_and(|l| l.contains(&norm(name)))
}

/// Whether an object has a name originally printed in the named expansion (CR 206.3).
pub fn has_name_originally_printed_in(c: &Characteristics, set: &str) -> bool {
    c.names().any(|n| originally_printed_in(set, n))
}

/// Whether the Comprehensive Rules list the names originally printed in the named
/// expansion (CR 206.3a-c).
pub fn has_listed_names(set: &str) -> bool {
    originally_printed_lists().contains_key(&norm(set))
}

/// The expansions whose original names the Comprehensive Rules list (CR 206.3a-c).
pub fn expansions_with_listed_names() -> Vec<String> {
    let mut v: Vec<String> = originally_printed_lists().keys().cloned().collect();
    v.sort();
    v
}
