//! Cards put onto the battlefield attached to something (CR 303.4f-g, 301.5c), and
//! descriptions of what could be attached: "an Aura card that could enchant that
//! creature" (Sovereigns of Lost Alara, Auratouched Mage) — a card that could legally be
//! attached to the object as it currently exists (its enchant ability, protection; the
//! object isn't targeted, so shroud and hexproof don't matter).

use super::FilterSuffixPattern;
use crate::ability::*;

/// "that could enchant it", "that could enchant that creature".
fn could_enchant<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that could enchant ")?;
    let (sel, rest) = super::filters_relational::object_in(r)?;
    Some((Filter::CouldAttachTo(Box::new(sel)), rest))
}

inventory::submit! { FilterSuffixPattern { name: "attach: that could enchant [object]", priority: 100, parse: could_enchant } }
