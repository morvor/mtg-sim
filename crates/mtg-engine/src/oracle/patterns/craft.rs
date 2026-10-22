//! Craft (CR 702.167, see `kw/craft.rs`): "the exiled card(s) used to craft it" in value
//! phrases ("the mana value of the exiled card used to craft it", "the total power of the
//! exiled cards used to craft it", CR 702.167c). The keyword line "Craft with [materials]
//! [cost]" is parsed in `k702_153_167.rs`.

use crate::ability::*;

/// "the exiled card(s) used to craft it" (or "~"). Returns the rest of the text.
fn used_to_craft(s: &str) -> Option<&str> {
    let r = s
        .strip_prefix("the exiled cards used to craft ")
        .or_else(|| s.strip_prefix("the exiled card used to craft "))?;
    r.strip_prefix("it").or_else(|| r.strip_prefix('~'))
}

/// Values of the exiled cards used to craft the source (CR 702.167c): "the mana value of
/// the exiled card used to craft it", "the total mana value of ...", "the total power of
/// the exiled cards used to craft it". Returns the value and the rest of the text.
pub fn used_to_craft_value(s: &str) -> Option<(Value, String)> {
    let used = || Box::new(crate::kw::craft::used_to_craft_sel());
    if let Some(r) = s
        .strip_prefix("the mana value of ")
        .or_else(|| s.strip_prefix("the total mana value of "))
    {
        let rest = used_to_craft(r)?;
        return Some((Value::ManaValueOf(used()), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("the total power of ") {
        let rest = used_to_craft(r)?;
        return Some((Value::PowerOf(used()), rest.to_string()));
    }
    None
}
