//! "Destroy target blocking creature. Creatures that were blocked by that creature this
//! combat gain trample until end of turn." (Ride Down): the attacking creatures the target
//! creature was declared to block this combat (CR 509.1a), judged as it last existed if it
//! has left the battlefield (CR 608.2h). See
//! [`crate::kw::blocked_this_turn::BLOCKED_BY_TARGET_THIS_COMBAT`].

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Stand-in for "that were blocked by that creature this combat" while the sentence is
/// parsed.
const MARK: &str = "\u{2}blocked-by-target-this-combat";

fn blocked_by_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let rest = end(l).strip_prefix("creatures that were blocked by that creature this combat ")?;
    // "that creature": the spell's single target.
    if !matches!(b.it, Sel::Target(0)) || b.targets.len() != 1 {
        return None;
    }
    parse_sentence(&format!("creatures {MARK} {rest}"), b)
}

inventory::submit! { EffectPattern { name: "creatures that were blocked by that creature this combat", priority: 120, parse: blocked_by_target } }

fn mark<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s.strip_prefix(MARK)?;
    Some((
        Filter::Custom(crate::kw::blocked_this_turn::BLOCKED_BY_TARGET_THIS_COMBAT.into()),
        r,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "that were blocked by target creature this combat", priority: 100, parse: mark } }
