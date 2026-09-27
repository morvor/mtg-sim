//! "also" in an instruction that adds to what the rest of the ability does: "If this spell
//! was bargained, that creature also gains flying and lifelink until end of turn." (Archon's
//! Glory), "Colorless creatures you control also gain first strike until end of turn.",
//! "If you control a Liliana planeswalker, each opponent also discards a card.", "If X is 10
//! or more, also create X 4/4 white Angel creature tokens ...". The word only stresses that
//! this happens in addition to the rest; the instruction means the same without it.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// "[subject] also [predicate]" / "also [instruction]" → the instruction without "also".
fn also(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let text = match l.strip_prefix("also ") {
        Some(r) => r.to_string(),
        None => {
            let (subject, rest) = l.split_once(" also ")?;
            // A subject, not a clause of its own ("..., then each player also ...").
            if subject.is_empty() || subject.contains(',') || subject.contains(". ") {
                return None;
            }
            format!("{subject} {rest}")
        }
    };
    if text.contains(" also ") {
        return None;
    }
    parse_clause(&text, b)
}

inventory::submit! { EffectPattern { name: "also: an instruction adding to the rest", priority: 150, parse: also } }
