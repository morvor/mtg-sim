//! "Choose target artifact creature you control. For each creature chosen this way,
//! create a token that's a copy of it." (March of Progress): the instruction is performed
//! once for each creature the earlier sentence chose, "it" being that creature. With one
//! target chosen, that's the instruction about the target; overloaded (CR 702.96a, the
//! target becoming "each"), the instruction applies to each creature it affects.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

fn for_each_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each creature chosen this way, ")?;
    let (slot, _) = b.chosen_creature.clone()?;
    let chosen = Sel::Target(slot);
    let saved = (b.targets.len(), b.it.clone());
    b.it = chosen;
    let e = parse_clause(r, b);
    // Only instructions about "it" (the chosen creature) and nothing newly targeted.
    let ok = matches!(
        &e,
        Some(Effect::CreateTokenCopy { of: Sel::Target(s), .. }) if *s == slot
    ) && b.targets.len() == saved.0;
    if !ok {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    e
}

inventory::submit! { EffectPattern { name: "for each creature chosen this way, [instruction about it]", priority: 90, parse: for_each_chosen } }
