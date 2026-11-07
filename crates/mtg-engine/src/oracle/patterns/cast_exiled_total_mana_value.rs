//! Rod of Absorption: "Whenever a player casts an instant or sorcery spell, exile it
//! instead of putting it into a graveyard as it resolves." (see
//! `kw/exile_as_it_resolves_linked.rs`) and "{X}, {T}, Sacrifice this artifact: You may
//! cast any number of spells from among cards exiled with this artifact with total mana
//! value X or less without paying their mana costs." (see
//! `kw/cast_up_to_total_mana_value.rs`): the spells are chosen and cast one at a time as
//! the ability resolves, their mana values adding up to at most X.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_up_to_total_mana_value::effect_name_with;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn exile_spell_as_it_resolves(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "exile it instead of putting it into a graveyard as it resolves"
        || !matches!(b.it, Sel::TriggerSpell)
    {
        return None;
    }
    Some(Effect::Custom(
        crate::kw::exile_as_it_resolves_linked::EFFECT.into(),
    ))
}

inventory::submit! { EffectPattern { name: "exile it instead of putting it into a graveyard as it resolves (exiled with ~)", priority: 100, parse: exile_spell_as_it_resolves } }

fn cast_exiled_with_total(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "You may cast": casting none is choosing to cast no spell.
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let r = l
        .strip_prefix("cast any number of spells from among cards exiled with ~ with total mana value ")?
        .strip_suffix(" or less without paying their mana costs")?;
    let total = match r {
        "x" => None,
        n => Some(n.parse().ok()?),
    };
    Some(Effect::Custom(
        effect_name_with(u32::MAX, total, &[ZoneKind::Exile], &[]).into(),
    ))
}

inventory::submit! { EffectPattern { name: "cast any number of spells from among cards exiled with ~ with total mana value N or less", priority: 100, parse: cast_exiled_with_total } }
