//! "[creature] that's attacking you" and "[creature] that's attacking you or a
//! planeswalker you control" after an object phrase (CR 506.2, 508.1b): "target creature
//! without flying that's attacking you" (Hunting Kavu, Ice Floe), "target creature that's
//! attacking you or a planeswalker you control" (Soul Snare). A creature attacking a
//! planeswalker isn't attacking you; one whose planeswalker left combat is attacking
//! neither (CR 506.4c).

use crate::ability::*;

fn that_is_attacking_you<'a>(t: &'a str, f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that's attacking you")?;
    if !serde_json::to_string(f).is_ok_and(|s| s.contains("\"Creature\"")) {
        return None;
    }
    if let Some(rest) = r.strip_prefix(" or a planeswalker you control") {
        if !(rest.is_empty() || rest.starts_with([',', '.'])) {
            return None;
        }
        let you_or_yours = Filter::Custom(
            crate::kw::activated_ability_kind::ATTACKING_YOU_OR_YOUR_PLANESWALKER.into(),
        );
        return Some((you_or_yours, rest));
    }
    if !(r.is_empty() || r.starts_with([',', '.'])) {
        return None;
    }
    Some((Filter::AttackingPlayer(PlayerRel::You), r))
}

inventory::submit! { super::FilterSuffixPattern { name: "r508: that's attacking you [or a planeswalker you control]", priority: 100, parse: that_is_attacking_you } }
