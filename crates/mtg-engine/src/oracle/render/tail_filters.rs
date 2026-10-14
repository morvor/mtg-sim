//! More object qualities implemented in code (`Filter::Custom`), put into words from what
//! each one checks (see [`super::custom_filters::custom_rel`]).

use super::custom_filters::CustomQuality;

/// The wording of a custom quality, if this module knows it.
pub(crate) fn custom_rel_more(name: &str) -> Option<CustomQuality> {
    use CustomQuality::*;
    // "a land that could produce {C}" (CR 106.7): one of its mana abilities could
    // produce mana of that type.
    if let Some(t) = name.strip_prefix(crate::kw::mana_grammar::COULD_PRODUCE) {
        let letter = t.chars().next()?;
        crate::mana::ManaType::from_letter(letter)?;
        return Some(Rel(format!(
            "that could produce {{{}}}",
            letter.to_ascii_uppercase()
        )));
    }
    Some(match name {
        // "each creature you control with a mana ability" (CR 605.1a).
        crate::kw::activated_ability_kind::HAS_MANA_ABILITY => Rel("with a mana ability".into()),
        // "protection from Ring-bearers": each player's Ring-bearer (CR 701.54a).
        crate::kw::game_rules::ANY_RING_BEARER => Kind("Ring-bearer"),
        // "When you cast an Equipment spell this way": cast with the permission the
        // ability or its source gave.
        crate::permissions::PLAYED_THIS_WAY => Rel("this way".into()),
        _ => return None,
    })
}
